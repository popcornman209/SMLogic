use crate::{
    simulator::get_canvas_raw_data,
    state::{AppState, CanvasSnapshot},
};

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct ExporterSettings {
    maintain_io_position: bool,
    io_x_scale: f32, // if maintiain io position is true, scales down canvas positions to SM ones
    io_y_scale: f32,
    max_x: Option<usize>,
    max_y: Option<usize>,
    max_z: Option<usize>,
}

fn get_user_folder() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = PathBuf::from(std::env::var("APPDATA").ok());

    #[cfg(not(target_os = "windows"))]
    let base = dirs::home_dir()?.join(
        ".steam/steam/steamapps/compatdata/387990/pfx/drive_c/users/steamuser/Application Data",
    ); // untested :P

    let folder = base.join("Axolot Games/Scrap Mechanic/User/");
    folder.is_dir().then_some(folder)
}

pub fn get_bp_folder() -> Option<PathBuf> {
    let user_folder = get_user_folder()?;
    let users: Vec<_> = fs::read_dir(&user_folder)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    if users.len() == 1 {
        Some(users[0].path().join("Blueprints/"))
    } else {
        None // more than one user, cant auto-detect (not sure why there would be multiple anyway??)
    }
}

fn get_id_from_name(name: &str, folder: &Path) -> Option<(String, PathBuf, Value)> {
    for entry in fs::read_dir(folder).ok()? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if let Ok(contents) = fs::read_to_string(path.join("description.json")) {
            if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                if json["name"].as_str() == Some(name) {
                    return Some((entry.file_name().to_string_lossy().into_owned(), path, json));
                }
            }
        }
    }
    None
}

pub struct BluePrint {
    pub name: String,
    pub description: String,
    pub uuid: String,
    pub path: PathBuf,
}

impl BluePrint {
    pub fn from_name(name: &str, folder: &Path) -> Option<Self> {
        if let Some((uuid, path, json)) = get_id_from_name(name, folder) {
            let name = &json["name"].as_str().unwrap_or("err");
            let description = &json["description"].as_str().unwrap_or("err");
            return Some(Self {
                name: name.to_string(),
                description: description.to_string(),
                uuid: uuid,
                path: path,
            });
        }
        None
    }

    pub fn from_uuid(uuid: String, bp_folder: PathBuf) -> Option<Self> {
        let path = bp_folder.join(&uuid);
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                let name = &json["name"].as_str().unwrap_or("err");
                let description = &json["description"].as_str().unwrap_or("err");
                return Some(Self {
                    name: name.to_string(),
                    description: description.to_string(),
                    uuid: uuid,
                    path: path,
                });
            }
        }
        None
    }

    pub fn new(name: String, description: String, bp_folder: PathBuf) -> Self {
        let uuid = Uuid::new_v4().to_string();
        Self {
            name: name,
            description: description,
            uuid: uuid.clone(),
            path: bp_folder.join(uuid),
        }
    }

    pub fn export(self, canvas: CanvasSnapshot) {
        let (parts, colors, positions, connections, _id_remap, _tunnels, io_parts) =
            get_canvas_raw_data(canvas, true);
    }
}
