//! MSVC member functions use ECX for this on x86. Static methods remain cdecl.
use super::*;
macro_rules! members {
    ($abi:literal) => {
        pub(super) type FnDrawPointSimple =
            unsafe extern $abi fn(*mut c_void, *const c_void, *const c_void);
        pub(super) type FnDrawPoint = unsafe extern $abi fn(*mut c_void, *const c_void, *const c_void, i32, i32);
        pub(super) type FnDrawRect =
            unsafe extern $abi fn(*mut c_void, *const c_void, i32, *const c_void, *mut c_void);
        pub(super) type FnDrawRectOption =
            unsafe extern $abi fn(*mut c_void, *const c_void, *const c_void, *const c_void);
        pub(super) type FnDrawRectCoords = unsafe extern $abi fn(
            *mut c_void,
            i32,
            i32,
            i32,
            i32,
            i32,
            *const c_void,
            *mut c_void,
        );
        pub(super) type FnQString5Ctor = unsafe extern $abi fn(*mut c_void, *const u16, i32) -> *mut c_void;
        pub(super) type FnQString6Ctor = unsafe extern $abi fn(*mut c_void, *const u16, isize) -> *mut c_void;
        pub(super) type FnQStringDtor = unsafe extern $abi fn(*mut c_void);
        pub(super) type FnQString5Size = unsafe extern $abi fn(*const c_void) -> i32;
        pub(super) type FnQString6Size = unsafe extern $abi fn(*const c_void) -> isize;
        pub(super) type FnQStringUtf16 = unsafe extern $abi fn(*const c_void) -> *const u16;
        pub(super) type FnQWidgetRepaint = unsafe extern $abi fn(*mut c_void);
        pub(super) unsafe extern $abi fn draw_point_simple_detour(
            painter: *mut c_void,
            point: *const c_void,
            text: *const c_void,
        ) {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            let Some(point_simple) = hooks.point_simple.as_ref() else {
                return;
            };
            draw_rect_with(
                hooks,
                text,
                || point_simple.call(painter, point, text),
                |replacement| point_simple.call(painter, point, replacement),
            );
        }
        pub(super) unsafe extern $abi fn draw_point_detour(
            painter: *mut c_void,
            point: *const c_void,
            text: *const c_void,
            from: i32,
            length: i32,
        ) {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            if ACTIVE_FEATURES.load(Ordering::Acquire) == 0 {
                return hooks.point.call(painter, point, text, from, length);
            }
            let Some(_guard) = CallbackGuard::enter() else {
                return hooks.point.call(painter, point, text, from, length);
            };
            let Some(full_text) = hooks.strings.read(text) else {
                return hooks.point.call(painter, point, text, from, length);
            };
            let Some(source) = drawn_point_text(&full_text, from, length) else {
                return hooks.point.call(painter, point, text, from, length);
            };
            if !eligible_source(&source) {
                return hooks.point.call(painter, point, text, from, length);
            }
            let _scope = text_scope();
            let Some(replacement) = replacement_for(&source) else {
                return hooks.point.call(painter, point, text, from, length);
            };
            if hooks
                .strings
                .with_temporary(&replacement, |replacement| {
                    hooks.point.call(painter, point, replacement, 0, -1);
                })
                .is_none()
            {
                hooks.point.call(painter, point, text, from, length);
            }
        }
        pub(super) unsafe extern $abi fn draw_rect_detour(
            painter: *mut c_void,
            rect: *const c_void,
            flags: i32,
            text: *const c_void,
            bounding_rect: *mut c_void,
        ) {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            draw_rect_with(
                hooks,
                text,
                || hooks.rect.call(painter, rect, flags, text, bounding_rect),
                |replacement| {
                    hooks
                        .rect
                        .call(painter, rect, flags, replacement, bounding_rect);
                },
            );
        }
        pub(super) unsafe extern $abi fn draw_rect_f_detour(
            painter: *mut c_void,
            rect: *const c_void,
            flags: i32,
            text: *const c_void,
            bounding_rect: *mut c_void,
        ) {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            draw_rect_with(
                hooks,
                text,
                || hooks.rect_f.call(painter, rect, flags, text, bounding_rect),
                |replacement| {
                    hooks
                        .rect_f
                        .call(painter, rect, flags, replacement, bounding_rect);
                },
            );
        }
        pub(super) unsafe extern $abi fn draw_rect_option_detour(
            painter: *mut c_void,
            rect: *const c_void,
            text: *const c_void,
            option: *const c_void,
        ) {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            draw_rect_with(
                hooks,
                text,
                || hooks.rect_option.call(painter, rect, text, option),
                |replacement| {
                    hooks.rect_option.call(painter, rect, replacement, option);
                },
            );
        }
        pub(super) unsafe extern $abi fn draw_rect_coords_detour(
            painter: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flags: i32,
            text: *const c_void,
            bounding_rect: *mut c_void,
        ) {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            let Some(rect_coords) = hooks.rect_coords.as_ref() else {
                return;
            };
            draw_rect_with(
                hooks,
                text,
                || {
                    rect_coords.call(
                        painter,
                        x,
                        y,
                        width,
                        height,
                        flags,
                        text,
                        bounding_rect,
                    )
                },
                |replacement| {
                    rect_coords.call(
                        painter,
                        x,
                        y,
                        width,
                        height,
                        flags,
                        replacement,
                        bounding_rect,
                    );
                },
            );
        }
    };
}
#[cfg(target_arch = "x86")]
members!("thiscall");
#[cfg(not(target_arch = "x86"))]
members!("system");
