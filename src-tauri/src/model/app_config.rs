use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::hotkey_binding::HotkeyBinding;
use super::macro_sequence::MacroSequence;
use super::target_spec::TargetSpec;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub targets: HashMap<String, TargetSpec>,
    pub macros: HashMap<String, MacroSequence>,
    pub hotkeys: Vec<HotkeyBinding>,
    pub recordings_directory: String,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            recordings_directory: "recordings".to_string(),
            ..Default::default()
        }
    }
}
