/// Runs a deterministic standard-control UIA target over a line-oriented stdin contract.
///
/// Commands: `update` changes the public label/edit/document values; `recreate` replaces the
/// top-level window and controls; `foreground` makes the deterministic Point fixture topmost;
/// `block-provider` stalls the window thread until the stdin reader receives `unblock-provider`;
/// `exit` closes the window. A password edit is also changed so the observer contract can prove
/// that sensitive text never reaches capture output.
pub fn run_uia_standard_control_server(
    keepalive: Option<std::time::Duration>,
) -> std::io::Result<()> {
    use std::io::{BufRead, Write};
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, TryRecvError};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::System::LibraryLoader::LoadLibraryW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, GetWindowRect, PeekMessageW,
        SetWindowPos, SetWindowTextW, ShowWindow, TranslateMessage, ES_AUTOHSCROLL, ES_AUTOVSCROLL,
        ES_MULTILINE, ES_PASSWORD, HWND_TOPMOST, MSG, PM_REMOVE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_SHOWNOACTIVATE, WS_BORDER, WS_CHILD, WS_EX_TOOLWINDOW,
        WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe fn create(
        class_name: &str,
        text: &str,
        style: u32,
        bounds: (i32, i32, i32, i32),
        parent: windows_sys::Win32::Foundation::HWND,
    ) -> windows_sys::Win32::Foundation::HWND {
        let class_name = wide(class_name);
        let text = wide(text);
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            text.as_ptr(),
            style,
            bounds.0,
            bounds.1,
            bounds.2,
            bounds.3,
            parent,
            null_mut(),
            null_mut(),
            null(),
        )
    }

    #[derive(Clone, Copy)]
    struct Controls {
        window: windows_sys::Win32::Foundation::HWND,
        label: windows_sys::Win32::Foundation::HWND,
        value: windows_sys::Win32::Foundation::HWND,
        document: windows_sys::Win32::Foundation::HWND,
        range: windows_sys::Win32::Foundation::HWND,
        password: windows_sys::Win32::Foundation::HWND,
    }

    unsafe fn create_controls(recreated: bool) -> Option<Controls> {
        let rich_edit_module = wide("Msftedit.dll");
        if LoadLibraryW(rich_edit_module.as_ptr()).is_null() {
            return None;
        }
        let class_name = wide("Static");
        let title = wide(if recreated {
            "GlyphShift recreated UIA fixture"
        } else {
            "GlyphShift UIA fixture"
        });
        let window = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            40,
            40,
            520,
            360,
            null_mut(),
            null_mut(),
            null_mut(),
            null(),
        );
        if window.is_null() {
            return None;
        }
        let label = create(
            "Static",
            if recreated {
                "Recreated label"
            } else {
                "Fixture label"
            },
            WS_CHILD | WS_VISIBLE,
            (16, 16, 450, 24),
            window,
        );
        let value = create(
            "Edit",
            if recreated {
                "Recreated value"
            } else {
                "Fixture value"
            },
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32,
            (16, 52, 450, 28),
            window,
        );
        let document = create(
            "Edit",
            if recreated {
                "Recreated document"
            } else {
                "Fixture document"
            },
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_MULTILINE as u32 | ES_AUTOVSCROLL as u32,
            (16, 92, 450, 64),
            window,
        );
        let range = create(
            "RICHEDIT50W",
            if recreated {
                "Recreated first line\r\nRecreated second line"
            } else {
                "First line\r\nSecond line"
            },
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_MULTILINE as u32 | ES_AUTOVSCROLL as u32,
            (16, 168, 450, 72),
            window,
        );
        let password = create(
            "Edit",
            if recreated {
                "Recreated secret"
            } else {
                "Fixture secret"
            },
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_PASSWORD as u32,
            (16, 252, 450, 28),
            window,
        );
        if [label, value, document, range, password]
            .into_iter()
            .any(|handle| handle.is_null())
        {
            DestroyWindow(window);
            return None;
        }
        ShowWindow(window, SW_SHOWNOACTIVATE);
        Some(Controls {
            window,
            label,
            value,
            document,
            range,
            password,
        })
    }

    let mut controls = unsafe { create_controls(false) }
        .ok_or_else(|| std::io::Error::other("uia fixture control unavailable"))?;

    let provider_blocked = Arc::new(AtomicBool::new(false));
    let reader_provider_blocked = Arc::clone(&provider_blocked);
    let (commands, incoming) = mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if line.trim() == "unblock-provider" {
                reader_provider_blocked.store(false, Ordering::Release);
                continue;
            }
            if commands.send(line).is_err() {
                break;
            }
        }
    });
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "uia-ready")?;
    stdout.flush()?;
    let disconnected_keepalive = keepalive.filter(|value| {
        let milliseconds = value.as_millis();
        (1..=60_000).contains(&milliseconds)
    });
    let started = Instant::now();
    let mut running = true;
    while running {
        if disconnected_keepalive.is_some_and(|keepalive| started.elapsed() >= keepalive) {
            break;
        }
        let mut message = MSG::default();
        unsafe {
            while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        match incoming.try_recv() {
            Ok(command) if command.trim() == "update" => {
                unsafe {
                    SetWindowTextW(controls.label, wide("Updated label").as_ptr());
                    SetWindowTextW(controls.value, wide("Updated value").as_ptr());
                    SetWindowTextW(controls.document, wide("Updated document").as_ptr());
                    SetWindowTextW(
                        controls.range,
                        wide("Updated first line\r\nUpdated second line").as_ptr(),
                    );
                    SetWindowTextW(controls.password, wide("Updated secret").as_ptr());
                }
                writeln!(stdout, "uia-updated")?;
                stdout.flush()?;
            }
            Ok(command) if command.trim() == "recreate" => {
                unsafe {
                    DestroyWindow(controls.window);
                }
                controls = unsafe { create_controls(true) }
                    .ok_or_else(|| std::io::Error::other("recreated uia fixture unavailable"))?;
                writeln!(stdout, "uia-recreated")?;
                stdout.flush()?;
            }
            Ok(command) if command.trim() == "block-provider" => {
                provider_blocked.store(true, Ordering::Release);
                writeln!(stdout, "uia-provider-blocking")?;
                stdout.flush()?;
                while provider_blocked.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(10));
                }
                writeln!(stdout, "uia-provider-unblocked")?;
                stdout.flush()?;
            }
            Ok(command) if command.trim() == "foreground" => {
                let positioned = unsafe {
                    SetWindowPos(
                        controls.window,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                    )
                } != 0;
                if !positioned {
                    return Err(std::io::Error::other("uia fixture foreground unavailable"));
                }
                writeln!(stdout, "uia-foreground")?;
                stdout.flush()?;
            }
            Ok(command) if command.trim() == "geometry" => {
                let mut label = windows_sys::Win32::Foundation::RECT::default();
                let mut range = windows_sys::Win32::Foundation::RECT::default();
                let mut password = windows_sys::Win32::Foundation::RECT::default();
                let available = unsafe {
                    GetWindowRect(controls.label, &mut label) != 0
                        && GetWindowRect(controls.range, &mut range) != 0
                        && GetWindowRect(controls.password, &mut password) != 0
                };
                if !available {
                    return Err(std::io::Error::other("uia fixture geometry unavailable"));
                }
                writeln!(
                    stdout,
                    "uia-geometry {} {} {} {} {} {} {} {}",
                    (label.left + label.right) / 2,
                    (label.top + label.bottom) / 2,
                    range.left + 24,
                    range.top + 12,
                    range.left + 72,
                    range.top + 36,
                    (password.left + password.right) / 2,
                    (password.top + password.bottom) / 2,
                )?;
                stdout.flush()?;
            }
            Ok(command) if command.trim() == "exit" => {
                writeln!(stdout, "uia-exiting")?;
                stdout.flush()?;
                running = false;
            }
            Ok(_) | Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected)
                if disconnected_keepalive
                    .is_some_and(|keepalive| started.elapsed() < keepalive) => {}
            Err(TryRecvError::Disconnected) => running = false,
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        DestroyWindow(controls.window);
    }
    Ok(())
}
