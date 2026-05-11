pub mod commands;
pub mod driver;
pub mod model;
pub mod service;
pub mod util;
pub mod win32;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::load_config_from_file,
            commands::apply_config,
            commands::get_config,
            commands::list_windows,
            commands::find_windows,
            commands::play_macro,
            commands::play_macro_direct,
            commands::play_macro_file,
            commands::parse_macro_file,
            commands::serialize_macro_to_text,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
