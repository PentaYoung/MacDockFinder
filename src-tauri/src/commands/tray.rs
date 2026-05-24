#[tauri::command]
pub fn get_active_windows() -> Result<Vec<crate::monitor::process::WindowInfo>, String> {
    Ok(crate::monitor::process::enumerate_windows())
}
