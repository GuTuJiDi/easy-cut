// src-tauri/src/settings_manager.rs
use serde::{Deserialize, Serialize};
use std::fs;
use crate::config_manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    #[serde(rename = "autoSaveEnabled")]
    pub auto_save_enabled: bool,
    #[serde(rename = "trashRetentionDays")]
    pub trash_retention_days: u32,
    // 未来可在这里继续拓展：如 hardware_acceleration, theme 等
}

// 提供默认设置
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_save_enabled: true, // 默认强烈建议开启自动保存
            trash_retention_days: 30, // 默认保留 30 天
        }
    }
}

/// 读取全局设置
pub fn load_settings() -> AppSettings {
    let config_path = config_manager::get_config_dir().join("settings.json");

    if !config_path.exists() {
        let default_settings = AppSettings::default();
        let _ = save_settings_logic(&default_settings); // 初始化一份默认配置
        return default_settings;
    }

    match fs::read_to_string(&config_path) {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap_or_else(|_| AppSettings::default()),
        Err(_) => AppSettings::default(),
    }
}

/// 保存全局设置
pub fn save_settings_logic(settings: &AppSettings) -> Result<(), String> {
    let config_path = config_manager::get_config_dir().join("settings.json");
    let json_string = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("配置序列化失败: {}", e))?;

    fs::write(&config_path, json_string)
        .map_err(|e| format!("配置写入失败: {}", e))?;

    Ok(())
}