use std::collections::HashMap;
use std::fs;
use anyhow::{bail, Context, Result};
use crate::model::app_config::AppConfig;
use crate::model::dispatch_mode::DispatchMode;
use crate::model::hotkey_binding::{HotkeyActionType, TriggerMode};
use crate::model::target_spec::{TargetSpec, TitleMatchMode};
use crate::util::string_util::{parse_bool, to_upper_ascii, trim};
use super::macro_parser::{load_macro_file, parse_hotkey_binding, TriggerAction};

pub fn load_from_ini(ini_path: &str) -> Result<AppConfig> {
    let content = fs::read_to_string(ini_path)
        .with_context(|| format!("Failed to read config file: {}", ini_path))?;

    let sections = parse_ini(&content)?;

    let mut config = AppConfig {
        targets: HashMap::new(),
        macros: HashMap::new(),
        hotkeys: Vec::new(),
        recordings_directory: "recordings".to_string(),
    };

    // Process [general]
    if let Some(general) = sections.get("general") {
        if let Some(rec_dir) = general.get("recordings_dir") {
            config.recordings_directory = rec_dir.clone();
        }
    }

    // Process [target.*]
    for (section_name, entries) in &sections {
        if section_name.starts_with("target.") {
            let target_name = section_name.strip_prefix("target.").unwrap().to_string();
            let target_spec = parse_target_spec(&target_name, entries)?;
            config.targets.insert(target_name, target_spec);
        }
    }

    // Process [macro.*]
    for (section_name, entries) in &sections {
        if section_name.starts_with("macro.") {
            let macro_name = section_name.strip_prefix("macro.").unwrap().to_string();
            let macro_seq = parse_macro_spec(&macro_name, entries)?;
            config.macros.insert(macro_name, macro_seq);
        }
    }

    // Process [hotkey.*]
    let mut hotkey_id = 1;
    for (section_name, entries) in &sections {
        if section_name.starts_with("hotkey.") {
            let hotkey_name = section_name.strip_prefix("hotkey.").unwrap().to_string();
            if let Some(hotkey) = parse_hotkey_spec(&hotkey_name, entries, hotkey_id)? {
                config.hotkeys.push(hotkey);
                hotkey_id += 1;
            }
        }
    }

    Ok(config)
}

fn parse_ini(content: &str) -> Result<HashMap<String, HashMap<String, String>>> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section: Option<String> = None;

    for (line_num, raw_line) in content.lines().enumerate() {
        let line = trim(raw_line);

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].to_string();
            current_section = Some(section_name.clone());
            sections.entry(section_name).or_insert_with(HashMap::new);
            continue;
        }

        // Key=value pair
        if let Some(eq_pos) = line.find('=') {
            let key = trim(&line[..eq_pos]);
            let value = trim(&line[eq_pos + 1..]);

            if let Some(ref section) = current_section {
                sections
                    .get_mut(section)
                    .unwrap()
                    .insert(key.to_string(), value.to_string());
            } else {
                bail!("Line {}: Key-value pair outside of section", line_num + 1);
            }
        }
    }

    Ok(sections)
}

fn parse_target_spec(
    target_name: &str,
    entries: &HashMap<String, String>,
) -> Result<TargetSpec> {
    let mut spec = TargetSpec::new(target_name.to_string());

    if let Some(proc) = entries.get("process") {
        spec.process_name = proc.clone();
    }

    if let Some(class) = entries.get("class") {
        spec.window_class = class.clone();
    }

    if let Some(title) = entries.get("title") {
        spec.window_title = title.clone();
        spec.title_match_mode = TitleMatchMode::Exact;
    }

    if let Some(title_contains) = entries.get("title_contains") {
        spec.window_title = title_contains.clone();
        spec.title_match_mode = TitleMatchMode::Contains;
    }

    if let Some(top_level) = entries.get("top_level_only") {
        spec.top_level_only = parse_bool(top_level, true);
    }

    if let Some(visible) = entries.get("visible_only") {
        spec.visible_only = parse_bool(visible, true);
    }

    Ok(spec)
}

fn parse_macro_spec(
    macro_name: &str,
    entries: &HashMap<String, String>,
) -> Result<crate::model::macro_sequence::MacroSequence> {
    let file_path = entries
        .get("file")
        .context("Macro section missing 'file' key")?;

    let target_name = entries.get("target").map(|s| s.as_str()).unwrap_or("");

    let mut macro_seq = load_macro_file(macro_name, target_name, file_path)?;
    
    if let Some(dispatch) = entries.get("dispatch") {
        macro_seq.dispatch_mode = parse_dispatch_mode_value(dispatch);
    }
    
    Ok(macro_seq)
}

fn parse_hotkey_spec(
    hotkey_name: &str,
    entries: &HashMap<String, String>,
    id_seed: i32,
) -> Result<Option<crate::model::hotkey_binding::HotkeyBinding>> {
    let keys = entries.get("keys").context("Hotkey missing 'keys' key")?;

    let action_value = entries
        .get("action")
        .context("Hotkey missing 'action' key")?;

    let trigger_action = parse_trigger_action(action_value, entries);

    Ok(parse_hotkey_binding(
        hotkey_name,
        keys,
        &trigger_action,
        id_seed,
    ))
}

fn parse_trigger_action(
    action_value: &str,
    entries: &HashMap<String, String>,
) -> TriggerAction {
    let action_upper = to_upper_ascii(action_value);

    let action_type = match action_upper.as_str() {
        "PLAY" => HotkeyActionType::PlayMacro,
        "PLAY_FILE" => HotkeyActionType::PlayFile,
        "RECORD_TOGGLE" => HotkeyActionType::RecordToggle,
        _ => HotkeyActionType::PlayMacro,
    };

    let trigger_mode = if let Some(mode_str) = entries.get("trigger") {
        match to_upper_ascii(mode_str).as_str() {
            "TOGGLE" => TriggerMode::Toggle,
            "HOLD" => TriggerMode::Hold,
            "PHASED" => TriggerMode::Phased,
            _ => TriggerMode::Once,
        }
    } else {
        TriggerMode::Once
    };

    let repeat_delay_ms = entries
        .get("repeat_delay")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let macro_name = entries.get("macro").cloned().unwrap_or_default();
    let file_path = entries.get("file").cloned().unwrap_or_default();
    let target_name = entries.get("target").cloned().unwrap_or_default();

    let dispatch_mode = if let Some(dispatch_str) = entries.get("dispatch") {
        parse_dispatch_mode_value(dispatch_str)
    } else {
        DispatchMode::SendInput
    };

    let has_dispatch_override = entries.contains_key("dispatch");

    TriggerAction {
        action_type,
        trigger_mode,
        repeat_delay_ms,
        macro_name,
        file_path,
        target_name,
        dispatch_mode,
        has_dispatch_override,
    }
}

fn parse_dispatch_mode_value(value: &str) -> DispatchMode {
    match to_upper_ascii(trim(value)).as_str() {
        "WINDOW_MESSAGE" => DispatchMode::WindowMessage,
        "LOGITECH" => DispatchMode::Logitech,
        _ => DispatchMode::SendInput,
    }
}
