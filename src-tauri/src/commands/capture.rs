use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use serde::Serialize;
use std::thread;
use std::time::Duration;
use windows::core::w;
use windows::Win32::Foundation::RECT;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CreateCompatibleBitmap, CreateCompatibleDC, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, GetDIBits, HDC, ReleaseDC, SelectObject,
};

// PrintWindow isn't always exposed through windows-rs' metadata-trimmed
// bindings; declare it directly. `PW_RENDERFULLCONTENT = 2` is the Win 8.1+
// flag designed for composited / DirectX windows. BOOL = i32 in the Win32 ABI.
#[link(name = "user32")]
unsafe extern "system" {
    fn PrintWindow(hwnd: HWND, hdc_blt: HDC, n_flags: u32) -> i32;
}
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetClientRect, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
    SetForegroundWindow,
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
pub fn is_poe_foreground() -> bool {
    unsafe {
        let Ok(poe_hwnd) = FindWindowW(None, w!("Path of Exile")) else {
            return false;
        };
        GetForegroundWindow() == poe_hwnd
    }
}

/// Result of the OCR-capture spike (Phase 6.6).
/// `non_black_fraction` is the fraction of pixels with any non-zero RGB —
/// near 0 means `PrintWindow` returned a black frame (the DX game refused
/// compositional capture) and we'd need to fall back to Windows.Graphics.Capture.
/// Near 1 means we have a real frame and can build OCR on top of this primitive.
#[derive(Serialize)]
pub struct CaptureTestResult {
    pub width: u32,
    pub height: u32,
    pub non_black_fraction: f64,
}

/// SPIKE (6.6): try to grab the PoE client area via `PrintWindow` with the
/// `PW_RENDERFULLCONTENT` (=2) flag, which is the Win 8.1+ flag designed for
/// composited / DirectX windows. Returns the dimensions and the fraction of
/// non-black pixels so the user can tell at a glance whether capture worked.
/// No file I/O, no new dependencies — keeps the spike cheap.
/// Capture the PoE client area as a top-down BGRA8 buffer via `PrintWindow`
/// (`PW_RENDERFULLCONTENT`). Shared by the capture spike and the OCR path.
fn capture_client_bgra() -> Result<(Vec<u8>, i32, i32), String> {
    unsafe {
        let hwnd = FindWindowW(None, w!("Path of Exile"))
            .map_err(|_| "Path of Exile window not found".to_string())?;

        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).map_err(|e| format!("GetClientRect: {e}"))?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err("PoE window has no client area (minimized?)".into());
        }

        let hdc_window = GetDC(Some(hwnd));
        if hdc_window.is_invalid() {
            return Err("GetDC failed".into());
        }
        let hdc_mem = CreateCompatibleDC(Some(hdc_window));
        let hbm = CreateCompatibleBitmap(hdc_window, width, height);
        let _old = SelectObject(hdc_mem, hbm.into());

        let ok = PrintWindow(hwnd, hdc_mem, 2) != 0;

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let pixel_count = (width as usize) * (height as usize);
        let mut pixels = vec![0u8; pixel_count * 4];
        let lines = GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup before any early return below.
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(Some(hwnd), hdc_window);

        if !ok || lines == 0 {
            return Err("PrintWindow / GetDIBits failed (likely DX-refused).".into());
        }

        Ok((pixels, width, height))
    }
}

#[tauri::command]
pub fn capture_poe_test() -> Result<CaptureTestResult, String> {
    let (pixels, width, height) = capture_client_bgra()?;
    let pixel_count = (width as usize) * (height as usize);
    let mut non_black: u64 = 0;
    for px in pixels.chunks_exact(4) {
        if px[0] > 0 || px[1] > 0 || px[2] > 0 {
            non_black += 1;
        }
    }
    let fraction = if pixel_count > 0 {
        non_black as f64 / pixel_count as f64
    } else {
        0.0
    };
    Ok(CaptureTestResult {
        width: width as u32,
        height: height as u32,
        non_black_fraction: fraction,
    })
}

