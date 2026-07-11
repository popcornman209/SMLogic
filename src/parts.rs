use crate::AppState;
use crate::colors::DEFAULT_GATE_COLOR;
use crate::state::{CanvasSnapshot, path_to_string};
use crate::tools::sort_by_position;
use egui::{Color32, Pos2, Vec2};
use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

pub const GATE_SIZE: Vec2 = Vec2::new(80.0, 60.0);
pub const PORT_SIZE: f32 = 6.0;
pub const PORT_GAP: f32 = 20.0;
pub const MIN_MODULE_WIDTH: f32 = 120.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartType {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    Timer,
    Module(PathBuf),
    Input,
    Output,
    Label,
}

impl PartType {
    /// all main logic parts, show up at the top of the left menu
    pub const MAIN_PARTS: &[PartType] = &[
        PartType::And,
        PartType::Or,
        PartType::Xor,
        PartType::Nand,
        PartType::Nor,
        PartType::Xnor,
        PartType::Timer,
    ];

    /// hows up in the io secction of the menu on the left
    pub const IO_PARTS: &[PartType] = &[PartType::Input, PartType::Output, PartType::Label];

    pub fn label(&self) -> &'static str {
        match self {
            PartType::And => "AND",
            PartType::Or => "OR",
            PartType::Xor => "XOR",
            PartType::Nand => "NAND",
            PartType::Nor => "NOR",
            PartType::Xnor => "XNOR",
            PartType::Timer => "Timer",
            PartType::Module(_) => "Module",
            PartType::Input => "Input",
            PartType::Output => "Output",
            PartType::Label => "Label",
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub enum GateType {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
}
impl GateType {
    pub fn to_label(&self) -> String {
        match self {
            GateType::And => "AND",
            GateType::Or => "OR",
            GateType::Xor => "XOR",
            GateType::Nand => "NAND",
            GateType::Nor => "NOR",
            GateType::Xnor => "XNOR",
        }
        .to_string()
    }

    pub fn from_part_type(part: PartType) -> Self {
        match part {
            PartType::Or => Self::Or,
            PartType::Xor => Self::Xor,
            PartType::Nand => Self::Nand,
            PartType::Nor => Self::Nor,
            PartType::Xnor => Self::Xnor,
            _ => Self::And,
        }
    }
    pub const TYPES: &[Self] = &[
        Self::And,
        Self::Or,
        Self::Xor,
        Self::Nand,
        Self::Nor,
        Self::Xnor,
    ];
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Gate {
    pub gate_type: GateType,
    #[serde(default)]
    pub important: bool,
}
impl Gate {
    pub fn new(gate_type: GateType) -> (PartData, String, Vec2) {
        (
            PartData::Gate(Self {
                gate_type: gate_type.clone(),
                important: false,
            }),
            gate_type.to_label(),
            -GATE_SIZE / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Timer {
    pub secs: u8,
    pub ticks: u8,
}
impl Timer {
    pub fn new() -> (PartData, String, Vec2) {
        (
            PartData::Timer(Self { secs: 0, ticks: 0 }),
            "Timer".to_string(),
            -GATE_SIZE / 2.0,
        )
    }
    pub fn get_ticks(self) -> usize {
        return (self.ticks + self.secs * 40).into();
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Module {
    pub path: PathBuf,
    #[serde(skip)]
    pub inputs: BTreeMap<u64, String>,
    #[serde(skip)]
    pub outputs: BTreeMap<u64, String>,
    pub canvas_snapshot: CanvasSnapshot,
    #[serde(skip)]
    pub min_size: Vec2,
    pub size: Vec2,
    #[serde(skip)]
    pub problematic: bool,
}
impl Module {
    pub fn reload(
        &mut self,
        project_path: Option<PathBuf>,
        toasts: &mut Toasts,
        ancestors: Vec<PathBuf>,
    ) {
        let full_path = if let Some(ref proj) = project_path {
            proj.join(&self.path)
        } else {
            self.path.clone()
        };
        if ancestors.contains(&full_path) {
            toasts.error(format!(
                "Cant put a module inside of itself! ({})",
                self.path.to_string_lossy()
            ));
            return;
        } else {
            let mut new_ancestors = ancestors.clone();
            new_ancestors.push(full_path.clone());
            match CanvasSnapshot::load(full_path, project_path.clone(), toasts, new_ancestors) {
                Ok(snapshot) => self.canvas_snapshot = snapshot,
                Err(e) => {
                    self.problematic = true;
                    toasts.error(format!("Failed to load file: {}", e));
                    return;
                }
            }
        }

        // load inputs/outputs
        self.inputs = BTreeMap::new();
        self.outputs = BTreeMap::new();
        for part in self.canvas_snapshot.parts.values() {
            if let PartData::IO(io) = &part.part_data {
                if io.input {
                    self.inputs.insert(part.id, part.label.clone());
                } else {
                    self.outputs.insert(part.id, part.label.clone());
                }
            }
        }

        // make sure height is tall enough
        let max_len = self.inputs.len().max(self.outputs.len()) as f32;
        self.min_size.y = GATE_SIZE.y + (PORT_GAP * (max_len - 1.0));
        if self.size.y <= self.min_size.y {
            self.size.y = self.min_size.y
        };
        if ancestors.is_empty() {
            toasts.success(format!(
                "Loaded module: {}",
                path_to_string(self.path.clone(), project_path)
            ));
        }
        self.problematic = false;
    }
    pub fn new(path: PathBuf, app_state: &mut AppState) -> (PartData, String, Vec2) {
        let final_path = if let Some(project_folder) = &app_state.project_folder {
            path.strip_prefix(project_folder)
                .map(|p| p.to_path_buf())
                .unwrap_or(path)
        } else {
            path
        };
        let mut module = Self {
            path: final_path.clone(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            canvas_snapshot: CanvasSnapshot {
                parts: HashMap::new(),
                connections: Vec::new(),
                next_id: 0,
            },
            min_size: Vec2::new(MIN_MODULE_WIDTH, 0.0),
            size: Vec2::new(120.0, 0.0),
            problematic: false,
        };
        module.reload(
            app_state.project_folder.clone(),
            &mut app_state.toasts,
            Vec::new(),
        );
        (
            PartData::Module(module.clone()),
            final_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            -module.size / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct IO {
    pub input: bool,
}
impl IO {
    pub fn new(input: bool) -> (PartData, String, Vec2) {
        (
            PartData::IO(Self { input: input }),
            if input { "Input" } else { "Output" }.to_string(),
            -GATE_SIZE / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Label {
    pub size: Vec2,
}
impl Label {
    pub fn new() -> (PartData, String, Vec2) {
        let size = Vec2::new(100.0, 20.0);
        (
            PartData::Label(Self { size: size }),
            "Label".to_string(),
            -size / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub enum PartData {
    Gate(Gate),
    Timer(Timer),
    Module(Module),
    IO(IO),
    Label(Label),
}
impl PartData {
    pub fn size(&self) -> Vec2 {
        match self {
            PartData::Gate(_) => GATE_SIZE,
            PartData::Timer(_) => GATE_SIZE,
            PartData::Module(module) => module.size,
            PartData::IO(_) => GATE_SIZE,
            PartData::Label(label) => label.size,
        }
    }
    pub fn max_connections(&self) -> u64 {
        match self {
            PartData::Gate(_) => 255,
            _ => 1,
        }
    }
    pub fn resizable(&self) -> bool {
        match self {
            Self::Module(_) | Self::Label(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Copy)]
pub struct Port {
    pub part: u64,
    pub input: bool,
    pub port_id: Option<u64>,
}
impl Port {
    pub fn pos(&self, app_state: &AppState) -> Option<Pos2> {
        if let Some(part) = app_state.canvas_snapshot.parts.get(&self.part) {
            if self.input {
                part.input_pos(self.port_id)
            } else {
                part.output_pos(self.port_id)
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Part {
    pub id: u64,
    pub part_data: PartData,
    pub pos: Pos2,
    pub label: String,
    pub color: Color32,
    #[serde(skip)]
    pub simulation_index: Option<usize>,
}
impl Part {
    pub fn new(part: PartType, app_state: &mut AppState, pos: Pos2) -> u64 {
        let (part_data, label, pos_offset): (PartData, String, Vec2) = match part.clone() {
            PartType::Timer => Timer::new(),
            PartType::Input | PartType::Output => IO::new(part == PartType::Input),
            PartType::Label => Label::new(),
            PartType::Module(path) => Module::new(path, app_state),
            _ => Gate::new(GateType::from_part_type(part)),
        };
        let id = app_state.canvas_snapshot.next_id;
        app_state.canvas_snapshot.next_id += 1;
        let mut part = Self {
            id: id,
            part_data: part_data,
            pos: pos + pos_offset,
            label: label,
            color: DEFAULT_GATE_COLOR,
            simulation_index: None,
        };
        if app_state.snap_to_grid {
            part.snap_pos();
        }
        app_state.canvas_snapshot.parts.insert(id, part);
        id
    }

    pub fn snap_pos(&mut self) {
        self.pos = Pos2::new(
            (self.pos.x / 20.0).round() * 20.0,
            (self.pos.y / 20.0).round() * 20.0,
        )
    }

    pub fn input_pos(&self, port_id: Option<u64>) -> Option<Pos2> {
        match &self.part_data {
            PartData::Gate(_) | PartData::Timer(_) => {
                Some(Pos2::new(self.pos.x, self.pos.y + GATE_SIZE.y / 2.0))
            }
            PartData::Label(_) => None,
            PartData::IO(io) => {
                if io.input {
                    None
                } else {
                    Some(Pos2::new(self.pos.x, self.pos.y + GATE_SIZE.y / 2.0))
                }
            }
            PartData::Module(module) => {
                if let Some(port) = port_id {
                    let mut ids: Vec<u64> = module.inputs.keys().cloned().collect();
                    sort_by_position(&mut ids, |id| {
                        module
                            .canvas_snapshot
                            .parts
                            .get(id)
                            .map(|p| p.pos)
                            .unwrap_or_default()
                    });
                    if let Some(index) = ids.iter().position(|id| *id == port) {
                        return Some(Pos2::new(
                            self.pos.x,
                            GATE_SIZE.y / 2.0 + PORT_GAP * index as f32 + self.pos.y,
                        ));
                    };
                };
                None
            }
        }
    }

    pub fn output_pos(&self, port_id: Option<u64>) -> Option<Pos2> {
        match &self.part_data {
            PartData::Gate(_) | PartData::Timer(_) => Some(Pos2::new(
                self.pos.x + self.part_data.size().x,
                self.pos.y + self.part_data.size().y / 2.0,
            )),
            PartData::Label(_) => None,
            PartData::IO(io) => {
                if io.input {
                    Some(Pos2::new(
                        self.pos.x + GATE_SIZE.x,
                        self.pos.y + GATE_SIZE.y / 2.0,
                    ))
                } else {
                    None
                }
            }
            PartData::Module(module) => {
                if let Some(port) = port_id {
                    let mut ids: Vec<u64> = module.outputs.keys().cloned().collect();
                    sort_by_position(&mut ids, |id| {
                        module
                            .canvas_snapshot
                            .parts
                            .get(id)
                            .map(|p| p.pos)
                            .unwrap_or_default()
                    });
                    if let Some(index) = ids.iter().position(|id| *id == port) {
                        return Some(Pos2::new(
                            self.pos.x + module.size.x,
                            GATE_SIZE.y / 2.0 + PORT_GAP * index as f32 + self.pos.y,
                        ));
                    };
                };
                None
            } // FIX THIS!! (im back 4 months later, what is there to fix???)
        }
    }

    pub fn get_ports(&self) -> Vec<Port> {
        match &self.part_data {
            PartData::Gate(_) | PartData::Timer(_) => vec![
                Port {
                    part: self.id,
                    port_id: None,
                    input: true,
                },
                Port {
                    part: self.id,
                    port_id: None,
                    input: false,
                },
            ],
            PartData::IO(io) => vec![Port {
                part: self.id,
                port_id: None,
                input: !io.input,
            }],
            PartData::Module(module) => {
                let mut result = Vec::new();
                for id in module.inputs.keys() {
                    result.push(Port {
                        part: self.id,
                        port_id: Some(id.clone()),
                        input: true,
                    });
                }
                for id in module.outputs.keys() {
                    result.push(Port {
                        part: self.id,
                        port_id: Some(id.clone()),
                        input: false,
                    });
                }
                result
            }
            PartData::Label(_) => Vec::new(),
        }
    }

    pub fn connections_pos_with_id(&self) -> Vec<(Pos2, bool, Option<u64>)> {
        match &self.part_data {
            PartData::Gate(_) | PartData::Timer(_) => vec![
                (
                    Pos2::new(
                        self.pos.x + self.part_data.size().x,
                        self.pos.y + self.part_data.size().y / 2.0,
                    ),
                    false,
                    None,
                ),
                (
                    Pos2::new(self.pos.x, self.pos.y + GATE_SIZE.y / 2.0),
                    true,
                    None,
                ),
            ],
            PartData::IO(io) => vec![(
                Pos2::new(
                    self.pos.x + if io.input { GATE_SIZE.x } else { 0.0 },
                    self.pos.y + GATE_SIZE.y / 2.0,
                ),
                !io.input,
                None,
            )],
            PartData::Module(module) => {
                let mut result = Vec::new();
                let mut input_ids: Vec<u64> = module.inputs.keys().cloned().collect();
                sort_by_position(&mut input_ids, |id| {
                    module
                        .canvas_snapshot
                        .parts
                        .get(id)
                        .map(|p| p.pos)
                        .unwrap_or_default()
                });
                for (i, id) in input_ids.iter().enumerate() {
                    result.push((
                        Pos2::new(
                            self.pos.x,
                            GATE_SIZE.y / 2.0 + PORT_GAP * i as f32 + self.pos.y,
                        ),
                        true,
                        Some(id.clone()),
                    ));
                }
                let mut output_ids: Vec<u64> = module.outputs.keys().cloned().collect();
                sort_by_position(&mut output_ids, |id| {
                    module
                        .canvas_snapshot
                        .parts
                        .get(id)
                        .map(|p| p.pos)
                        .unwrap_or_default()
                });
                for (i, id) in output_ids.iter().enumerate() {
                    result.push((
                        Pos2::new(
                            self.pos.x + module.size.x,
                            GATE_SIZE.y / 2.0 + PORT_GAP * i as f32 + self.pos.y,
                        ),
                        false,
                        Some(id.clone()),
                    ));
                }
                result
            }
            PartData::Label(_) => Vec::new(),
        }
    }
}
