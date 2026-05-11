use windows::Win32::Foundation::HWND;
use crate::model::macro_step::*;

pub trait InputDispatcher: Send {
    fn attach(&mut self, window: HWND) -> bool;
    fn send_key(&self, step: &KeyStep) -> bool;
    fn send_text(&self, step: &TextStep) -> bool;
    fn move_mouse(&self, step: &MouseMoveStep) -> bool;
    fn click_mouse(&self, step: &MouseClickStep) -> bool;
    fn name(&self) -> &str;
}

// Helper structs for dispatcher methods - extract from MacroStep enum
pub struct KeyStep {
    pub virtual_key: u16,
    pub action: KeyAction,
}

pub struct TextStep {
    pub text: String,
}

pub struct MouseMoveStep {
    pub x: i32,
    pub y: i32,
    pub coordinate_mode: CoordinateMode,
}

pub struct MouseClickStep {
    pub button: MouseButton,
    pub action: MouseButtonAction,
}
