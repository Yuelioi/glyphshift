use super::*;
use glyphshift_domain::{DeferredGlyph, DeferredTextDraw, GlyphStyle};
use retour::GenericDetour;
use std::{
    cell::RefCell,
    ffi::c_void,
    sync::{atomic::AtomicUsize, OnceLock},
};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError},
    System::Threading::GetCurrentProcess,
};
mod contexts;
mod discovery;
mod shadow;

#[derive(Clone, Copy)]
struct Layout {
    font: usize,
    color: usize,
    pos: usize,
    get_pos: usize,
    print: usize,
    character: usize,
    flush: usize,
    measure: usize,
    width: usize,
    query_start: usize,
}
type Factory = unsafe extern "C" fn(*const u8, *mut i32) -> *mut c_void;
type Paint = unsafe extern "thiscall" fn(*mut c_void);
type Font = unsafe extern "thiscall" fn(*mut c_void, u32);
type Color = unsafe extern "thiscall" fn(*mut c_void, i32, i32, i32, i32);
type PackedColor = unsafe extern "thiscall" fn(*mut c_void, u32);
type Pos = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type GetPos = unsafe extern "thiscall" fn(*mut c_void, *mut i32, *mut i32);
type Print = unsafe extern "thiscall" fn(*mut c_void, *const u16, i32, i32);
type Character = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type Flush = unsafe extern "thiscall" fn(*mut c_void);
type Measure = unsafe extern "thiscall" fn(*mut c_void, u32, *const u16, *mut i32, *mut i32);
type Width = unsafe extern "thiscall" fn(*mut c_void, u32, i32) -> i32;
type PushCurrent = unsafe extern "thiscall" fn(*mut c_void, u32, bool);
type PopCurrent = unsafe extern "thiscall" fn(*mut c_void, u32);

struct Image {
    target: usize,
    hook: GenericDetour<Paint>,
    _module: discovery::Pin,
}
struct Hooks {
    factory: Factory,
    surface: usize,
    original_table: usize,
    _shadow: shadow::Shadow,
    _surface_module: discovery::Pin,
    font: Font,
    color: Color,
    packed_color: PackedColor,
    pos: Pos,
    get_pos: GetPos,
    print: Print,
    character: Character,
    flush: Flush,
    measure: Measure,
    width: Width,
    push_current: PushCurrent,
    pop_current: PopCurrent,
    images: Vec<Image>,
}
static HOOKS: OnceLock<Hooks> = OnceLock::new();
static INSTALL: Mutex<()> = Mutex::new(());
#[derive(Default, Clone, Copy)]
struct State {
    font: Option<u32>,
    color: Option<[i32; 4]>,
    pos: Option<(i32, i32)>,
    cursor_known: bool,
}
struct Frame {
    panel: Option<u32>,
    draw: DeferredTextDraw,
    disabled: bool,
    font_seen: bool,
    color_seen: bool,
    scope: Option<NativeTextScope>,
    host: NativeTextHostBinding,
}
#[derive(Default)]
struct Thread {
    state: State,
    frame: Option<Frame>,
    replay: bool,
    panels: Vec<contexts::PanelScope>,
    ignored_panels: u32,
}
thread_local! {static THREAD:RefCell<Thread>=RefCell::new(Thread::default());}
fn thread<T>(call: impl FnOnce(&mut Thread) -> T) -> T {
    THREAD.with(|state| call(&mut state.borrow_mut()))
}
fn replaying() -> bool {
    thread(|state| state.replay)
}
struct Replay(bool);
impl Replay {
    fn enter() -> Self {
        Self(thread(|state| {
            let old = state.replay;
            state.replay = true;
            old
        }))
    }
}
impl Drop for Replay {
    fn drop(&mut self) {
        thread(|state| state.replay = self.0);
    }
}
struct Flight;
impl Drop for Flight {
    fn drop(&mut self) {
        IN_FLIGHT.fetch_sub(1, Ordering::AcqRel);
    }
}

