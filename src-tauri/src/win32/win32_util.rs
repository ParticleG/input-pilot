use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::Graphics::Gdi::ClientToScreen;

/// Get window title as a String.
pub fn get_window_text(hwnd: HWND) -> String {
    unsafe {
        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; (length + 1) as usize];
        let actual = GetWindowTextW(hwnd, &mut buffer);
        String::from_utf16_lossy(&buffer[..actual as usize])
    }
}

/// Get window class name as a String.
pub fn get_window_class(hwnd: HWND) -> String {
    unsafe {
        let mut buffer = [0u16; 256];
        let length = GetClassNameW(hwnd, &mut buffer);
        String::from_utf16_lossy(&buffer[..length as usize])
    }
}

/// Get process name for a window.
pub fn get_process_name(hwnd: HWND) -> String {
    unsafe {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return String::new();
        }
        let process = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        );
        let Ok(process) = process else {
            return String::new();
        };
        let mut buffer = [0u16; 260]; // MAX_PATH
        let length = GetModuleBaseNameW(process, None, &mut buffer);
        let _ = CloseHandle(process);
        if length == 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buffer[..length as usize])
    }
}

/// Format last Win32 error as a string.
pub fn format_last_error(prefix: &str) -> String {
    let error = unsafe { GetLastError() };
    format!("{} (error={})", prefix, error.0)
}

/// Convert client coordinates to screen coordinates for a window.
pub fn client_to_screen_point(hwnd: HWND, mut point: POINT) -> POINT {
    unsafe {
        let _ = ClientToScreen(hwnd, &mut point);
    }
    point
}

/// Pack coordinates into LPARAM for PostMessage.
pub fn make_client_lparam(x: i32, y: i32) -> LPARAM {
    LPARAM(((y as i16 as u16 as u32) << 16 | (x as i16 as u16 as u32)) as isize)
}
