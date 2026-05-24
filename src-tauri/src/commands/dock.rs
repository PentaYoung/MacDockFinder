use tauri::State;
use crate::config::store::{ConfigStore, PinnedItem};

fn seed_defaults() -> Vec<PinnedItem> {
    vec![
        PinnedItem { id: "default-explorer".into(), path: "explorer.exe".into(), label: "文件资源管理器".into(), icon_path: None },
        PinnedItem { id: "default-terminal".into(), path: "wt.exe".into(), label: "终端".into(), icon_path: None },
        PinnedItem { id: "default-edge".into(), path: "msedge.exe".into(), label: "Microsoft Edge".into(), icon_path: None },
        PinnedItem { id: "default-calculator".into(), path: "calculator.exe".into(), label: "计算器".into(), icon_path: None },
        PinnedItem { id: "default-notepad".into(), path: "notepad.exe".into(), label: "记事本".into(), icon_path: None },
    ]
}

#[tauri::command]
pub fn get_pinned_items(store: State<ConfigStore>) -> Result<Vec<PinnedItem>, String> {
    let config = store.get_config();
    eprintln!("[dock] config has {} pinned items", config.pinned_items.len());
    if config.pinned_items.is_empty() {
        let defaults = seed_defaults();
        eprintln!("[dock] returning {} defaults", defaults.len());
        return Ok(defaults);
    }
    eprintln!("[dock] returning stored items");
    Ok(config.pinned_items)
}

#[tauri::command]
pub fn add_pinned_item(path: String, store: State<ConfigStore>) -> Result<PinnedItem, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let label = std::path::Path::new(&path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let item = PinnedItem {
        id,
        path,
        label,
        icon_path: None,
    };
    store.add_pinned_item(item.clone())?;
    Ok(item)
}

#[tauri::command]
pub fn remove_pinned_item(id: String, store: State<ConfigStore>) -> Result<(), String> {
    store.remove_pinned_item(&id)
}

#[tauri::command]
pub fn reorder_items(ids: Vec<String>, store: State<ConfigStore>) -> Result<(), String> {
    store.reorder_items(ids)
}

#[tauri::command]
pub fn launch_app(path: String) -> Result<(), String> {
    std::process::Command::new(&path)
        .spawn()
        .map_err(|e| format!("failed to launch {}: {}", path, e))?;
    Ok(())
}
