use tauri::State;
use crate::config::store::{ConfigStore, Settings};

#[tauri::command]
pub fn get_settings(store: State<ConfigStore>) -> Result<Settings, String> {
    let config = store.get_config();
    Ok(config.settings)
}

#[tauri::command]
pub fn update_settings(settings: Settings, store: State<ConfigStore>) -> Result<(), String> {
    store.update_settings(settings)
}
