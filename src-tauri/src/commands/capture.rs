use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use serde::Serialize;
use std::thread;
use std::time::Duration;
use windows::core::w;
use windows::Win32::Foundation::RECT;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetWindowRect, GetWindowThreadProcessId, SetForegroundWindow,
};

#[derive(Serialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[tauri::command]
pub fn get_poe_window_rect() -> Result<WindowRect, String> {
    unsafe {
        let hwnd = FindWindowW(None, w!("Path of Exile"))
            .map_err(|_| "Path of Exile window not found".to_string())?;
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).map_err(|e| e.to_string())?;
        Ok(WindowRect {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        })
    }
}

#[tauri::command]
pub fn focus_poe_window() -> Result<String, String> {
    unsafe {
        let hwnd = FindWindowW(None, w!("Path of Exile"))
            .map_err(|_| "Path of Exile window not found".to_string())?;

        let current_tid = GetCurrentThreadId();
        let target_tid = GetWindowThreadProcessId(hwnd, None);
        tracing::debug!("focus_poe: cur_thread={current_tid}, target_thread={target_tid}");

        let attached = if current_tid != target_tid {
            let r = AttachThreadInput(current_tid, target_tid, true);
            tracing::debug!("AttachThreadInput(attach): {r:?}");
            r.as_bool()
        } else {
            false
        };

        let result = SetForegroundWindow(hwnd);
        tracing::debug!("SetForegroundWindow: {result:?}");

        if attached {
            let _ = AttachThreadInput(current_tid, target_tid, false);
        }

        if result.as_bool() {
            Ok("focus succeeded".into())
        } else {
            Ok("SetForegroundWindow returned false".into())
        }
    }
}

#[tauri::command]
pub fn capture_item_text() -> Result<String, String> {
    // Simulate Ctrl+C to copy hovered item text in PoE
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('c'), Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Release).map_err(|e| e.to_string())?;

    // Wait for PoE to populate the clipboard
    thread::sleep(Duration::from_millis(100));

    // Read clipboard contents
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = clipboard.get_text().map_err(|e| e.to_string())?;

    Ok(text)
}
