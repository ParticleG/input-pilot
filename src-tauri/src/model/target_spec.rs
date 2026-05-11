use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TitleMatchMode {
    #[default]
    Ignore,
    Exact,
    Contains,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetSpec {
    pub name: String,
    pub process_name: String,
    pub window_class: String,
    pub window_title: String,
    pub title_match_mode: TitleMatchMode,
    pub top_level_only: bool,
    pub visible_only: bool,
}

impl TargetSpec {
    pub fn new(name: String) -> Self {
        Self {
            name,
            top_level_only: true,
            visible_only: true,
            ..Default::default()
        }
    }
}
