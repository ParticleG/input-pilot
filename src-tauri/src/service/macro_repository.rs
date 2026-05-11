use crate::model::app_config::AppConfig;
use crate::model::hotkey_binding::HotkeyBinding;
use crate::model::macro_sequence::MacroSequence;
use crate::model::target_spec::TargetSpec;

pub struct MacroRepository {
    config: AppConfig,
    last_recorded_macro: Option<MacroSequence>,
}

impl MacroRepository {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            last_recorded_macro: None,
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn find_macro(&self, name: &str) -> Option<&MacroSequence> {
        self.config.macros.get(name)
    }

    pub fn find_target(&self, name: &str) -> Option<&TargetSpec> {
        self.config.targets.get(name)
    }

    pub fn find_hotkey_by_id(&self, id: i32) -> Option<&HotkeyBinding> {
        self.config.hotkeys.iter().find(|h| h.id == id)
    }

    pub fn set_last_recorded_macro(&mut self, macro_seq: MacroSequence) {
        self.last_recorded_macro = Some(macro_seq);
    }

    pub fn last_recorded_macro(&self) -> Option<&MacroSequence> {
        self.last_recorded_macro.as_ref()
    }
}