unsafe fn replay(glyphs: &[DeferredGlyph], tail: State) {
    if glyphs.is_empty() {
        return;
    }
    let Some(h) = HOOKS.get() else {
        return;
    };
    let _replay = Replay::enter();
    let surface = h.surface as *mut _;
    for glyph in glyphs {
        (h.font)(surface, glyph.style.font as u32);
        let color = glyph.style.color.to_le_bytes();
        (h.color)(
            surface,
            color[0].into(),
            color[1].into(),
            color[2].into(),
            color[3].into(),
        );
        (h.pos)(surface, glyph.x, glyph.y);
        (h.character)(surface, glyph.unit, glyph.style.draw_kind as i32);
    }
    if let Some(font) = tail.font {
        (h.font)(surface, font);
    }
    if let Some([r, g, b, a]) = tail.color {
        (h.color)(surface, r, g, b, a);
    }
    if tail.cursor_known {
        if let Some((x, y)) = tail.pos {
            (h.pos)(surface, x, y);
        }
    }
    sync_position(h);
}
unsafe fn sync_position(h: &Hooks) {
    let mut x = 0;
    let mut y = 0;
    (h.get_pos)(h.surface as *mut _, &mut x, &mut y);
    thread(|state| {
        state.state.pos = Some((x, y));
        state.state.cursor_known = false;
    });
}
unsafe fn abort(force: bool) {
    if replaying() {
        return;
    }
    let pending = thread(|state| {
        let tail = state.state;
        let frame = state.frame.as_mut()?;
        if frame.disabled || (!force && frame.draw.is_empty()) {
            return None;
        }
        frame.disabled = true;
        frame.scope.take();
        Some((std::mem::take(&mut frame.draw).finish(), tail))
    });
    if let Some((draw, tail)) = pending {
        replay(draw.original(), tail);
    }
}
extern "C" fn barrier() {
    unsafe {
        let error = GetLastError();
        if !replaying() {
            if thread(|state| state.frame.is_some()) {
                PAINT_STATUS[4].fetch_add(1, Ordering::Relaxed);
            }
            abort(false);
            thread(|state| {
                state.state = State::default();
                if let Some(frame) = state.frame.as_mut() {
                    frame.font_seen = false;
                    frame.color_seen = false;
                }
            });
        }
        SetLastError(error);
    }
}

unsafe extern "thiscall" fn set_font(surface: *mut c_void, font: u32) {
    let h = HOOKS.get().unwrap();
    if surface as usize != h.surface {
        abort(false);
        (h.font)(surface, font);
        return;
    }
    if !replaying() && thread(|state| state.state.font != Some(font)) {
        abort(false);
    }
    (h.font)(surface, font);
    if !replaying() {
        thread(|state| {
            state.state.font = Some(font);
            if let Some(frame) = state.frame.as_mut() {
                frame.font_seen = true;
            }
        });
    }
}
unsafe extern "thiscall" fn set_color(surface: *mut c_void, r: i32, g: i32, b: i32, a: i32) {
    let h = HOOKS.get().unwrap();
    if surface as usize != h.surface {
        abort(false);
        (h.color)(surface, r, g, b, a);
        return;
    }
    let color = [r, g, b, a];
    if !replaying() && thread(|state| state.state.color != Some(color)) {
        abort(false);
    }
    (h.color)(surface, r, g, b, a);
    if !replaying() {
        thread(|state| {
            state.state.color = Some(color);
            if let Some(frame) = state.frame.as_mut() {
                frame.color_seen = true;
            }
        });
    }
}
unsafe extern "thiscall" fn set_packed_color(surface: *mut c_void, packed: u32) {
    let h = HOOKS.get().unwrap();
    if surface as usize != h.surface {
        abort(false);
        (h.packed_color)(surface, packed);
        return;
    }
    let color = packed.to_le_bytes().map(i32::from);
    if !replaying() && thread(|state| state.state.color != Some(color)) {
        abort(false);
    }
    (h.packed_color)(surface, packed);
    if !replaying() {
        thread(|state| {
            state.state.color = Some(color);
            if let Some(frame) = state.frame.as_mut() {
                frame.color_seen = true;
            }
        });
    }
}
unsafe extern "thiscall" fn set_pos(surface: *mut c_void, x: i32, y: i32) {
    let h = HOOKS.get().unwrap();
    if surface as usize != h.surface {
        abort(false);
        (h.pos)(surface, x, y);
        return;
    }
    (h.pos)(surface, x, y);
    if !replaying() {
        thread(|state| {
            state.state.pos = Some((x, y));
            state.state.cursor_known = true;
        });
    }
}
unsafe extern "thiscall" fn get_pos(surface: *mut c_void, x: *mut i32, y: *mut i32) {
    if !replaying() {
        abort(false);
    }
    (HOOKS.get().unwrap().get_pos)(surface, x, y);
}
unsafe extern "thiscall" fn flush(surface: *mut c_void) {
    if !replaying() {
        abort(false);
    }
    (HOOKS.get().unwrap().flush)(surface);
}

