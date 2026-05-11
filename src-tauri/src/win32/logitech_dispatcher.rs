// Logitech input dispatcher: sends input through the Logitech Gaming Software driver.
// Bypasses standard Win32 input injection, making it work with anti-cheat protected games.

use std::cell::UnsafeCell;

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use super::input_dispatcher::*;
use super::win32_util::*;
use crate::driver::logitech_driver::*;
use crate::model::macro_step::*;

pub struct LogitechDispatcher {
    target_window: HWND,
    // UnsafeCell because InputDispatcher trait methods take &self
    // but LogitechSender::send_keyboard_key needs &mut self for state tracking.
    // Safety: dispatcher is only used from a single thread at a time.
    sender: UnsafeCell<LogitechSender>,
    initialized: bool,
}

impl LogitechDispatcher {
    pub fn new() -> Self {
        Self {
            target_window: HWND::default(),
            sender: UnsafeCell::new(LogitechSender::new()),
            initialized: false,
        }
    }

    fn ensure_initialized(&mut self) -> bool {
        if self.initialized {
            return true;
        }
        let sender = self.sender.get_mut();
        match sender.initialize() {
            Ok(()) => {
                self.initialized = true;
                true
            }
            Err(e) => {
                log::error!("Logitech driver init failed: {}", e);
                false
            }
        }
    }
}

impl InputDispatcher for LogitechDispatcher {
    fn attach(&mut self, window: HWND) -> bool {
        self.target_window = window;
        if self.target_window.is_invalid() {
            log::warn!("LogitechDispatcher attach: target window is null");
            return false;
        }
        if !self.ensure_initialized() {
            return false;
        }

        unsafe {
            let _ = ShowWindow(self.target_window, SW_RESTORE);
            if !SetForegroundWindow(self.target_window).as_bool() {
                log::warn!("{}", format_last_error("LogitechDispatcher SetForegroundWindow failed"));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(120));
        true
    }

    fn send_key(&self, step: &KeyStep) -> bool {
        // SAFETY: dispatcher is used from a single macro execution thread
        let sender = unsafe { &mut *self.sender.get() };

        let vk = step.virtual_key;
        match step.action {
            KeyAction::Tap => {
                if !sender.send_keyboard_key(vk, true) {
                    return false;
                }
                sender.send_keyboard_key(vk, false)
            }
            KeyAction::Down => sender.send_keyboard_key(vk, true),
            KeyAction::Up => sender.send_keyboard_key(vk, false),
        }
    }

    fn send_text(&self, step: &TextStep) -> bool {
        // SAFETY: dispatcher is used from a single macro execution thread
        let sender = unsafe { &mut *self.sender.get() };
        for ch in step.text.chars() {
            if ch.is_ascii_alphanumeric() || ch == ' ' {
                let vk = if ch == ' ' { 0x20 } else { ch.to_ascii_uppercase() as u16 };
                if !sender.send_keyboard_key(vk, true) { return false; }
                if !sender.send_keyboard_key(vk, false) { return false; }
            }
            // Non-ASCII characters are silently skipped with Logitech driver
        }
        true
    }

    fn move_mouse(&self, step: &MouseMoveStep) -> bool {
        let mut x = step.x;
        let mut y = step.y;

        if step.coordinate_mode == CoordinateMode::Client && !self.target_window.is_invalid() {
            let pt = client_to_screen_point(self.target_window, POINT { x, y });
            x = pt.x;
            y = pt.y;
        }

        // Convert absolute screen coords to relative movement from current cursor
        let mut cursor = POINT::default();
        unsafe { let _ = GetCursorPos(&mut cursor); }
        let dx = x - cursor.x;
        let dy = y - cursor.y;

        let sender = unsafe { &*self.sender.get() };
        sender.send_mouse_move(dx, dy)
    }

    fn click_mouse(&self, step: &MouseClickStep) -> bool {
        let sender = unsafe { &*self.sender.get() };
        let button_bit = match step.button {
            MouseButton::Left => LG_MOUSE_LEFT,
            MouseButton::Right => LG_MOUSE_RIGHT,
            MouseButton::Middle => LG_MOUSE_MIDDLE,
        };

        match step.action {
            MouseButtonAction::Click => {
                if !sender.send_mouse_button(button_bit, true) {
                    return false;
                }
                sender.send_mouse_button(button_bit, false)
            }
            MouseButtonAction::Down => sender.send_mouse_button(button_bit, true),
            MouseButtonAction::Up => sender.send_mouse_button(button_bit, false),
        }
    }

    fn name(&self) -> &str {
        "IbInputLogitech"
    }
}

// SAFETY: LogitechDispatcher is only used from a single macro execution thread
unsafe impl Send for LogitechDispatcher {}
