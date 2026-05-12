use tauri::State;
use parking_lot::Mutex;

use crate::model::app_config::AppConfig;
use crate::model::dispatch_mode::DispatchMode;
use crate::model::macro_sequence::MacroSequence;
use crate::model::target_spec::TargetSpec;
use crate::service::config_loader;
use crate::service::macro_executor;
use crate::service::macro_parser;
use crate::service::macro_repository::MacroRepository;
use crate::service::macro_serialization;
use crate::service::hotkey_daemon::HotkeyDaemon;
use crate::win32::window_finder;

/// Managed state for the application
pub struct AppState {
    pub repository: Mutex<Option<MacroRepository>>,
    pub daemon: HotkeyDaemon,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repository: Mutex::new(None),
            daemon: HotkeyDaemon::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Config management commands
// ---------------------------------------------------------------------------

/// Load configuration from an INI file (legacy support)
#[tauri::command]
pub fn load_config_from_file(path: String) -> Result<AppConfig, String> {
    config_loader::load_from_ini(&path).map_err(|e| e.to_string())
}

/// Apply an AppConfig directly (from frontend Pinia store)
#[tauri::command]
pub fn apply_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    // Update the daemon with the new config (triggers hotkey re-registration if running)
    state.daemon.update_config(config.clone());

    let repo = MacroRepository::new(config);
    *state.repository.lock() = Some(repo);
    Ok(())
}

/// Get the current loaded config
#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let repo_lock = state.repository.lock();
    let repo = repo_lock.as_ref().ok_or("Config not loaded")?;
    Ok(repo.config().clone())
}

// ---------------------------------------------------------------------------
// Window inspection
// ---------------------------------------------------------------------------

/// List all visible top-level windows
#[tauri::command]
pub fn list_windows() -> Vec<window_finder::WindowMatch> {
    window_finder::list_visible_top_level_windows()
}

/// Find windows matching a target spec
#[tauri::command]
pub fn find_windows(target: TargetSpec) -> Vec<window_finder::WindowMatch> {
    window_finder::find_all(&target)
}

// ---------------------------------------------------------------------------
// Macro execution
// ---------------------------------------------------------------------------

/// Play a named macro using the currently applied config
#[tauri::command]
pub fn play_macro(state: State<'_, AppState>, macro_name: String) -> Result<bool, String> {
    let repo_lock = state.repository.lock();
    let repo = repo_lock.as_ref().ok_or("Config not applied")?;
    let sequence = repo
        .find_macro(&macro_name)
        .ok_or(format!("Macro not found: {}", macro_name))?;
    macro_executor::execute(repo, sequence).map_err(|e| e.to_string())
}

/// Play a macro by directly providing the sequence and target spec.
/// No pre-loaded config needed — everything comes from the frontend.
#[tauri::command]
pub fn play_macro_direct(
    sequence: MacroSequence,
    target: TargetSpec,
) -> Result<bool, String> {
    // Build a minimal config with just the target
    let mut config = AppConfig::new();
    config
        .targets
        .insert(target.name.clone(), target);

    let repo = MacroRepository::new(config);
    macro_executor::execute(&repo, &sequence).map_err(|e| e.to_string())
}

/// Play a macro file with inline parameters
#[tauri::command]
pub fn play_macro_file(
    file_path: String,
    target_name: Option<String>,
    dispatch: Option<String>,
    target: Option<TargetSpec>,
) -> Result<bool, String> {
    let t_name = target_name.as_deref().unwrap_or("");
    let mut macro_seq =
        macro_parser::load_macro_file("adhoc", t_name, &file_path).map_err(|e| e.to_string())?;

    if let Some(d) = dispatch {
        macro_seq.dispatch_mode = match d.to_uppercase().as_str() {
            "WINDOW_MESSAGE" => DispatchMode::WindowMessage,
            "LOGITECH" => DispatchMode::Logitech,
            _ => DispatchMode::SendInput,
        };
    }

    let mut config = AppConfig::new();
    if let Some(t) = target {
        config.targets.insert(t.name.clone(), t);
    }

    let repo = MacroRepository::new(config);
    macro_executor::execute(&repo, &macro_seq).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Macro parsing / serialization
// ---------------------------------------------------------------------------

/// Parse a macro file and return the sequence
#[tauri::command]
pub fn parse_macro_file(file_path: String) -> Result<MacroSequence, String> {
    macro_parser::load_macro_file("parsed", "", &file_path).map_err(|e| e.to_string())
}

/// Serialize a macro sequence to text lines (for preview/export)
#[tauri::command]
pub fn serialize_macro_to_text(macro_seq: MacroSequence) -> Vec<String> {
    macro_serialization::serialize_macro(&macro_seq)
}

// ---------------------------------------------------------------------------
// Hotkey daemon commands
// ---------------------------------------------------------------------------

/// Start the hotkey daemon (registers hotkeys, begins listening)
#[tauri::command]
pub fn start_hotkey_daemon(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.daemon.start())
}

/// Stop the hotkey daemon
#[tauri::command]
pub fn stop_hotkey_daemon(state: State<'_, AppState>) -> Result<(), String> {
    state.daemon.stop();
    Ok(())
}

/// Check if the hotkey daemon is running
#[tauri::command]
pub fn is_hotkey_daemon_running(state: State<'_, AppState>) -> bool {
    state.daemon.is_running()
}