unsafe extern "thiscall" fn character(surface: *mut c_void, unit: u16, kind: i32) {
    let h = HOOKS.get().unwrap();
    if surface as usize != h.surface {
        abort(false);
        (h.character)(surface, unit, kind);
        return;
    }
    if replaying() || ACTIVE.load(Ordering::Acquire) == 0 {
        (h.character)(surface, unit, kind);
        return;
    }
    let input = thread(|state| {
        if state.frame.is_none() {
            PAINT_STATUS[7].fetch_add(1, Ordering::Relaxed);
        }
        let frame = state.frame.as_ref()?;
        if frame.disabled || !frame.font_seen || !frame.color_seen || !state.state.cursor_known {
            MISSING_STATE.store(
                i32::from(frame.disabled)
                    | (i32::from(!frame.font_seen) << 1)
                    | (i32::from(!frame.color_seen) << 2)
                    | (i32::from(!state.state.cursor_known) << 3)
                    | (i32::from(state.state.font.is_none()) << 4)
                    | (i32::from(state.state.color.is_none()) << 5)
                    | (i32::from(state.state.pos.is_none()) << 6),
                Ordering::Relaxed,
            );
            PAINT_STATUS[5].fetch_add(1, Ordering::Relaxed);
            return None;
        }
        let font = state.state.font?;
        let color = state.state.color?;
        let (x, y) = state.state.pos?;
        if font == 0 || color[3] == 0 || color.iter().any(|v| !(0..=255).contains(v)) {
            return None;
        }
        Some((font, u32::from_le_bytes(color.map(|v| v as u8)), x, y))
    });
    if let Some((font, color, x, y)) = input {
        let advance = (h.width)(surface, font, i32::from(unit));
        let glyph = DeferredGlyph {
            unit,
            x,
            y,
            advance,
            style: GlyphStyle {
                font: font.into(),
                color,
                draw_kind: kind as u32,
            },
        };
        let buffered = thread(|state| {
            let Some(next_x) = x.checked_add(advance) else {
                return false;
            };
            let accepted = state
                .frame
                .as_mut()
                .is_some_and(|frame| !frame.disabled && frame.draw.push(glyph).is_ok());
            if accepted {
                // DrawUnicodeChar advances the native pen. While the draw is
                // deferred, preserve that public effect in the shadow cursor.
                state.state.pos = Some((next_x, y));
                state.state.cursor_known = true;
            }
            accepted
        });
        if buffered {
            PAINT_STATUS[2].fetch_add(1, Ordering::Relaxed);
            return;
        }
        PAINT_STATUS[6].fetch_add(1, Ordering::Relaxed);
    }
    abort(true);
    (h.character)(surface, unit, kind);
    sync_position(h);
}

