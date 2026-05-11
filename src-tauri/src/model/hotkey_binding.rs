use serde::{Deserialize, Serialize};
use super::dispatch_mode::DispatchMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HotkeyActionType {
    #[default]
    PlayMacro,
    RecordToggle,
    PlayFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TriggerMode {
    #[default]
    Once,
    Toggle,
    Hold,
    Phased,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub id: i32,
    pub modifiers: u32,
    pub virtual_key: u32,
    pub action: HotkeyActionType,
    pub trigger_mode: TriggerMode,
    pub repeat_delay_ms: i32,
    pub macro_name: String,
    pub file_path: String,
    pub target_name: String,
    pub dispatch_mode: DispatchMode,
    pub has_dispatch_override: bool,
    pub description: String,
}
