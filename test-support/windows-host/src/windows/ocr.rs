/// Runs a deterministic top-level window for Windows Graphics Capture boundary tests.
///
/// The window can be placed partly outside the virtual desktop and can opt into Windows capture
/// exclusion. It remains alive until stdin receives `exit`, then destroys the HWND before return.
pub fn run_ocr_capture_server(protected: bool, negative_origin: bool) -> std::io::Result<()> {
    use std::io::{BufRead, Write};
    use std::ptr::{null, null_mut};
    use std::sync::mpsc::{self, TryRecvError};
    use std::time::Duration;
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::UpdateWindow;
    use windows_sys::Win32::UI::HiDpi::{
        SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, GetWindowRect, PeekMessageW,
        SetWindowDisplayAffinity, ShowWindow, TranslateMessage, MSG, PM_REMOVE, SW_SHOWNOACTIVATE,
        WDA_EXCLUDEFROMCAPTURE, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let (left, top) = if negative_origin {
        (-96, -64)
    } else {
        (96, 96)
    };
    let window = unsafe {
        CreateWindowExW(
            0,
            wide("Static").as_ptr(),
            wide("GlyphShift OCR boundary fixture ABC 123").as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            left,
            top,
            480,
            240,
            null_mut(),
            null_mut(),
            null_mut(),
            null(),
        )
    };
    if window.is_null() {
        return Err(std::io::Error::other("OCR fixture window unavailable"));
    }
    if protected && unsafe { SetWindowDisplayAffinity(window, WDA_EXCLUDEFROMCAPTURE) } == 0 {
        unsafe {
            DestroyWindow(window);
        }
        return Err(std::io::Error::other(
            "OCR fixture capture exclusion unavailable",
        ));
    }
    unsafe {
        ShowWindow(window, SW_SHOWNOACTIVATE);
        UpdateWindow(window);
    }
    let mut bounds = RECT::default();
    if unsafe { GetWindowRect(window, &mut bounds) } == 0 {
        unsafe {
            DestroyWindow(window);
        }
        return Err(std::io::Error::other("OCR fixture bounds unavailable"));
    }

    let (commands, incoming) = mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if commands.send(line).is_err() {
                break;
            }
        }
    });
    let mut stdout = std::io::stdout().lock();
    writeln!(
        stdout,
        "ocr-ready {} {} {} {}",
        bounds.left, bounds.top, bounds.right, bounds.bottom
    )?;
    stdout.flush()?;

    let mut running = true;
    while running {
        let mut message = MSG::default();
        unsafe {
            while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        match incoming.try_recv() {
            Ok(command) if command.trim() == "exit" => {
                writeln!(stdout, "ocr-exiting")?;
                stdout.flush()?;
                running = false;
            }
            Ok(_) | Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => running = false,
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        DestroyWindow(window);
    }
    Ok(())
}
