use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeyAction {
    #[default]
    Tap,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MouseButtonAction {
    #[default]
    Click,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CoordinateMode {
    #[default]
    Screen,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroStep {
    Delay { milliseconds: u32 },
    Key { virtual_key: u16, action: KeyAction },
    MouseMove { x: i32, y: i32, coordinate_mode: CoordinateMode },
    MouseClick { button: MouseButton, action: MouseButtonAction },
    Text { text: String },
}
