use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedItem {
    pub id: String,
    pub path: String,
    pub label: String,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub position: String,
    pub auto_hide: bool,
    pub icon_size: u32,
    pub magnification: bool,
    pub minimize_to_tray: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            position: "bottom".to_string(),
            auto_hide: false,
            icon_size: 48,
            magnification: true,
            minimize_to_tray: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockConfig {
    pub settings: Settings,
    pub pinned_items: Vec<PinnedItem>,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            pinned_items: seed_defaults(),
        }
    }
}

pub struct ConfigStore {
    config: Mutex<DockConfig>,
    config_path: PathBuf,
}

fn seed_defaults() -> Vec<PinnedItem> {
    vec![
        PinnedItem {
            id: "default-explorer".into(),
            path: "explorer.exe".into(),
            label: "文件资源管理器".into(),
            icon_path: None,
        },
        PinnedItem {
            id: "default-terminal".into(),
            path: "wt.exe".into(),
            label: "终端".into(),
            icon_path: None,
        },
        PinnedItem {
            id: "default-edge".into(),
            path: "msedge.exe".into(),
            label: "Microsoft Edge".into(),
            icon_path: None,
        },
        PinnedItem {
            id: "default-calculator".into(),
            path: "calculator.exe".into(),
            label: "计算器".into(),
            icon_path: None,
        },
        PinnedItem {
            id: "default-notepad".into(),
            path: "notepad.exe".into(),
            label: "记事本".into(),
            icon_path: None,
        },
    ]
}

impl ConfigStore {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        eprintln!("[store] config path: {:?}", config_path);
        eprintln!("[store] config exists: {}", config_path.exists());
        let mut config = if config_path.exists() {
            let c = Self::load_from_file(&config_path);
            eprintln!("[store] loaded {} pinned items", c.pinned_items.len());
            c
        } else {
            eprintln!("[store] using default config");
            DockConfig::default()
        };
        if config.pinned_items.is_empty() {
            eprintln!("[store] seeding defaults");
            config.pinned_items = seed_defaults();
            Self::save_to_file(&config_path, &config).ok();
        }
        Self {
            config: Mutex::new(config),
            config_path,
        }
    }

    fn get_config_path() -> PathBuf {
        let mut path = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("macdockfinder");
        fs::create_dir_all(&path).ok();
        path.push("config.json");
        path
    }

    pub fn get_config(&self) -> DockConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn update_settings(&self, settings: Settings) -> Result<(), String> {
        let mut config = self.config.lock().unwrap();
        config.settings = settings;
        Self::save_to_file(&self.config_path, &config).map_err(|e| e.to_string())
    }

    pub fn add_pinned_item(&self, item: PinnedItem) -> Result<(), String> {
        let mut config = self.config.lock().unwrap();
        config.pinned_items.push(item);
        Self::save_to_file(&self.config_path, &config).map_err(|e| e.to_string())
    }

    pub fn remove_pinned_item(&self, id: &str) -> Result<(), String> {
        let mut config = self.config.lock().unwrap();
        config.pinned_items.retain(|i| i.id != id);
        Self::save_to_file(&self.config_path, &config).map_err(|e| e.to_string())
    }

    pub fn reorder_items(&self, ids: Vec<String>) -> Result<(), String> {
        let mut config = self.config.lock().unwrap();
        let mut items: Vec<PinnedItem> = ids
            .iter()
            .filter_map(|id| config.pinned_items.iter().find(|i| i.id == *id).cloned())
            .collect();
        let remaining: Vec<PinnedItem> = config
            .pinned_items
            .iter()
            .filter(|i| !ids.contains(&i.id))
            .cloned()
            .collect();
        items.extend(remaining);
        config.pinned_items = items;
        Self::save_to_file(&self.config_path, &config).map_err(|e| e.to_string())
    }

    fn load_from_file(path: &PathBuf) -> DockConfig {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn save_to_file(path: &PathBuf, config: &DockConfig) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content)
    }
}
