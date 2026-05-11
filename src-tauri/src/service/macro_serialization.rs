use crate::model::macro_sequence::MacroSequence;
use crate::model::macro_step::{
    CoordinateMode, KeyAction, MacroStep, MouseButton, MouseButtonAction,
};

pub fn key_to_token(virtual_key: u16) -> String {
    // Named keys
    match virtual_key {
        0x0D => return "ENTER".to_string(),
        0x09 => return "TAB".to_string(),
        0x1B => return "ESC".to_string(),
        0x20 => return "SPACE".to_string(),
        0x25 => return "LEFT".to_string(),
        0x27 => return "RIGHT".to_string(),
        0x26 => return "UP".to_string(),
        0x28 => return "DOWN".to_string(),
        0x11 => return "CTRL".to_string(),
        0x12 => return "ALT".to_string(),
        0x10 => return "SHIFT".to_string(),
        0x5B => return "WIN".to_string(),
        0x2E => return "DELETE".to_string(),
        0x08 => return "BACKSPACE".to_string(),
        0x24 => return "HOME".to_string(),
        0x23 => return "END".to_string(),
        0x21 => return "PAGEUP".to_string(),
        0x22 => return "PAGEDOWN".to_string(),
        0x2D => return "INSERT".to_string(),
        0xA2 => return "LCTRL".to_string(),
        0xA3 => return "RCTRL".to_string(),
        0xA4 => return "LALT".to_string(),
        0xA5 => return "RALT".to_string(),
        0xA0 => return "LSHIFT".to_string(),
        0xA1 => return "RSHIFT".to_string(),
        0x5C => return "RWIN".to_string(),
        0x14 => return "CAPSLOCK".to_string(),
        0x90 => return "NUMLOCK".to_string(),
        0x91 => return "SCROLLLOCK".to_string(),
        0x2C => return "PRINTSCREEN".to_string(),
        0x13 => return "PAUSE".to_string(),
        _ => {}
    }

    // F1-F24
    if (0x70..=0x87).contains(&virtual_key) {
        let f_num = virtual_key - 0x70 + 1;
        return format!("F{}", f_num);
    }

    // A-Z
    if (0x41..=0x5A).contains(&virtual_key) {
        return (virtual_key as u8 as char).to_string();
    }

    // 0-9
    if (0x30..=0x39).contains(&virtual_key) {
        return (virtual_key as u8 as char).to_string();
    }

    // Fallback to VK_0xNN format
    format!("VK_0x{:02X}", virtual_key)
}

pub fn escape_macro_string(value: &str) -> String {
    let mut result = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }
    result
}

pub fn serialize_macro_step(step: &MacroStep) -> String {
    match step {
        MacroStep::Delay { milliseconds } => format!("DELAY {}", milliseconds),
        MacroStep::Text { text } => format!("TEXT \"{}\"", escape_macro_string(text)),
        MacroStep::Key {
            virtual_key,
            action,
        } => {
            let key_name = key_to_token(*virtual_key);
            let action_str = match action {
                KeyAction::Tap => "TAP",
                KeyAction::Down => "DOWN",
                KeyAction::Up => "UP",
            };
            format!("KEY {} {}", key_name, action_str)
        }
        MacroStep::MouseMove { x, y, coordinate_mode } => {
            let mode_str = match coordinate_mode {
                CoordinateMode::Client => "CLIENT",
                CoordinateMode::Screen => "SCREEN",
            };
            format!("MOUSE_MOVE {} {} {}", x, y, mode_str)
        }
        MacroStep::MouseClick { button, action } => {
            let button_str = match button {
                MouseButton::Left => "LEFT",
                MouseButton::Right => "RIGHT",
                MouseButton::Middle => "MIDDLE",
            };
            let action_str = match action {
                MouseButtonAction::Click => "CLICK",
                MouseButtonAction::Down => "DOWN",
                MouseButtonAction::Up => "UP",
            };
            format!("CLICK {} {}", button_str, action_str)
        }
    }
}

pub fn serialize_macro(macro_seq: &MacroSequence) -> Vec<String> {
    let mut lines = Vec::new();

    // Header comments
    lines.push(format!("# Macro: {}", macro_seq.name));
    if !macro_seq.target_name.is_empty() {
        lines.push(format!("TARGET {}", macro_seq.target_name));
    }

    let dispatch_str = match macro_seq.dispatch_mode {
        crate::model::dispatch_mode::DispatchMode::SendInput => "SEND_INPUT",
        crate::model::dispatch_mode::DispatchMode::WindowMessage => "WINDOW_MESSAGE",
        crate::model::dispatch_mode::DispatchMode::Logitech => "LOGITECH",
    };
    lines.push(format!("DISPATCH {}", dispatch_str));
    lines.push(String::new());

    if macro_seq.has_phases {
        if !macro_seq.on_press_steps.is_empty() {
            lines.push("PHASE ON_PRESS".to_string());
            for step in &macro_seq.on_press_steps {
                lines.push(serialize_macro_step(step));
            }
            lines.push(String::new());
        }

        if !macro_seq.on_hold_steps.is_empty() {
            lines.push("PHASE ON_HOLD".to_string());
            for step in &macro_seq.on_hold_steps {
                lines.push(serialize_macro_step(step));
            }
            lines.push(String::new());
        }

        if !macro_seq.on_release_steps.is_empty() {
            lines.push("PHASE ON_RELEASE".to_string());
            for step in &macro_seq.on_release_steps {
                lines.push(serialize_macro_step(step));
            }
            lines.push(String::new());
        }
    } else {
        for step in &macro_seq.steps {
            lines.push(serialize_macro_step(step));
        }
    }

    lines
}
