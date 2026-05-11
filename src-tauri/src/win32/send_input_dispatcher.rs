use super::input_dispatcher::*;
use super::win32_util::*;
use crate::model::macro_step::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::thread;
use std::time::Duration;

pub struct SendInputDispatcher {
    target_window: HWND,
}

// SAFETY: HWND is a window handle that can be safely sent between threads in Windows.
// Window handles are process-wide identifiers and Win32 API functions are thread-safe for HWND.
unsafe impl Send for SendInputDispatcher {}

impl SendInputDispatcher {
    pub fn new() -> Self {
        Self {
            target_window: HWND::default(),
        }
    }

    fn is_extended_key(vk: u16) -> bool {
        vk == VK_UP.0
            || vk == VK_DOWN.0
            || vk == VK_LEFT.0
            || vk == VK_RIGHT.0
            || vk == VK_HOME.0
            || vk == VK_END.0
            || vk == VK_PRIOR.0
            || vk == VK_NEXT.0
            || vk == VK_INSERT.0
            || vk == VK_DELETE.0
            || vk == VK_RCONTROL.0
            || vk == VK_RMENU.0
            || vk == VK_NUMLOCK.0
            || vk == VK_CANCEL.0
            || vk == VK_SNAPSHOT.0
            || vk == VK_DIVIDE.0
    }

    fn send_key_input(vk: u16, action: KeyAction) -> bool {
        unsafe {
            let scan_code = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16;

            let use_scancode = scan_code != 0;
            let is_extended = Self::is_extended_key(vk);

            let send_down = matches!(action, KeyAction::Tap | KeyAction::Down);
            let send_up = matches!(action, KeyAction::Tap | KeyAction::Up);

            let mut inputs = Vec::new();

            if send_down {
                let mut input = INPUT::default();
                input.r#type = INPUT_KEYBOARD;
                if use_scancode {
                    input.Anonymous.ki.wScan = scan_code;
                    input.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE;
                    if is_extended {
                        input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
                    }
                } else {
                    input.Anonymous.ki.wVk = VIRTUAL_KEY(vk);
                    input.Anonymous.ki.dwFlags = KEYBD_EVENT_FLAGS(0);
                }
                inputs.push(input);
            }

            if send_up {
                let mut input = INPUT::default();
                input.r#type = INPUT_KEYBOARD;
                if use_scancode {
                    input.Anonymous.ki.wScan = scan_code;
                    input.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                    if is_extended {
                        input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
                    }
                } else {
                    input.Anonymous.ki.wVk = VIRTUAL_KEY(vk);
                    input.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                }
                inputs.push(input);
            }

            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            sent == inputs.len() as u32
        }
    }

    fn send_text_input(text: &str) -> bool {
        unsafe {
            let utf16_chars: Vec<u16> = text.encode_utf16().collect();
            let mut inputs = Vec::new();

            for ch in utf16_chars {
                // Key down
                let mut input_down = INPUT::default();
                input_down.r#type = INPUT_KEYBOARD;
                input_down.Anonymous.ki.wScan = ch;
                input_down.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE;
                inputs.push(input_down);

                // Key up
                let mut input_up = INPUT::default();
                input_up.r#type = INPUT_KEYBOARD;
                input_up.Anonymous.ki.wScan = ch;
                input_up.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
                inputs.push(input_up);
            }

            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            sent == inputs.len() as u32
        }
    }

    fn send_mouse_move_input(x: i32, y: i32, coordinate_mode: CoordinateMode, hwnd: HWND) -> bool {
        unsafe {
            let (screen_x, screen_y) = match coordinate_mode {
                CoordinateMode::Client => {
                    let point = client_to_screen_point(hwnd, POINT { x, y });
                    (point.x, point.y)
                }
                CoordinateMode::Screen => (x, y),
            };

            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);

            if screen_width == 0 || screen_height == 0 {
                return false;
            }

            let normalized_x = (screen_x * 65535) / (screen_width - 1);
            let normalized_y = (screen_y * 65535) / (screen_height - 1);

            let mut input = INPUT::default();
            input.r#type = INPUT_MOUSE;
            input.Anonymous.mi.dx = normalized_x;
            input.Anonymous.mi.dy = normalized_y;
            input.Anonymous.mi.dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE;

            let sent = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            sent == 1
        }
    }

    fn send_mouse_click_input(button: MouseButton, action: MouseButtonAction) -> bool {
        unsafe {
            let (down_flag, up_flag) = match button {
                MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            };

            let send_down = matches!(action, MouseButtonAction::Click | MouseButtonAction::Down);
            let send_up = matches!(action, MouseButtonAction::Click | MouseButtonAction::Up);

            let mut inputs = Vec::new();

            if send_down {
                let mut input = INPUT::default();
                input.r#type = INPUT_MOUSE;
                input.Anonymous.mi.dwFlags = down_flag;
                inputs.push(input);
            }

            if send_up {
                let mut input = INPUT::default();
                input.r#type = INPUT_MOUSE;
                input.Anonymous.mi.dwFlags = up_flag;
                inputs.push(input);
            }

            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            sent == inputs.len() as u32
        }
    }
}

impl InputDispatcher for SendInputDispatcher {
    fn attach(&mut self, window: HWND) -> bool {
        unsafe {
            self.target_window = window;

            // Restore window if minimized
            let _ = ShowWindow(window, SW_RESTORE);

            // Bring to foreground
            if !SetForegroundWindow(window).as_bool() {
                return false;
            }

            // Wait for window to be ready
            thread::sleep(Duration::from_millis(120));

            true
        }
    }

    fn send_key(&self, step: &KeyStep) -> bool {
        Self::send_key_input(step.virtual_key, step.action)
    }

    fn send_text(&self, step: &TextStep) -> bool {
        Self::send_text_input(&step.text)
    }

    fn move_mouse(&self, step: &MouseMoveStep) -> bool {
        Self::send_mouse_move_input(step.x, step.y, step.coordinate_mode, self.target_window)
    }

    fn click_mouse(&self, step: &MouseClickStep) -> bool {
        Self::send_mouse_click_input(step.button, step.action)
    }

    fn name(&self) -> &str {
        "SendInput"
    }
}
