//! Public Surface context pairs bound custom panel painting without inspecting
//! any panel fields, class names, or private virtual method layouts.
use super::*;

pub(super) struct PanelScope {
    panel: u32,
    parent: Option<Frame>,
    _flight: Flight,
}

pub(super) unsafe extern "thiscall" fn push(surface: *mut c_void, panel: u32, insets: bool) {
    let h = HOOKS.get().unwrap();
    if replaying() || surface as usize != h.surface {
        if !replaying() {
            abort(true);
        }
        (h.push_current)(surface, panel, insets);
        return;
    }
    abort(true);
    {
        let _replay = Replay::enter();
        (h.push_current)(surface, panel, insets);
    }
    let eligible = thread(|state| {
        state.state = State::default();
        let eligible = state.ignored_panels == 0
            && state.panels.len() < 32
            && state
                .frame
                .as_ref()
                .is_none_or(|frame| frame.panel.is_some())
            && panel != 0
            && ACTIVE.load(Ordering::Acquire) != 0
            && !STOPPING.load(Ordering::Acquire);
        if !eligible && (!state.panels.is_empty() || state.ignored_panels != 0) {
            state.ignored_panels = state.ignored_panels.saturating_add(1);
        }
        eligible
    });
    if !eligible {
        return;
    }
    IN_FLIGHT.fetch_add(1, Ordering::AcqRel);
    let flight = Flight;
    if STOPPING.load(Ordering::Acquire) || ACTIVE.load(Ordering::Acquire) == 0 {
        thread(|state| state.ignored_panels = state.ignored_panels.saturating_add(1));
        return;
    }
    let host = HOST.lock().ok().and_then(|host| *host);
    let bound = host.and_then(|host| host.enter_scope().map(|scope| (host, scope)));
    let Some((host, scope)) = bound else {
        thread(|state| state.ignored_panels = state.ignored_panels.saturating_add(1));
        return;
    };
    thread(|state| {
        let parent = state.frame.replace(Frame {
            panel: Some(panel),
            draw: DeferredTextDraw::default(),
            disabled: false,
            font_seen: false,
            color_seen: false,
            scope: Some(scope),
            host,
        });
        state.panels.push(PanelScope {
            panel,
            parent,
            _flight: flight,
        });
    });
}

pub(super) unsafe extern "thiscall" fn pop(surface: *mut c_void, panel: u32) {
    let h = HOOKS.get().unwrap();
    if replaying() || surface as usize != h.surface {
        if !replaying() {
            abort(true);
        }
        (h.pop_current)(surface, panel);
        return;
    }
    let ignored = thread(|state| {
        if state.ignored_panels == 0 {
            return false;
        }
        state.ignored_panels -= 1;
        true
    });
    if ignored {
        let _replay = Replay::enter();
        (h.pop_current)(surface, panel);
        return;
    }
    let paired = thread(|state| {
        state.panels.last().is_some_and(|top| top.panel == panel)
            && state
                .frame
                .as_ref()
                .is_some_and(|frame| frame.panel == Some(panel))
    });
    if paired {
        let (scope, frame, tail) = thread(|state| {
            (
                state.panels.pop().unwrap(),
                state.frame.take().unwrap(),
                state.state,
            )
        });
        // Commit while the native panel transform and clipping are still current.
        finish_frame(h, frame, tail);
        {
            let _replay = Replay::enter();
            (h.pop_current)(surface, panel);
        }
        thread(|state| {
            state.frame = scope.parent;
            state.state = State::default();
        });
    } else {
        abort(true);
        // Preserve a live Image wrapper's frame; its lexical owner will release it.
        thread(|state| {
            if state
                .frame
                .as_ref()
                .is_some_and(|frame| frame.panel.is_some())
            {
                state.frame = None;
                state.panels.clear();
            }
            state.state = State::default();
        });
        let _replay = Replay::enter();
        (h.pop_current)(surface, panel);
    }
}