unsafe fn translated(
    host: NativeTextHostBinding,
    source: &[u16],
    font: Option<u32>,
    maximum: Option<i32>,
) -> Option<Vec<u16>> {
    if STOPPING.load(Ordering::Acquire)
        || ACTIVE.load(Ordering::Acquire) & FEATURE_TEXT_REPLACE == 0
        || font.is_none()
        || maximum.is_none_or(|width| width <= 0)
        || source
            .iter()
            .any(|unit| *unit < 32 || (0xd800..=0xdfff).contains(unit))
    {
        let mut event = NativeTextEventV1::complete_draw(source);
        event.kind = TEXT_EVENT_OBSERVE;
        host.decide(&event, &mut [], &mut []);
        return None;
    }
    let h = HOOKS.get()?;
    let mut output = vec![0; 16 * 1024];
    let result = host.decide(
        &NativeTextEventV1::complete_draw(source),
        &mut output,
        &mut [],
    );
    if result.status != STATUS_OK {
        DRAW_STATUS.host_errors.fetch_add(1, Ordering::Relaxed);
        DRAW_STATUS
            .last_host_status
            .store(result.status, Ordering::Relaxed);
    }
    if result.status != STATUS_OK
        || result.decision_bits & DECISION_TEXT_REPLACE == 0
        || result.text_len == 0
        || result.text_len as usize > output.len()
    {
        return None;
    }
    output.truncate(result.text_len as usize);
    DRAW_STATUS.matched.fetch_add(1, Ordering::Relaxed);
    if output
        .iter()
        .any(|unit| *unit < 32 || (0xd800..=0xdfff).contains(unit))
    {
        return None;
    }
    output.push(0);
    let mut width = 0;
    let mut height = 0;
    (h.measure)(
        h.surface as *mut _,
        font?,
        output.as_ptr(),
        &mut width,
        &mut height,
    );
    DRAW_STATUS.measured.fetch_add(1, Ordering::Relaxed);
    DRAW_STATUS.width.store(width, Ordering::Relaxed);
    DRAW_STATUS.height.store(height, Ordering::Relaxed);
    DRAW_STATUS
        .maximum
        .store(maximum.unwrap_or(0), Ordering::Relaxed);
    if width <= 0 || height <= 0 || maximum.is_none_or(|maximum| width > maximum) {
        DRAW_STATUS.rejected.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    DRAW_STATUS.accepted.fetch_add(1, Ordering::Relaxed);
    Some(output)
}
unsafe extern "thiscall" fn print(surface: *mut c_void, text: *const u16, length: i32, kind: i32) {
    let h = HOOKS.get().unwrap();
    if surface as usize != h.surface {
        abort(false);
        (h.print)(surface, text, length, kind);
        return;
    }
    if replaying()
        || ACTIVE.load(Ordering::Acquire) == 0
        || (length <= 0 || length > 16 * 1024)
        || text.is_null()
    {
        (h.print)(surface, text, length, kind);
        return;
    }
    IN_FLIGHT.fetch_add(1, Ordering::AcqRel);
    let _flight = Flight;
    if STOPPING.load(Ordering::Acquire) || ACTIVE.load(Ordering::Acquire) == 0 {
        (h.print)(surface, text, length, kind);
        return;
    }
    // PrintText already supplies a complete drawing boundary. Its most recent
    // explicit state remains usable until an opaque operation invalidates it;
    // an enclosing TextImage is only required for deferred character calls.
    let font = thread(|state| {
        let color = state.state.color?;
        (state.state.cursor_known
            && color[3] > 0
            && color.iter().all(|value| (0..=255).contains(value)))
        .then_some(state.state.font)
        .flatten()
        .filter(|font| *font != 0)
    });
    // A complete call cannot join a buffered prefix, but an empty parent still
    // owns any later glyph run up to its explicit context end.
    abort(false);
    let Some(host) = HOST.lock().ok().and_then(|host| *host) else {
        (h.print)(surface, text, length, kind);
        return;
    };
    let Some(_scope) = host.enter_scope() else {
        (h.print)(surface, text, length, kind);
        return;
    };
    let source = std::slice::from_raw_parts(text, length as usize);
    let mut terminated = source.to_vec();
    terminated.push(0);
    let mut width = 0;
    let mut height = 0;
    if let Some(font) = font {
        (h.measure)(surface, font, terminated.as_ptr(), &mut width, &mut height);
    }
    let replacement = translated(host, source, font, Some(width));
    let _replay = Replay::enter();
    if let Some(replacement) = replacement {
        DRAW_STATUS.direct_draws.fetch_add(1, Ordering::Relaxed);
        let mut x = 0;
        let mut y = 0;
        (h.get_pos)(surface, &mut x, &mut y);
        DRAW_STATUS.x.store(x, Ordering::Relaxed);
        DRAW_STATUS.y.store(y, Ordering::Relaxed);
        (h.print)(
            surface,
            replacement.as_ptr(),
            replacement.len() as i32 - 1,
            kind,
        );
    } else {
        (h.print)(surface, text, length, kind);
    }
    sync_position(h);
}

unsafe extern "thiscall" fn paint(object: *mut c_void) {
    PAINT_STATUS[0].fetch_add(1, Ordering::Relaxed);
    let Some(h) = HOOKS.get() else {
        return;
    };
    let Some(table) = discovery::read::<usize>(object as usize) else {
        return;
    };
    let Some(target) = discovery::read::<usize>(table) else {
        return;
    };
    let Some(image) = h.images.iter().find(|image| image.target == target) else {
        return;
    };
    if ACTIVE.load(Ordering::Acquire) == 0
        || STOPPING.load(Ordering::Acquire)
        || (h.factory)(c"VGUI_Surface031".as_ptr().cast(), std::ptr::null_mut()) as usize
            != h.surface
    {
        abort(true);
        image.hook.call(object);
        return;
    }
    IN_FLIGHT.fetch_add(1, Ordering::AcqRel);
    let _flight = Flight;
    if STOPPING.load(Ordering::Acquire) || ACTIVE.load(Ordering::Acquire) == 0 {
        abort(true);
        image.hook.call(object);
        return;
    }
    abort(true);
    let Some(host) = HOST.lock().ok().and_then(|host| *host) else {
        image.hook.call(object);
        return;
    };
    let Some(scope) = host.enter_scope() else {
        image.hook.call(object);
        return;
    };
    let parent = thread(|state| {
        PAINT_STATUS[1].fetch_add(1, Ordering::Relaxed);
        state.state.cursor_known = false;
        state.frame.replace(Frame {
            panel: None,
            draw: DeferredTextDraw::default(),
            disabled: false,
            font_seen: false,
            color_seen: false,
            scope: Some(scope),
            host,
        })
    });
    image.hook.call(object);
    let (frame, tail) = thread(|state| (state.frame.take().unwrap(), state.state));
    finish_frame(h, frame, tail);
    thread(|state| state.frame = parent);
}

unsafe fn finish_frame(h: &Hooks, frame: Frame, tail: State) {
    if frame.disabled || frame.draw.is_empty() {
        PAINT_STATUS[3].fetch_add(1, Ordering::Relaxed);
    }
    if !frame.disabled {
        let commit = frame.draw.finish();
        let replacement = if let Some(source) = commit.source() {
            let units = source.encode_utf16().collect::<Vec<_>>();
            let maximum = commit
                .original()
                .first()
                .zip(commit.original().last())
                .and_then(|(first, last)| last.x.checked_add(last.advance)?.checked_sub(first.x));
            translated(
                frame.host,
                &units,
                commit.origin().map(|(_, _, style)| style.font as u32),
                maximum,
            )
        } else {
            None
        };
        if let Some(replacement) = replacement {
            let _replay = Replay::enter();
            let (x, y, style) = commit.origin().unwrap();
            DRAW_STATUS.deferred_draws.fetch_add(1, Ordering::Relaxed);
            DRAW_STATUS.x.store(x, Ordering::Relaxed);
            DRAW_STATUS.y.store(y, Ordering::Relaxed);
            let color = style.color.to_le_bytes();
            let surface = h.surface as *mut _;
            (h.font)(surface, style.font as u32);
            (h.color)(
                surface,
                color[0].into(),
                color[1].into(),
                color[2].into(),
                color[3].into(),
            );
            (h.pos)(surface, x, y);
            (h.print)(
                surface,
                replacement.as_ptr(),
                replacement.len() as i32 - 1,
                style.draw_kind as i32,
            );
            if tail.cursor_known {
                if let Some((x, y)) = tail.pos {
                    (h.pos)(surface, x, y);
                }
            }
            sync_position(h);
        } else {
            replay(commit.original(), tail);
        }
    }
    drop(frame.scope);
}

pub(super) unsafe fn install() -> Result<(), ()> {
    let _install = INSTALL.lock().map_err(|_| ())?;
    if let Some(h) = HOOKS.get() {
        if (h.factory)(c"VGUI_Surface031".as_ptr().cast(), std::ptr::null_mut()) as usize
            != h.surface
            || discovery::read::<usize>(h.surface) != Some(h._shadow.table)
        {
            return Err(());
        }
        for image in &h.images {
            if !image.hook.is_enabled() {
                image.hook.enable().map_err(|_| ())?;
            }
        }
        ACTIVATION_STAGE.store(50, Ordering::Release);
        return Ok(());
    }
    ACTIVATION_STAGE.store(10, Ordering::Release);
    let (held, surface, entries, layout, factory) = discovery::surface().ok_or(())?;
    let original_table = discovery::read::<usize>(surface).ok_or(())?;
    let font_factory = discovery::measure_factory(entries[layout.measure]).ok_or(())?;
    for (slot, cleanup) in [
        (layout.font, 4),
        (layout.color, 16),
        (layout.color - 1, 4),
        (layout.pos, 8),
        (layout.get_pos, 8),
        (layout.print, 12),
        (layout.character, 8),
        (layout.flush, 0),
        (layout.measure, 16),
        (layout.width, 8),
        (layout.font - 9, 8),
        (layout.font - 8, 4),
    ] {
        if !discovery::cleanup_matches(entries[slot], cleanup) {
            ACTIVATION_STAGE.store(-1000 - slot as i32, Ordering::Release);
            return Err(());
        }
    }
    let mut queries = Vec::new();
    for (slot, entry) in entries
        .iter()
        .enumerate()
        .take(layout.measure + 1)
        .skip(layout.query_start)
    {
        if discovery::font_query(*entry, font_factory) {
            queries.push(slot);
        }
    }
    if !queries.contains(&layout.width) {
        return Err(());
    }
    let overrides = [
        (layout.font - 9, contexts::push as *const () as usize),
        (layout.font - 8, contexts::pop as *const () as usize),
        (layout.font, set_font as *const () as usize),
        (layout.color, set_color as *const () as usize),
        (layout.color - 1, set_packed_color as *const () as usize),
        (layout.pos, set_pos as *const () as usize),
        (layout.get_pos, get_pos as *const () as usize),
        (layout.print, print as *const () as usize),
        (layout.character, character as *const () as usize),
        (layout.flush, flush as *const () as usize),
    ];
    let table = shadow::create(original_table, &entries, &overrides, &queries)?;
    ACTIVATION_STAGE.store(30, Ordering::Release);
    let mut images = Vec::new();
    for (module, _table, target) in discovery::images() {
        if !discovery::cleanup_matches(target, 0) {
            continue;
        }
        let method: Paint = std::mem::transmute(target);
        let hook = GenericDetour::new(method, paint).map_err(|_| ())?;
        images.push(Image {
            target,
            hook,
            _module: module,
        });
    }
    if images.is_empty() {
        return Err(());
    }
    let h = Hooks {
        factory,
        surface,
        original_table,
        _shadow: table,
        _surface_module: held,
        font: std::mem::transmute::<usize, Font>(entries[layout.font]),
        color: std::mem::transmute::<usize, Color>(entries[layout.color]),
        packed_color: std::mem::transmute::<usize, PackedColor>(entries[layout.color - 1]),
        pos: std::mem::transmute::<usize, Pos>(entries[layout.pos]),
        get_pos: std::mem::transmute::<usize, GetPos>(entries[layout.get_pos]),
        print: std::mem::transmute::<usize, Print>(entries[layout.print]),
        character: std::mem::transmute::<usize, Character>(entries[layout.character]),
        flush: std::mem::transmute::<usize, Flush>(entries[layout.flush]),
        measure: std::mem::transmute::<usize, Measure>(entries[layout.measure]),
        width: std::mem::transmute::<usize, Width>(entries[layout.width]),
        push_current: std::mem::transmute::<usize, PushCurrent>(entries[layout.font - 9]),
        pop_current: std::mem::transmute::<usize, PopCurrent>(entries[layout.font - 8]),
        images,
    };
    HOOKS.set(h).map_err(|_| ())?;
    let h = HOOKS.get().unwrap();
    ACTIVATION_STAGE.store(40, Ordering::Release);
    let pointer = &*(surface as *const AtomicUsize);
    pointer
        .compare_exchange(
            h.original_table,
            h._shadow.table,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .map_err(|_| ())?;
    for image in &h.images {
        image.hook.enable().map_err(|_| ())?;
    }
    ACTIVATION_STAGE.store(50, Ordering::Release);
    Ok(())
}
