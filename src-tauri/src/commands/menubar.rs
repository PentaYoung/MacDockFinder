#[derive(serde::Serialize)]
pub struct VolumeInfo {
    pub level: f32,
    pub muted: bool,
}

#[derive(serde::Serialize)]
pub struct BatteryInfo {
    pub percent: u8,
    pub charging: bool,
}

#[derive(serde::Serialize)]
pub struct WifiInfo {
    pub ssid: String,
    pub signal: u8,
    pub connected: bool,
}

mod imp {
    pub fn get_volume() -> Result<(f32, bool), String> {
        Ok((0.75, false))
    }

    pub fn set_volume(_level: f32) -> Result<(), String> {
        Ok(())
    }

    pub fn get_battery() -> Result<(u8, bool), String> {
        Ok((100, true))
    }

    pub fn get_wifi() -> Result<(String, u8, bool), String> {
        Ok(("HomeWiFi".to_string(), 3, true))
    }
}

#[tauri::command]
pub fn get_volume() -> Result<VolumeInfo, String> {
    let (level, muted) = imp::get_volume()?;
    Ok(VolumeInfo { level, muted })
}

#[tauri::command]
pub fn set_volume(level: f32) -> Result<(), String> {
    imp::set_volume(level)
}

#[tauri::command]
pub fn get_battery() -> Result<BatteryInfo, String> {
    let (percent, charging) = imp::get_battery()?;
    Ok(BatteryInfo { percent, charging })
}

#[tauri::command]
pub fn get_wifi() -> Result<WifiInfo, String> {
    let (ssid, signal, connected) = imp::get_wifi()?;
    Ok(WifiInfo { ssid, signal, connected })
}
