use std::fs;
use anyhow::{bail, Context, Result};
use crate::model::dispatch_mode::DispatchMode;
use crate::model::hotkey_binding::{HotkeyActionType, HotkeyBinding, TriggerMode};
use crate::model::macro_sequence::MacroSequence;
use crate::model::macro_step::{
    CoordinateMode, KeyAction, MacroStep, MouseButton, MouseButtonAction,
};
use crate::util::string_util::{to_upper_ascii, trim};

#[derive(Debug, Clone, Default)]
pub struct TriggerAction {
    pub action_type: HotkeyActionType,
    pub trigger_mode: TriggerMode,
    pub repeat_delay_ms: i32,
    pub macro_name: String,
    pub file_path: String,
    pub target_name: String,
    pub dispatch_mode: DispatchMode,
    pub has_dispatch_override: bool,
}

pub fn parse_virtual_key(token: &str) -> Option<u16> {
    let upper = to_upper_ascii(token);
    let trimmed = trim(&upper);

    // Named keys
    match trimmed {
        "ENTER" | "RETURN" => return Some(0x0D),
        "TAB" => return Some(0x09),
        "ESC" | "ESCAPE" => return Some(0x1B),
        "SPACE" => return Some(0x20),
        "LEFT" => return Some(0x25),
        "RIGHT" => return Some(0x27),
        "UP" => return Some(0x26),
        "DOWN" => return Some(0x28),
        "CTRL" => return Some(0x11),
        "ALT" => return Some(0x12),
        "SHIFT" => return Some(0x10),
        "WIN" => return Some(0x5B),
        "DELETE" | "DEL" => return Some(0x2E),
        "BACKSPACE" | "BACK" => return Some(0x08),
        "HOME" => return Some(0x24),
        "END" => return Some(0x23),
        "PAGEUP" | "PGUP" => return Some(0x21),
        "PAGEDOWN" | "PGDN" => return Some(0x22),
        "INSERT" | "INS" => return Some(0x2D),
        "LCTRL" => return Some(0xA2),
        "RCTRL" => return Some(0xA3),
        "LALT" => return Some(0xA4),
        "RALT" => return Some(0xA5),
        "LSHIFT" => return Some(0xA0),
        "RSHIFT" => return Some(0xA1),
        "LWIN" => return Some(0x5B),
        "RWIN" => return Some(0x5C),
        "CAPSLOCK" | "CAPS" => return Some(0x14),
        "NUMLOCK" => return Some(0x90),
        "SCROLLLOCK" | "SCROLL" => return Some(0x91),
        "PRINTSCREEN" | "PRINT" => return Some(0x2C),
        "PAUSE" => return Some(0x13),
        _ => {}
    }

    // F1-F24
    if trimmed.starts_with('F') {
        if let Ok(num) = trimmed[1..].parse::<u16>() {
            if (1..=24).contains(&num) {
                return Some(0x70 + num - 1);
            }
        }
    }

    // VK_0xNN hex format
    if trimmed.starts_with("VK_0X") {
        if let Ok(vk) = u16::from_str_radix(&trimmed[5..], 16) {
            return Some(vk);
        }
    }

    // Decimal VK code
    if let Ok(vk) = trimmed.parse::<u16>() {
        return Some(vk);
    }

    // Single character
    if trimmed.len() == 1 {
        let ch = trimmed.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            return Some(ch.to_ascii_uppercase() as u16);
        }
        if ch.is_ascii_digit() {
            return Some(ch as u16);
        }
        return Some(ch as u16);
    }

    None
}

pub fn parse_hotkey_binding(
    name: &str,
    keys_value: &str,
    action: &TriggerAction,
    id_seed: i32,
) -> Option<HotkeyBinding> {
    let parts: Vec<&str> = keys_value.split('+').map(|s| trim(s)).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = 0u32;
    let mut vk = 0u16;

    for part in &parts {
        let upper = to_upper_ascii(part);
        match upper.as_str() {
            "CTRL" | "CONTROL" => modifiers |= 0x0002, // MOD_CONTROL
            "ALT" => modifiers |= 0x0001,              // MOD_ALT
            "SHIFT" => modifiers |= 0x0004,            // MOD_SHIFT
            "WIN" | "WINDOWS" => modifiers |= 0x0008,  // MOD_WIN
            _ => {
                if let Some(parsed_vk) = parse_virtual_key(part) {
                    vk = parsed_vk;
                }
            }
        }
    }

    if vk == 0 {
        return None;
    }

    Some(HotkeyBinding {
        id: id_seed,
        description: name.to_string(),
        modifiers,
        virtual_key: vk as u32,
        action: action.action_type,
        trigger_mode: action.trigger_mode,
        repeat_delay_ms: action.repeat_delay_ms,
        macro_name: action.macro_name.clone(),
        file_path: action.file_path.clone(),
        target_name: action.target_name.clone(),
        dispatch_mode: action.dispatch_mode,
        has_dispatch_override: action.has_dispatch_override,
    })
}

