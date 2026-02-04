use crate::colors::ColorPallet;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub show_arrows: bool,
    pub show_grid: bool,
    pub show_connection_count: bool,
    pub snap_to_grid: bool,
    pub color_pallet: ColorPallet,
}

impl Config {
    fn config_dir() -> PathBuf {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        home_dir.join(".smlogic")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }
    pub fn save(&self) {
        let dir = Self::config_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::config_path(), json);
        }
    }
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(json) = std::fs::read_to_string(&path) {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_arrows: false,
            show_grid: true,
            show_connection_count: true,
            snap_to_grid: true,
            color_pallet: ColorPallet::DEFAULT_PALLET,
        }
    }
}
