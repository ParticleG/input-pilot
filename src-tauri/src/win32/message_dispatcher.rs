use super::input_dispatcher::*;
use super::win32_util::*;
use crate::model::macro_step::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::ScreenToClient;
use std::cell::Cell;

pub struct MessageDispatcher {
    target_window: HWND,
    last_client_point: Cell<POINT>,
}

// SAFETY: HWND is a window handle that can be safely sent between threads in Windows.
// Window handles are process-wide identifiers and Win32 API functions are thread-safe for HWND.
unsafe impl Send for MessageDispatcher {}

impl MessageDispatcher {
    pub fn new() -> Self {
        Self {
            target_window: HWND::default(),
            last_client_point: Cell::new(POINT { x: 0, y: 0 }),
        }
    }

    fn send_key_message(hwnd: HWND, vk: u16, action: KeyAction) -> bool {
        unsafe {
            let send_down = matches!(action, KeyAction::Tap | KeyAction::Down);
            let send_up = matches!(action, KeyAction::Tap | KeyAction::Up);

            if send_down {
                let lparam = LPARAM(1); // Repeat count = 1
                if PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(vk as usize), lparam).is_err() {
                    return false;
                }
            }

            if send_up {
                // lparam for keyup: bit 30 (previous state) and bit 31 (transition state) set
                let lparam = LPARAM(((1 << 30) | (1 << 31) | 1) as isize);
                if PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(vk as usize), lparam).is_err() {
                    return false;
                }
            }

            true
        }
    }

    fn send_text_message(hwnd: HWND, text: &str) -> bool {
        unsafe {
            let utf16_chars: Vec<u16> = text.encode_utf16().collect();

            for ch in utf16_chars {
                let lparam = LPARAM(1); // Repeat count = 1
                if PostMessageW(Some(hwnd), WM_CHAR, WPARAM(ch as usize), lparam).is_err() {
                    return false;
                }
            }

            true
        }
    }

    fn send_mouse_move_message(
        &self,
        x: i32,
        y: i32,
        coordinate_mode: CoordinateMode,
    ) -> bool {
        unsafe {
            let (client_x, client_y) = match coordinate_mode {
                CoordinateMode::Screen => {
                    let mut point = POINT { x, y };
                    let _ = ScreenToClient(self.target_window, &mut point);
                    (point.x, point.y)
                }
                CoordinateMode::Client => (x, y),
            };

            self.last_client_point.set(POINT {
                x: client_x,
                y: client_y,
            });

            let lparam = make_client_lparam(client_x, client_y);
            PostMessageW(Some(self.target_window), WM_MOUSEMOVE, WPARAM(0), lparam).is_ok()
        }
    }

    fn send_mouse_click_message(&self, button: MouseButton, action: MouseButtonAction) -> bool {
        unsafe {
            let (down_msg, up_msg) = match button {
                MouseButton::Left => (WM_LBUTTONDOWN, WM_LBUTTONUP),
                MouseButton::Right => (WM_RBUTTONDOWN, WM_RBUTTONUP),
                MouseButton::Middle => (WM_MBUTTONDOWN, WM_MBUTTONUP),
            };

            let send_down = matches!(action, MouseButtonAction::Click | MouseButtonAction::Down);
            let send_up = matches!(action, MouseButtonAction::Click | MouseButtonAction::Up);

            let last_point = self.last_client_point.get();
            let lparam = make_client_lparam(last_point.x, last_point.y);

            if send_down {
                if PostMessageW(Some(self.target_window), down_msg, WPARAM(0), lparam).is_err() {
                    return false;
                }
            }

            if send_up {
                if PostMessageW(Some(self.target_window), up_msg, WPARAM(0), lparam).is_err() {
                    return false;
                }
            }

            true
        }
    }
}

impl InputDispatcher for MessageDispatcher {
    fn attach(&mut self, window: HWND) -> bool {
        self.target_window = window;
        true
    }

    fn send_key(&self, step: &KeyStep) -> bool {
        Self::send_key_message(self.target_window, step.virtual_key, step.action)
    }

    fn send_text(&self, step: &TextStep) -> bool {
        Self::send_text_message(self.target_window, &step.text)
    }

    fn move_mouse(&self, step: &MouseMoveStep) -> bool {
        self.send_mouse_move_message(step.x, step.y, step.coordinate_mode)
    }

    fn click_mouse(&self, step: &MouseClickStep) -> bool {
        self.send_mouse_click_message(step.button, step.action)
    }

    fn name(&self) -> &str {
        "WindowMessage"
    }
}