pub fn load_macro_file(
    macro_name: &str,
    target_name: &str,
    file_path: &str,
) -> Result<MacroSequence> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read macro file: {}", file_path))?;

    let mut macro_seq = MacroSequence {
        name: macro_name.to_string(),
        target_name: target_name.to_string(),
        dispatch_mode: DispatchMode::SendInput,
        source_file: file_path.to_string(),
        steps: Vec::new(),
        on_press_steps: Vec::new(),
        on_hold_steps: Vec::new(),
        on_release_steps: Vec::new(),
        has_phases: false,
    };

    let mut current_phase: Option<String> = None;
    let mut line_num = 0;

    for raw_line in content.lines() {
        line_num += 1;
        let line = remove_inline_comment(raw_line);
        let trimmed = trim(&line);

        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = to_upper_ascii(parts[0]);

        match cmd.as_str() {
            "TARGET" => {
                if parts.len() >= 2 {
                    macro_seq.target_name = parts[1].to_string();
                }
            }
            "DISPATCH" => {
                if parts.len() >= 2 {
                    macro_seq.dispatch_mode = parse_dispatch_mode(parts[1]);
                }
            }
            "PHASE" => {
                if parts.len() >= 2 {
                    let phase = to_upper_ascii(parts[1]);
                    current_phase = Some(phase.clone());
                    macro_seq.has_phases = true;
                } else {
                    bail!("{}:{}: PHASE requires phase name", file_path, line_num);
                }
            }
            "DELAY" => {
                if parts.len() < 2 {
                    bail!("{}:{}: DELAY requires milliseconds", file_path, line_num);
                }
                let ms = parts[1]
                    .parse::<u32>()
                    .with_context(|| format!("{}:{}: Invalid delay value", file_path, line_num))?;
                let step = MacroStep::Delay { milliseconds: ms };
                add_step_to_phase(&mut macro_seq, &current_phase, step);
            }
            "TEXT" => {
                let text = extract_quoted_text(&line)
                    .with_context(|| format!("{}:{}: TEXT parsing failed", file_path, line_num))?;
                let step = MacroStep::Text { text };
                add_step_to_phase(&mut macro_seq, &current_phase, step);
            }
            "KEY" => {
                if parts.len() < 3 {
                    bail!("{}:{}: KEY requires <name> <action>", file_path, line_num);
                }
                let vk = parse_virtual_key(parts[1]).with_context(|| {
                    format!("{}:{}: Unknown key: {}", file_path, line_num, parts[1])
                })?;
                let action = parse_key_action(parts[2]);
                let step = MacroStep::Key {
                    virtual_key: vk,
                    action,
                };
                add_step_to_phase(&mut macro_seq, &current_phase, step);
            }
            "MOUSE_MOVE" => {
                if parts.len() < 4 {
                    bail!(
                        "{}:{}: MOUSE_MOVE requires <x> <y> <mode>",
                        file_path,
                        line_num
                    );
                }
                let x = parts[1].parse::<i32>().with_context(|| {
                    format!("{}:{}: Invalid x coordinate", file_path, line_num)
                })?;
                let y = parts[2].parse::<i32>().with_context(|| {
                    format!("{}:{}: Invalid y coordinate", file_path, line_num)
                })?;
                let mode = parse_coordinate_mode(parts[3]);
                let step = MacroStep::MouseMove { x, y, coordinate_mode: mode };
                add_step_to_phase(&mut macro_seq, &current_phase, step);
            }
            "CLICK" => {
                if parts.len() < 3 {
                    bail!("{}:{}: CLICK requires <button> <action>", file_path, line_num);
                }
                let button = parse_mouse_button(parts[1]);
                let action = parse_mouse_button_action(parts[2]);
                let step = MacroStep::MouseClick { button, action };
                add_step_to_phase(&mut macro_seq, &current_phase, step);
            }
            _ => {
                bail!("{}:{}: Unknown command: {}", file_path, line_num, cmd);
            }
        }
    }

    Ok(macro_seq)
}

fn remove_inline_comment(line: &str) -> String {
    let mut result = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    for ch in line.chars() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_quotes {
            result.push(ch);
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_quotes = !in_quotes;
            result.push(ch);
            continue;
        }

        if ch == '#' && !in_quotes {
            break;
        }

        result.push(ch);
    }

    result
}

fn extract_quoted_text(line: &str) -> Result<String> {
    let first_quote = line.find('"').context("No opening quote found")?;
    let last_quote = line.rfind('"').context("No closing quote found")?;

    if first_quote == last_quote {
        bail!("Only one quote found");
    }

    let quoted = &line[first_quote + 1..last_quote];
    let mut result = String::new();
    let mut chars = quoted.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => {
                        result.push('\\');
                        result.push(next_ch);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

fn parse_dispatch_mode(value: &str) -> DispatchMode {
    match to_upper_ascii(value).as_str() {
        "WINDOW_MESSAGE" => DispatchMode::WindowMessage,
        "LOGITECH" => DispatchMode::Logitech,
        _ => DispatchMode::SendInput,
    }
}

fn parse_key_action(value: &str) -> KeyAction {
    match to_upper_ascii(value).as_str() {
        "DOWN" => KeyAction::Down,
        "UP" => KeyAction::Up,
        _ => KeyAction::Tap,
    }
}

fn parse_coordinate_mode(value: &str) -> CoordinateMode {
    match to_upper_ascii(value).as_str() {
        "SCREEN" => CoordinateMode::Screen,
        _ => CoordinateMode::Client,
    }
}

fn parse_mouse_button(value: &str) -> MouseButton {
    match to_upper_ascii(value).as_str() {
        "RIGHT" => MouseButton::Right,
        "MIDDLE" => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

fn parse_mouse_button_action(value: &str) -> MouseButtonAction {
    match to_upper_ascii(value).as_str() {
        "DOWN" => MouseButtonAction::Down,
        "UP" => MouseButtonAction::Up,
        _ => MouseButtonAction::Click,
    }
}

fn add_step_to_phase(
    macro_seq: &mut MacroSequence,
    current_phase: &Option<String>,
    step: MacroStep,
) {
    if let Some(phase) = current_phase {
        match phase.as_str() {
            "ON_PRESS" => macro_seq.on_press_steps.push(step),
            "ON_HOLD" => macro_seq.on_hold_steps.push(step),
            "ON_RELEASE" => macro_seq.on_release_steps.push(step),
            _ => macro_seq.steps.push(step),
        }
    } else {
        macro_seq.steps.push(step);
    }
}