/// Crop a sub-rectangle from a top-down BGRA buffer, clamped to bounds, forcing
/// alpha to 255 (PrintWindow leaves alpha 0, which OCR treats as transparent).
fn crop_bgra(
    src: &[u8],
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    cw: i32,
    ch: i32,
) -> Result<(Vec<u8>, i32, i32), String> {
    let x = x.clamp(0, w.max(0));
    let y = y.clamp(0, h.max(0));
    let cw = cw.min(w - x);
    let ch = ch.min(h - y);
    if cw <= 0 || ch <= 0 {
        return Err("OCR region is empty or out of bounds".into());
    }
    let stride = (w as usize) * 4;
    let mut out = Vec::with_capacity((cw as usize) * (ch as usize) * 4);
    for row in 0..ch as usize {
        let sy = y as usize + row;
        let start = sy * stride + (x as usize) * 4;
        for px in src[start..start + (cw as usize) * 4].chunks_exact(4) {
            out.extend_from_slice(&[px[0], px[1], px[2], 0xFF]);
        }
    }
    Ok((out, cw, ch))
}

/// Run Windows.Media.Ocr over a tight BGRA8 buffer; returns the recognized text.
fn ocr_bgra(pixels: &[u8], width: i32, height: i32) -> Result<String, String> {
    use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
    use windows::Media::Ocr::OcrEngine;
    use windows::Security::Cryptography::CryptographicBuffer;

    let buffer = CryptographicBuffer::CreateFromByteArray(pixels)
        .map_err(|e| format!("OCR buffer: {e}"))?;
    let bmp = SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, width, height)
        .map_err(|e| format!("OCR bitmap: {e}"))?;
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|e| format!("OCR engine (no language pack?): {e}"))?;
    let result = engine
        .RecognizeAsync(&bmp)
        .map_err(|e| format!("OCR recognize: {e}"))?
        .get()
        .map_err(|e| format!("OCR await: {e}"))?;
    Ok(result.Text().map_err(|e| format!("OCR text: {e}"))?.to_string())
}

/// Capture the PoE client, crop to a calibrated rectangle, and OCR it. Returns the
/// raw recognized text (used by the calibration "test" button).
#[tauri::command]
pub fn ocr_region(x: i32, y: i32, width: i32, height: i32) -> Result<String, String> {
    let (pixels, w, h) = capture_client_bgra()?;
    let (crop, cw, ch) = crop_bgra(&pixels, w, h, x, y, width, height)?;
    ocr_bgra(&crop, cw, ch)
}

/// Like `ocr_region` but parses the first integer out of the recognized text
/// (commas/spaces/thousands separators stripped) — for numeric resource panels.
pub fn ocr_region_int(x: i32, y: i32, width: i32, height: i32) -> Result<i64, String> {
    let text = ocr_region(x, y, width, height)?;
    extract_int(&text).ok_or_else(|| format!("no number found in OCR text: {text:?}"))
}

/// Pull the first integer from OCR text, ignoring thousands separators and stray
/// non-digits (e.g. "1,234 / 5,000" → 1234).
fn extract_int(text: &str) -> Option<i64> {
    let mut digits = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if c == ',' || c == '.' || c == ' ' {
            // thousands separator inside a number — keep scanning this run
            if !digits.is_empty() {
                continue;
            }
        } else if !digits.is_empty() {
            break; // end of the first numeric run
        }
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_int_handles_separators_and_noise() {
        assert_eq!(extract_int("1,234"), Some(1234));
        assert_eq!(extract_int("Gold: 12 500"), Some(12500));
        assert_eq!(extract_int("1,234 / 5,000"), Some(1234));
        assert_eq!(extract_int("no digits here"), None);
        assert_eq!(extract_int("99"), Some(99));
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
