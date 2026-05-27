use crate::{
    simulator::{PartType, get_canvas_raw_data},
    state::{AppState, CanvasSnapshot},
};

use egui::Pos2;
use egui_notify::Toasts;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DEFAULT_ICON: &[u8] = include_bytes!("../assets/default_icon.png");

#[derive(Clone, PartialEq)]
pub enum ExportType {
    FromName,
    FromUUID,
    New,
}
impl ExportType {
    pub const TYPES: &[Self] = &[Self::FromName, Self::FromUUID, Self::New];

    pub fn to_label(&self) -> &'static str {
        match self {
            Self::FromName => "Overwrite by name",
            Self::FromUUID => "Overwrite by UUID",
            Self::New => "New blueprint",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ExporterSettings {
    pub maintain_io_position: bool,
    pub io_x_scale: f32, // if maintiain io position is true, scales down canvas positions to SM ones
    pub io_y_scale: f32,
    pub max_x: Option<usize>,
    pub max_y: Option<usize>,
    pub max_z: Option<usize>,
    pub export_type: ExportType,
    pub identifier: Option<String>,
    pub new_name: Option<String>,
    pub new_desc: Option<String>,
    pub new_icon: Option<std::path::PathBuf>,
}

impl AppState {
    pub fn export(&mut self, exporter_settings: ExporterSettings) {
        if (exporter_settings.max_x.unwrap_or(1) <= 0)
            | (exporter_settings.max_y.unwrap_or(1) <= 0)
            | (exporter_settings.max_x.unwrap_or(1) <= 0)
        {
            self.toasts.error("cant set max size <= 0!");
            return;
        }
        if let Some(bp_folder) = self.bp_folder.clone() {
            let mut blueprint: Option<BluePrint> = match exporter_settings.export_type {
                ExportType::FromName => {
                    if let Some(ref name) = exporter_settings.identifier {
                        if let Some(bp) = BluePrint::from_name(&name, &bp_folder) {
                            Some(bp)
                        } else {
                            self.toasts.error("couldn't find blueprint!");
                            None
                        }
                    } else {
                        self.toasts.error("no name supplied?");
                        None
                    }
                }
                ExportType::FromUUID => {
                    if let Some(ref uuid) = exporter_settings.identifier {
                        if let Some(bp) = BluePrint::from_uuid(uuid.clone(), bp_folder) {
                            Some(bp)
                        } else {
                            self.toasts.error("couldn't find blueprint!");
                            None
                        }
                    } else {
                        self.toasts.error("no uuid supplied?");
                        None
                    }
                }
                ExportType::New => {
                    if let Some(ref name) = exporter_settings.new_name {
                        if name.is_empty() {
                            self.toasts.error("new blueprints need a name!");
                            None
                        } else {
                            let icon = if let Some(ref icon_path) = exporter_settings.new_icon {
                                if let Ok(icon_data) = fs::read(icon_path) {
                                    if !is_128x128_png(&icon_data) {
                                        self.toasts.error("icon must be a 128x128 png!");
                                        return;
                                    }
                                    Some(icon_data)
                                } else {
                                    self.toasts.error("failed to read supplied icon!");
                                    return;
                                }
                            } else {
                                None
                            };
                            Some(BluePrint::new(
                                name.clone(),
                                exporter_settings.new_desc.clone().unwrap_or(String::new()),
                                bp_folder,
                                icon,
                            ))
                        }
                    } else {
                        self.toasts.error("new blueprints need a name!");
                        None
                    }
                }
            };

            if let Some(ref mut bp) = blueprint {
                if bp.description.is_empty() {
                    bp.description = "#{STEAM_WORKSHOP_NO_DESCRIPTION}".to_string();
                }
                if matches!(
                    exporter_settings.export_type,
                    ExportType::FromName | ExportType::FromUUID
                ) {
                    if let Some(ref name) = exporter_settings.new_name {
                        bp.name = name.clone()
                    }
                    if let Some(ref desc) = exporter_settings.new_desc {
                        bp.description = desc.clone()
                    }
                    if let Some(ref icon_path) = exporter_settings.new_icon {
                        if let Ok(icon_data) = fs::read(icon_path) {
                            if !is_128x128_png(&icon_data) {
                                self.toasts.error("icon must be a 128x128 png!");
                                return;
                            }
                            bp.icon = Some(icon_data);
                        } else {
                            self.toasts.error("failed to read supplied icon!");
                            return;
                        }
                    }
                }
                bp.export(
                    self.canvas_snapshot.clone(),
                    &mut self.toasts,
                    exporter_settings,
                );
            } else {
                return;
            }
        } else {
            self.toasts
                .error("No Blueprint folder set! (check settings)");
            return;
        }
    }
}

// this function was ai generated, didnt feel like doing all that math myself
fn compute_positions(
    total: usize,
    io_indices: &[usize],
    canvas_positions: &[Pos2],
    settings: &ExporterSettings,
) -> Vec<(i32, i32, i32)> {
    let io_set: std::collections::HashSet<usize> = io_indices.iter().copied().collect();
    let non_io_count = total - io_indices.len();
    let io_count = io_indices.len();

    let n = non_io_count as f32;
    let ceil_div = |a: f32, b: f32| (a / b).ceil() as usize;
    let (eff_x, eff_y, eff_z) = match (settings.max_x, settings.max_y, settings.max_z) {
        (Some(x), Some(y), Some(z)) => (x.max(1), y.max(1), z.max(1)),
        (Some(x), Some(y), None) => (x.max(1), y.max(1), ceil_div(n, (x * y) as f32).max(1)),
        (Some(x), None, Some(z)) => (x.max(1), ceil_div(n, (x * z) as f32).max(1), z.max(1)),
        (None, Some(y), Some(z)) => (ceil_div(n, (y * z) as f32).max(1), y.max(1), z.max(1)),
        (Some(x), None, None) => {
            let s = (n / x as f32).ceil().sqrt().ceil() as usize;
            (x.max(1), s.max(1), s.max(1))
        }
        (None, Some(y), None) => {
            let s = (n / y as f32).ceil().sqrt().ceil() as usize;
            (s.max(1), y.max(1), s.max(1))
        }
        (None, None, Some(z)) => {
            let s = (n / z as f32).ceil().sqrt().ceil() as usize;
            (s.max(1), s.max(1), z.max(1))
        }
        (None, None, None) => {
            let s = n.cbrt().ceil() as usize;
            (s.max(1), s.max(1), s.max(1))
        }
    };

    let io_side = (io_count as f32).sqrt().ceil() as usize;

    let mut out = vec![(0i32, 0i32, 0i32); total];
    let mut gate_counter = 0;
    let mut io_counter = 0;

    for i in 0..total {
        out[i] = if io_set.contains(&i) {
            let pos = if settings.maintain_io_position {
                (
                    -1,
                    -(canvas_positions[i].x * settings.io_x_scale).round() as i32,
                    -(canvas_positions[i].y * settings.io_y_scale).round() as i32,
                )
            } else {
                let side = io_side.max(1);
                (
                    -1,
                    -((io_counter % side) as i32),
                    -((io_counter / side) as i32),
                )
            };
            io_counter += 1;
            pos
        } else {
            let x = (gate_counter % eff_x) as i32;
            let y = ((gate_counter / eff_x) % eff_y) as i32;
            let z = ((gate_counter / (eff_x * eff_y)) % eff_z) as i32;
            gate_counter += 1;
            (x, y, z)
        };
    }

    // normalize IO parts so the corner lines up with y=1, z=0
    if let (Some(&min_y), Some(&min_z)) = (
        io_indices
            .iter()
            .map(|&i| out[i].1)
            .collect::<Vec<_>>()
            .iter()
            .min(),
        io_indices
            .iter()
            .map(|&i| out[i].2)
            .collect::<Vec<_>>()
            .iter()
            .min(),
    ) {
        for &i in io_indices {
            out[i].1 = out[i].1 - min_y + 1;
            out[i].2 = out[i].2 - min_z;
        }
    }

    out
}

fn get_user_folder() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = PathBuf::from(std::env::var("APPDATA").ok()?);

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

fn is_128x128_png(data: &[u8]) -> bool {
    if data.len() < 24 {
        return false;
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    width == 128 && height == 128
}

pub struct BluePrint {
    pub name: String,
    pub description: String,
    pub uuid: String,
    pub path: PathBuf,
    pub icon: Option<Vec<u8>>,
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
                icon: None,
            });
        }
        None
    }

    pub fn from_uuid(uuid: String, bp_folder: PathBuf) -> Option<Self> {
        let path = bp_folder.join(&uuid);
        if let Ok(contents) = fs::read_to_string(&path.join("description.json")) {
            if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                let name = &json["name"].as_str().unwrap_or("err");
                let description = &json["description"].as_str().unwrap_or("err");
                return Some(Self {
                    name: name.to_string(),
                    description: description.to_string(),
                    uuid: uuid,
                    path: path,
                    icon: None,
                });
            }
        }
        None
    }

    pub fn new(
        name: String,
        description: String,
        bp_folder: PathBuf,
        icon: Option<Vec<u8>>,
    ) -> Self {
        let uuid = Uuid::new_v4().to_string();
        let final_icon = if icon.is_some() {
            icon
        } else {
            Some(DEFAULT_ICON.to_vec())
        };
        Self {
            name: name,
            description: description,
            uuid: uuid.clone(),
            path: bp_folder.join(uuid),
            icon: final_icon,
        }
    }

    pub fn export(
        &mut self,
        canvas: CanvasSnapshot,
        toasts: &mut Toasts,
        exporter_settings: ExporterSettings,
    ) {
        let (parts, colors, positions, connections, _id_remap, _tunnels, io_parts) =
            get_canvas_raw_data(canvas, true);

        let mut out_connections: Vec<Vec<usize>> = vec![Vec::new(); parts.len()];
        for (from, to) in &connections {
            out_connections[*from].push(*to);
        }

        let positioning = compute_positions(parts.len(), &io_parts, &positions, &exporter_settings);

        let mut children: Vec<Value> = Vec::new();
        for (i, part_type) in parts.iter().enumerate() {
            let color = format!(
                "{:02X}{:02X}{:02X}",
                colors[i].r(),
                colors[i].g(),
                colors[i].b()
            );
            let controllers: Vec<Value> = out_connections[i]
                .iter()
                .map(|&id| json!({ "id": id }))
                .collect();

            let (px, py, pz) = positioning[i];
            let is_io = io_parts.contains(&i);
            let (xaxis, zaxis) = if is_io { (3, -2) } else { (2, 1) };

            let child = match part_type {
                PartType::Timer(buffer) => json!({
                    "color": color,
                    "controller": {
                        "active": false,
                        "controllers": controllers,
                        "id": i,
                        "joints": null,
                        "seconds": buffer.len() / 40,
                        "ticks": buffer.len() % 40
                    },
                    "pos": { "x": px, "y": py, "z": pz },
                    "shapeId": "8f7fd0e7-c46e-4944-a414-7ce2437bb30f",
                    "xaxis": 1,
                    "zaxis": 3
                }),
                _ => {
                    let mode = match part_type {
                        PartType::And => 0,
                        PartType::Or => 1,
                        PartType::Xor => 2,
                        PartType::Nand => 3,
                        PartType::Nor => 4,
                        PartType::Xnor => 5,
                        _ => 0,
                    };
                    json!({
                        "color": color,
                        "controller": {
                            "active": false,
                            "controllers": controllers,
                            "id": i,
                            "joints": null,
                            "mode": mode
                        },
                        "pos": { "x": px, "y": py, "z": pz },
                        "shapeId": "9f0f56e8-2c31-4d83-996c-d00a9b296c3f",
                        "xaxis": xaxis,
                        "zaxis": zaxis
                    })
                }
            };
            children.push(child);
        }

        // glass backing behind IO parts
        if !io_parts.is_empty() {
            let min_y = io_parts
                .iter()
                .map(|&i| positioning[i].1)
                .min()
                .unwrap_or(0);
            let min_z = io_parts
                .iter()
                .map(|&i| positioning[i].2)
                .min()
                .unwrap_or(0);
            let max_y = io_parts
                .iter()
                .map(|&i| positioning[i].1)
                .max()
                .unwrap_or(0);
            let max_z = io_parts
                .iter()
                .map(|&i| positioning[i].2)
                .max()
                .unwrap_or(0);
            children.push(json!({
                "bounds": { "x": 1, "y": max_y - min_y + 1, "z": max_z - min_z + 1 },
                "color": "E4F8FF",
                "pos": { "x": -1, "y": min_y - 1, "z": min_z },
                "shapeId": "5f41af56-df4c-4837-9b3c-10781335757f",
                "xaxis": 1,
                "zaxis": 3
            }));
        }

        let blueprint = json!({
            "bodies": [{ "childs": children }],
            "version": 4
        });

        _ = fs::create_dir_all(&self.path);

        if let Ok(output) = serde_json::to_string(&blueprint) {
            if fs::write(self.path.join("blueprint.json"), output).is_err() {
                toasts.error("Failed to write to blueprint.json!");
                return;
            }
        }

        if let Ok(output) = serde_json::to_string(
            &json!({ "description": self.description, "localId": self.uuid, "name": self.name, "type": "Blueprint", "version": 0}),
        ) {
            if fs::write(self.path.join("description.json"), output).is_err() {
                toasts.error("Failed to write description.json");
                if matches!(
                    fs::exists(self.path.join("description.json")),
                    Ok(true) | Err(_)
                ) {
                    _ = fs::remove_dir_all(&self.path);
                    return;
                }
            }
        }
        if let Some(icon) = self.icon.clone() {
            if fs::write(self.path.join("icon.png"), icon).is_err() {
                toasts.error("Failed to write icon.png");
            }
        }
        toasts.success("Exported blueprint!");
    }
}
