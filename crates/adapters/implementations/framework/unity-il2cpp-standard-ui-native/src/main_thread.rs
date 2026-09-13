use std::time::Duration;

type DispatchCallback = fn() -> bool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WindowCandidate {
    thread_id: u32,
    window: usize,
}

fn single_window_thread(candidates: &[WindowCandidate]) -> Option<WindowCandidate> {
    let first = *candidates.first()?;
    if first.thread_id == 0 || first.window == 0 {
        return None;
    }
    candidates
        .iter()
        .all(|candidate| candidate.thread_id == first.thread_id)
        .then_some(first)
}

#[cfg(windows)]
mod windows_dispatch {
    use super::{single_window_thread, DispatchCallback, Duration, WindowCandidate};
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Condvar, Mutex, OnceLock};
    use std::time::Instant;
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, EnumWindows, GetWindowThreadProcessId, IsWindowVisible, PostMessageW,
        SetWindowsHookExW, UnhookWindowsHookEx, MSG, WH_GETMESSAGE, WM_NULL,
    };

    const DISPATCH_MESSAGE_WPARAM: usize = 0x4753_4955;

    struct PendingCallback {
        generation: usize,
        callback: DispatchCallback,
    }

    struct HookRegistration {
        generation: usize,
        thread_id: u32,
        handle: isize,
    }

    #[derive(Default)]
    struct Completion {
        generation: usize,
        result: Option<bool>,
    }

    static NEXT_GENERATION: AtomicUsize = AtomicUsize::new(0);
    static REQUEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static PENDING_CALLBACK: OnceLock<Mutex<Option<PendingCallback>>> = OnceLock::new();
    static HOOK: OnceLock<Mutex<Option<HookRegistration>>> = OnceLock::new();
    static COMPLETED: OnceLock<(Mutex<Completion>, Condvar)> = OnceLock::new();

    pub(super) fn dispatch(callback: DispatchCallback, timeout: Duration) -> bool {
        let Ok(_request) = REQUEST_LOCK.get_or_init(|| Mutex::new(())).lock() else {
            return false;
        };
        let Some(candidate) = discover_window_thread() else {
            return false;
        };
        if unsafe { GetCurrentThreadId() } == candidate.thread_id {
            return std::panic::catch_unwind(callback).unwrap_or(false);
        }

        let generation = next_generation();
        reset_completion(generation);
        if !install_hook(candidate.thread_id, generation) {
            return false;
        }
        let callbacks = PENDING_CALLBACK.get_or_init(|| Mutex::new(None));
        let Ok(mut pending) = callbacks.lock() else {
            remove_hook(generation);
            return false;
        };
        *pending = Some(PendingCallback {
            generation,
            callback,
        });
        drop(pending);

        let posted = unsafe {
            PostMessageW(
                candidate.window as HWND,
                WM_NULL,
                DISPATCH_MESSAGE_WPARAM,
                generation as isize,
            )
        } != 0;
        if !posted {
            take_callback(generation);
            remove_hook(generation);
            return false;
        }

        let result = wait_for_completion(generation, timeout);
        if result.is_none() {
            take_callback(generation);
            remove_hook(generation);
        }
        result.unwrap_or(false)
    }

    fn next_generation() -> usize {
        loop {
            let generation = NEXT_GENERATION
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1);
            if generation != 0 {
                return generation;
            }
        }
    }

    fn discover_window_thread() -> Option<WindowCandidate> {
        struct State {
            process_id: u32,
            candidates: Vec<WindowCandidate>,
        }

        unsafe extern "system" fn visit(window: HWND, state: LPARAM) -> i32 {
            let state = &mut *(state as *mut State);
            if IsWindowVisible(window) == 0 {
                return 1;
            }
            let mut process_id = 0;
            let thread_id = GetWindowThreadProcessId(window, &mut process_id);
            if process_id == state.process_id && thread_id != 0 {
                state.candidates.push(WindowCandidate {
                    thread_id,
                    window: window as usize,
                });
            }
            1
        }

        let mut state = State {
            process_id: unsafe { GetCurrentProcessId() },
            candidates: Vec::new(),
        };
        unsafe {
            EnumWindows(
                Some(visit),
                std::ptr::from_mut(&mut state).cast::<c_void>() as LPARAM,
            )
        };
        single_window_thread(&state.candidates)
    }

    fn install_hook(thread_id: u32, generation: usize) -> bool {
        if thread_id == 0 {
            return false;
        }
        let handle = unsafe {
            SetWindowsHookExW(
                WH_GETMESSAGE,
                Some(dispatch_hook),
                std::ptr::null_mut(),
                thread_id,
            )
        };
        if handle.is_null() {
            return false;
        }
        let hooks = HOOK.get_or_init(|| Mutex::new(None));
        let Ok(mut current) = hooks.lock() else {
            unsafe { UnhookWindowsHookEx(handle) };
            return false;
        };
        if let Some(previous) = current.take() {
            unsafe { UnhookWindowsHookEx(previous.handle as *mut c_void) };
        }
        *current = Some(HookRegistration {
            generation,
            thread_id,
            handle: handle as isize,
        });
        true
    }

    fn take_hook(generation: usize) -> Option<HookRegistration> {
        HOOK.get()
            .and_then(|hooks| hooks.lock().ok())
            .and_then(|mut hook| {
                hook.as_ref()
                    .is_some_and(|hook| hook.generation == generation)
                    .then(|| hook.take())
                    .flatten()
            })
    }

    fn remove_hook(generation: usize) {
        if let Some(hook) = take_hook(generation) {
            unsafe { UnhookWindowsHookEx(hook.handle as *mut c_void) };
        }
    }

    fn take_callback(generation: usize) -> Option<DispatchCallback> {
        PENDING_CALLBACK
            .get()
            .and_then(|callbacks| callbacks.lock().ok())
            .and_then(|mut pending| {
                pending
                    .as_ref()
                    .is_some_and(|pending| pending.generation == generation)
                    .then(|| pending.take().map(|pending| pending.callback))
                    .flatten()
            })
    }

    fn reset_completion(generation: usize) {
        let (completed, _) =
            COMPLETED.get_or_init(|| (Mutex::new(Completion::default()), Condvar::new()));
        if let Ok(mut completed) = completed.lock() {
            completed.generation = generation;
            completed.result = None;
        }
    }

    fn complete(generation: usize, result: bool) {
        let (completed, signal) =
            COMPLETED.get_or_init(|| (Mutex::new(Completion::default()), Condvar::new()));
        if let Ok(mut completed) = completed.lock() {
            if completed.generation == generation {
                completed.result = Some(result);
                signal.notify_all();
            }
        }
    }

    fn wait_for_completion(generation: usize, timeout: Duration) -> Option<bool> {
        let (completed, signal) =
            COMPLETED.get_or_init(|| (Mutex::new(Completion::default()), Condvar::new()));
        let Ok(mut state) = completed.lock() else {
            return None;
        };
        let started = Instant::now();
        loop {
            if state.generation != generation {
                return None;
            }
            if let Some(result) = state.result {
                return Some(result);
            }
            let remaining = timeout.checked_sub(started.elapsed())?;
            let Ok((next, wait)) = signal.wait_timeout(state, remaining) else {
                return None;
            };
            state = next;
            if wait.timed_out() && state.result.is_none() {
                return None;
            }
        }
    }

    unsafe extern "system" fn dispatch_hook(code: i32, wparam: usize, lparam: isize) -> isize {
        if code >= 0 && lparam != 0 {
            let message = &*(lparam as *const MSG);
            let generation = message.lParam as usize;
            let thread_id = GetCurrentThreadId();
            if message.message == WM_NULL && message.wParam == DISPATCH_MESSAGE_WPARAM {
                if let Some(hook) = take_hook(generation) {
                    UnhookWindowsHookEx(hook.handle as *mut c_void);
                    let right_thread = hook.thread_id == thread_id;
                    let result = take_callback(generation)
                        .filter(|_| right_thread)
                        .map(|callback| std::panic::catch_unwind(callback).unwrap_or(false))
                        .unwrap_or(false);
                    complete(generation, result);
                }
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::mpsc;
        use std::thread;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, PostThreadMessageW,
            TranslateMessage, CW_USEDEFAULT, WM_QUIT, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        };

        static TEST_LOCK: Mutex<()> = Mutex::new(());
        static CALLBACK_THREAD: AtomicU32 = AtomicU32::new(0);

        fn record_callback_thread() -> bool {
            CALLBACK_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Release);
            true
        }

        #[test]
        fn bounded_wait_times_out_without_a_matching_gui_callback() {
            let _serial = TEST_LOCK.lock().expect("dispatch test lock");
            let generation = usize::MAX - 1;
            reset_completion(generation);
            assert_eq!(
                wait_for_completion(generation, Duration::from_millis(1)),
                None
            );
        }

        #[test]
        fn dispatch_runs_callback_on_the_visible_window_thread() {
            let _serial = TEST_LOCK.lock().expect("dispatch test lock");
            let (ready_tx, ready_rx) = mpsc::channel();
            let window_thread = thread::spawn(move || unsafe {
                let class_name = "STATIC\0".encode_utf16().collect::<Vec<_>>();
                let title = "Glyphshift IL2CPP dispatch fixture\0"
                    .encode_utf16()
                    .collect::<Vec<_>>();
                let window = CreateWindowExW(
                    0,
                    class_name.as_ptr(),
                    title.as_ptr(),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    320,
                    180,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                );
                let thread_id = GetCurrentThreadId();
                ready_tx
                    .send((window as usize, thread_id))
                    .expect("send fixture window");
                if window.is_null() {
                    return;
                }
                let mut message = std::mem::zeroed::<MSG>();
                while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                DestroyWindow(window);
            });
            let (window, thread_id) = ready_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("fixture window ready");

            CALLBACK_THREAD.store(0, Ordering::Release);
            let dispatched =
                window != 0 && dispatch(record_callback_thread, Duration::from_secs(2));
            let callback_thread = CALLBACK_THREAD.load(Ordering::Acquire);
            unsafe { PostThreadMessageW(thread_id, WM_QUIT, 0, 0) };
            window_thread.join().expect("fixture window thread");

            assert_ne!(window, 0, "fixture window must be created");
            assert!(dispatched, "dispatch must complete through WH_GETMESSAGE");
            assert_eq!(callback_thread, thread_id);
        }
    }
}

pub(crate) fn dispatch(callback: DispatchCallback, timeout: Duration) -> bool {
    #[cfg(windows)]
    {
        windows_dispatch::dispatch(callback, timeout)
    }
    #[cfg(not(windows))]
    {
        let _ = (callback, timeout);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_many_visible_windows_only_when_they_share_one_thread() {
        let selected = single_window_thread(&[
            WindowCandidate {
                thread_id: 7,
                window: 100,
            },
            WindowCandidate {
                thread_id: 7,
                window: 101,
            },
        ])
        .expect("same GUI thread");
        assert_eq!(selected.thread_id, 7);
        assert_eq!(selected.window, 100);
    }

    #[test]
    fn rejects_zero_or_ambiguous_window_threads() {
        assert!(single_window_thread(&[]).is_none());
        assert!(single_window_thread(&[WindowCandidate {
            thread_id: 0,
            window: 100,
        }])
        .is_none());
        assert!(single_window_thread(&[
            WindowCandidate {
                thread_id: 7,
                window: 100,
            },
            WindowCandidate {
                thread_id: 8,
                window: 101,
            },
        ])
        .is_none());
    }
}
