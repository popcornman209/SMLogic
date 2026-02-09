use crate::AppState;
use crate::colors::DEFAULT_GATE_COLOR;
use crate::state::CanvasSnapshot;
use egui::{Color32, Pos2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub const GATE_SIZE: Vec2 = Vec2::new(80.0, 60.0);
pub const SWITCH_SIZE: Vec2 = Vec2::new(60.0, 60.0);
pub const PORT_SIZE: f32 = 5.0;
pub const PORT_GAP: f32 = 15.0;
pub const MIN_MODULE_WIDTH: f32 = 80.0;

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
    Button,
    Switch,
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

    /// purely logic gate parts, not sure if this will be used for anything so might remove.
    pub const GATES: &[PartType] = &[
        PartType::And,
        PartType::Or,
        PartType::Xor,
        PartType::Nand,
        PartType::Nor,
        PartType::Xnor,
    ];

    /// hows up in the io secction of the menu on the left
    pub const IO_PARTS: &[PartType] = &[
        PartType::Input,
        PartType::Output,
        PartType::Button,
        PartType::Switch,
        PartType::Label,
    ];

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
            PartType::Button => "Button",
            PartType::Switch => "Swtich",
            PartType::Label => "Label",
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
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
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Gate {
    pub gate_type: GateType,
    pub powered: bool,
    pub powered_next: bool,
}
impl Gate {
    pub fn new(gate_type: GateType) -> (PartData, String, Vec2) {
        (
            PartData::Gate(Self {
                gate_type: gate_type.clone(),
                powered: false,
                powered_next: false,
            }),
            gate_type.to_label(),
            -GATE_SIZE / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Timer {
    pub buffer: Vec<bool>,
    pub secs: u8,
    pub ticks: u8,
}
impl Timer {
    pub fn new() -> (PartData, String, Vec2) {
        (
            PartData::Timer(Self {
                buffer: Vec::new(),
                secs: 0,
                ticks: 0,
            }),
            "Timer".to_string(),
            -GATE_SIZE / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Module {
    pub path: PathBuf,
    pub inputs: Vec<(u64, String)>,
    pub outputs: Vec<(u64, String)>,
    pub canvas_snapshot: CanvasSnapshot,
    pub min_size: Vec2,
    pub size: Vec2,
}
impl Module {
    pub fn reload(&mut self) {
        self.canvas_snapshot = CanvasSnapshot::load(&self.path);

        // load inputs/outputs
        self.inputs = Vec::new();
        self.outputs = Vec::new();
        for part in self.canvas_snapshot.parts.values() {
            if let PartData::IO(io) = &part.part_data {
                if io.input {
                    self.inputs.push((part.id, part.label.clone()))
                } else {
                    self.outputs.push((part.id, part.label.clone()))
                }
            }
        }

        // make sure height is tall enough
        let max_len = self.inputs.len().max(self.outputs.len()) as f32;
        self.min_size.y = GATE_SIZE.y + (PORT_GAP * (max_len - 1.0));
        if self.size.y <= self.min_size.y {
            self.size.y = self.min_size.y
        };
    }
    pub fn new(path: PathBuf) -> (PartData, String, Vec2) {
        let mut module = Self {
            path: path,
            inputs: Vec::new(),
            outputs: Vec::new(),
            canvas_snapshot: CanvasSnapshot {
                parts: HashMap::new(),
                connections: Vec::new(),
                next_id: 0,
            },
            min_size: Vec2::new(MIN_MODULE_WIDTH, 0.0),
            size: Vec2::new(120.0, 0.0),
        };
        module.reload();
        (
            PartData::Module(module.clone()),
            "template label".to_string(),
            -module.size / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct Switch {
    pub toggle: bool,
    pub powered: bool,
}
impl Switch {
    pub fn new(toggle: bool) -> (PartData, String, Vec2) {
        (
            PartData::Switch(Self {
                toggle: toggle,
                powered: false,
            }),
            if toggle { "Switch" } else { "Button" }.to_string(),
            -GATE_SIZE / 2.0,
        )
    }
}

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub enum PartData {
    Gate(Gate),
    Timer(Timer),
    Module(Module),
    IO(IO),
    Switch(Switch),
    Label(Label),
}
impl PartData {
    pub fn size(&self) -> Vec2 {
        match self {
            PartData::Gate(_) => GATE_SIZE,
            PartData::Timer(_) => GATE_SIZE,
            PartData::Module(module) => module.size,
            PartData::IO(_) => GATE_SIZE,
            PartData::Switch(_) => SWITCH_SIZE,
            PartData::Label(label) => label.size,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Port {
    pub part: u64,
    pub input: bool,
    pub port_id: Option<u64>,
}
impl Port {
    pub fn pos(&self, app_state: &AppState) -> Option<Pos2> {
        if self.input {
            app_state.canvas_snapshot.parts[&self.part].input_pos(self.port_id)
        } else {
            app_state.canvas_snapshot.parts[&self.part].output_pos(self.port_id)
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Part {
    pub id: u64,
    pub part_data: PartData,
    pub pos: Pos2,
    pub label: String,
    pub color: Color32,
}
impl Part {
    pub fn new(
        part: PartType,
        snapshot: &mut CanvasSnapshot,
        pos: Pos2,
        snap_to_grid: bool,
    ) -> u64 {
        let (part_data, label, pos_offset): (PartData, String, Vec2) = match part.clone() {
            PartType::Timer => Timer::new(),
            PartType::Input | PartType::Output => IO::new(part == PartType::Input),
            PartType::Button | PartType::Switch => Switch::new(part == PartType::Switch),
            PartType::Label => Label::new(),
            PartType::Module(path) => Module::new(path),
            _ => Gate::new(GateType::from_part_type(part)),
        };
        let id = snapshot.next_id;
        snapshot.next_id += 1;
        let mut part = Self {
            id: id,
            part_data: part_data,
            pos: pos + pos_offset,
            label: label,
            color: DEFAULT_GATE_COLOR,
        };
        if snap_to_grid {
            part.snap_pos();
        }
        snapshot.parts.insert(id, part);
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
            PartData::Switch(_) | PartData::Label(_) => None,
            PartData::IO(io) => {
                if io.input {
                    None
                } else {
                    Some(Pos2::new(self.pos.x, self.pos.y + GATE_SIZE.y / 2.0))
                }
            }
            PartData::Module(module) => None, // FIX THIS!!
        }
    }

    pub fn output_pos(&self, port_id: Option<u64>) -> Option<Pos2> {
        match &self.part_data {
            PartData::Gate(_) | PartData::Timer(_) | PartData::Switch(_) => Some(Pos2::new(
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
            PartData::Module(_) => None, // FIX THIS!!
        }
    }

    pub fn connections_pos(&self) -> Vec<Pos2> {
        match &self.part_data {
            PartData::Gate(_) | PartData::Timer(_) => vec![
                Pos2::new(
                    self.pos.x + self.part_data.size().x,
                    self.pos.y + self.part_data.size().y / 2.0,
                ),
                Pos2::new(self.pos.x, self.pos.y + GATE_SIZE.y / 2.0),
            ],
            PartData::Switch(_) => vec![Pos2::new(
                self.pos.x + SWITCH_SIZE.x,
                self.pos.y + SWITCH_SIZE.y / 2.0,
            )],
            PartData::IO(io) => vec![Pos2::new(
                self.pos.x + if io.input { GATE_SIZE.x } else { 0.0 },
                self.pos.y + GATE_SIZE.y / 2.0,
            )],
            PartData::Module(module) => Vec::new(),
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
            PartData::Switch(_) => vec![(
                Pos2::new(self.pos.x + SWITCH_SIZE.x, self.pos.y + SWITCH_SIZE.y / 2.0),
                false,
                None,
            )],
            PartData::IO(io) => vec![(
                Pos2::new(
                    self.pos.x + if io.input { GATE_SIZE.x } else { 0.0 },
                    self.pos.y + GATE_SIZE.y / 2.0,
                ),
                io.input,
                None,
            )],
            PartData::Module(module) => Vec::new(),
            PartData::Label(_) => Vec::new(),
        }
    }
}
