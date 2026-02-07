use crate::AppState;
use crate::colors::DEFAULT_GATE_COLOR;
use crate::state::CanvasSnapshot;
use egui::{Color32, Pos2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const GATE_SIZE: Vec2 = Vec2::new(80.0, 60.0);
pub const SWITCH_SIZE: Vec2 = Vec2::new(60.0, 60.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartType {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    Timer,
    Module,
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
            PartType::Module => "Module",
            PartType::Input => "Input",
            PartType::Output => "Output",
            PartType::Button => "Button",
            PartType::Switch => "Swtich",
            PartType::Label => "Label",
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Module {
    pub path: &'static str,
    pub inputs: Vec<(u64, &'static str)>,
    pub outputs: Vec<(u64, &'static str)>,
    pub canvas: CanvasSnapshot,
    pub size: Vec2,
}
impl Module {
    pub fn new(path: &'static str) -> (PartData, String, Vec2) {
        //TODO make actually load module instead of... this
        let size = Vec2::new(120.0, 100.0);
        (
            PartData::Module(Self {
                path: path,
                inputs: Vec::new(),
                outputs: Vec::new(),
                canvas: CanvasSnapshot {
                    parts: HashMap::new(),
                    next_id: 0,
                },
                size: size,
            }),
            "template label".to_string(),
            -size / 2.0,
        )
    }
}

#[derive(Clone, Copy)]
pub struct IO {
    pub input: bool,
    pub powered: bool,
    pub powered_next: bool,
}
impl IO {
    pub fn new(input: bool) -> (PartData, String, Vec2) {
        (
            PartData::IO(Self {
                input: input,
                powered: false,
                powered_next: false,
            }),
            if input { "Input" } else { "Output" }.to_string(),
            -GATE_SIZE / 2.0,
        )
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub enum PartData {
    Gate(Gate),
    Timer(Timer),
    Module(Module),
    IO(IO),
    Switch(Switch),
    Label(Label),
}

#[derive(Clone)]
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
}
