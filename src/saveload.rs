use crate::AppState;
use crate::colors::ColorPallet;
use crate::parts::{Module, PartData};
use crate::state::CanvasSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
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
        if let Ok(json) = serde_json::to_value(self) {
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            std::fs::write(path, pretty).expect("failed to save json");
        }
    }

    pub fn load(
        path: PathBuf,
        project_path: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&contents)?;

        let mut canvas_snapshot: Self = serde_json::from_value(json)?;
        canvas_snapshot.reload_modules(project_path);
        Ok(canvas_snapshot)
    }

    pub fn reload_modules(&mut self, project_path: Option<PathBuf>) {
        for part in self.parts.values_mut() {
            if let PartData::Module(data) = &mut part.part_data {
                data.reload(project_path.clone());
            }
        }
    }
}

impl AppState {
    pub fn reload_project_folder(&mut self) {
        if let Some(project_folder) = &self.project_folder {
            // clear subfolder if you arent in one
            if let Some(sub_folder) = &self.project_sub_folder {
                if sub_folder.as_os_str().is_empty() {
                    self.project_sub_folder = None;
                };
            };
            // get the folder to read
            let folder_to_read = if let Some(sub_folder) = &self.project_sub_folder {
                project_folder.join(sub_folder)
            } else {
                project_folder.clone()
            };
            //load all entries
            let mut entries: Vec<PathBuf> = fs::read_dir(folder_to_read)
                .expect("failed to load folder")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect();
            // sort and apply all entries
            entries.sort_by_key(|p| !p.is_dir());
            self.current_folder_files = entries;
        }
    }
}
