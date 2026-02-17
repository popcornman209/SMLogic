use crate::AppState;
use crate::colors::ColorPallet;
use crate::connections::Connection;
use crate::parts::{Part, PartData, Port};
use crate::state::{CanvasSnapshot, Selection};
use egui::Pos2;
use egui_notify::Toasts;
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
    pub last_project: Option<PathBuf>,
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
            last_project: None,
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
        toasts: &mut Toasts,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&contents)?;

        let mut canvas_snapshot: Self = serde_json::from_value(json)?;
        canvas_snapshot.reload_modules(project_path, toasts);
        Ok(canvas_snapshot)
    }

    pub fn reload_modules(&mut self, project_path: Option<PathBuf>, toasts: &mut Toasts) {
        for part in self.parts.values_mut() {
            if let PartData::Module(data) = &mut part.part_data {
                data.reload(project_path.clone(), toasts);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClipboardData {
    parts: Vec<Part>,
    connections: Vec<Connection>,
    mouse_pos: Pos2,
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

    pub fn to_clipboard(&mut self, world_pos: Pos2) {
        let mut output = ClipboardData {
            parts: Vec::new(),
            connections: Vec::new(),
            mouse_pos: world_pos,
        };
        for selection in &self.selection {
            match selection {
                Selection::Part(part_id) => {
                    if let Some(part) = self.canvas_snapshot.parts.get(part_id) {
                        output.parts.push(part.clone());
                    }
                }
                Selection::Connection(conn_id) => {
                    if let Some(conn) = self.canvas_snapshot.connections.get(conn_id.clone()) {
                        output.connections.push(conn.clone());
                    }
                }
            }
        }
        self.clipboard_data = Some(output);
    }
    pub fn load_clipboard(&mut self, world_pos: Pos2) {
        {
            if let Some(value) = self.clipboard_data.clone() {
                if value.parts.len() > 0 {
                    self.selection.clear();
                    self.push_undo();
                    let mut id_remap: HashMap<u64, u64> = HashMap::new();
                    for part in value.parts {
                        let id = self.canvas_snapshot.next_id.clone();
                        self.canvas_snapshot.next_id += 1;
                        id_remap.insert(part.id, id);
                        let mut new_part = part.clone();
                        new_part.id = id;
                        new_part.pos += world_pos - value.mouse_pos;
                        if self.snap_to_grid {
                            new_part.snap_pos();
                        }
                        self.canvas_snapshot.parts.insert(id, new_part);
                        self.selection.push(Selection::Part(id));
                    }

                    for connection in value.connections {
                        if let Some(new_start) = id_remap.get(&connection.start.part) {
                            if let Some(new_end) = id_remap.get(&connection.end.part) {
                                self.canvas_snapshot.connections.push(Connection {
                                    start: Port {
                                        part: new_start.clone(),
                                        port_id: connection.start.port_id,
                                        input: connection.start.input,
                                    },
                                    end: Port {
                                        part: new_end.clone(),
                                        port_id: connection.end.port_id,
                                        input: connection.end.input,
                                    },
                                    powered: connection.powered,
                                });
                            }
                        }
                    }
                    self.reload_connection_counts();
                }
            }
        }
    }
}
