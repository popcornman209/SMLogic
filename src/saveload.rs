use crate::colors::ColorPallet;
use crate::parts::PartData;
use crate::state::CanvasSnapshot;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// config file

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub show_arrows: bool,
    pub show_grid: bool,
    pub show_connection_count: bool,
    pub snap_to_grid: bool,
    pub show_fps: bool,
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
            show_fps: false,
            color_pallet: ColorPallet::DEFAULT_PALLET,
        }
    }
}

// saving/loading modules
impl CanvasSnapshot {
    pub fn save(&self, path: PathBuf) {
        if let Ok(mut json) = serde_json::to_value(self) {
            if let Some(obj) = json.as_object_mut() {
                // Add your custom fields here
                obj.insert("version".to_string(), serde_json::json!(1));
                obj.insert("other_thing".to_string(), serde_json::json!("whatever"));

                // Clear module canvas_snapshots
                if let Some(parts) = obj.get_mut("parts").and_then(|p| p.as_object_mut()) {
                    for (_id, part) in parts.iter_mut() {
                        if let Some(module) = part
                            .get_mut("part_data")
                            .and_then(|pd| pd.get_mut("Module"))
                            .and_then(|m| m.as_object_mut())
                        {
                            module.insert("canvas_snapshot".to_string(), serde_json::json!({}));
                        }
                    }
                }
            }
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            std::fs::write(path, pretty).expect("failed to save json");
        }
    }

    pub fn load(path: &PathBuf) -> Self {
        let contents = std::fs::read_to_string(path).expect("failed to read file");
        let json: serde_json::Value =
            serde_json::from_str(&contents).expect("failed to parse json");

        // Check version if needed
        if let Some(version) = json.get("version").and_then(|v| v.as_i64()) {
            println!("Loading file version {}", version);
        }

        // Deserialize the rest into Self
        let mut canvas_snapshot: Self =
            serde_json::from_value(json).expect("failed to parse json obj");
        for part in canvas_snapshot.parts.values_mut() {
            if let PartData::Module(data) = &mut part.part_data {
                data.canvas_snapshot = Self::load(&data.path);
            }
        }
        canvas_snapshot
    }
}
