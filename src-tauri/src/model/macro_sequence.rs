use serde::{Deserialize, Serialize};
use super::dispatch_mode::DispatchMode;
use super::macro_step::MacroStep;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacroSequence {
    pub name: String,
    pub target_name: String,
    pub dispatch_mode: DispatchMode,
    pub source_file: String,
    /// Flat steps (when no PHASE directives)
    pub steps: Vec<MacroStep>,
    /// Phased steps
    pub on_press_steps: Vec<MacroStep>,
    pub on_hold_steps: Vec<MacroStep>,
    pub on_release_steps: Vec<MacroStep>,
    pub has_phases: bool,
}
