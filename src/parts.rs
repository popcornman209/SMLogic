use crate::state::CanvasSnapshot;
use egui::{Color32, Pos2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Clone)]
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

pub struct Gate {
    gate_type: GateType,
    powered: bool,
    powered_next: bool,
}
impl Gate {
    pub fn new(gate_type: GateType) -> (PartData, String) {
        (
            PartData::Gate(Self {
                gate_type: gate_type.clone(),
                powered: false,
                powered_next: false,
            }),
            gate_type.to_label(),
        )
    }
}

pub struct Timer {
    buffer: Vec<bool>,
    secs: u8,
    ticks: u8,
}
impl Timer {
    pub fn new() -> (PartData, String) {
        (
            PartData::Timer(Self {
                buffer: Vec::new(),
                secs: 0,
                ticks: 0,
            }),
            "Timer".to_string(),
        )
    }
}

pub struct Module {
    path: &'static str,
    inputs: Vec<u64>,
    outputs: Vec<u64>,
    canvas: CanvasSnapshot,
    size: Vec2,
}
impl Module {
    pub fn new(path: &'static str) -> (PartData, String) {
        //TODO make actually load module instead of... this
        (
            PartData::Module(Self {
                path: path,
                inputs: Vec::new(),
                outputs: Vec::new(),
                canvas: CanvasSnapshot {
                    parts: HashMap::new(),
                    next_id: 0,
                },
                size: Vec2::new(0.0, 0.0),
            }),
            "template label".to_string(),
        )
    }
}

pub struct IO {
    input: bool,
}
impl IO {
    pub fn new(input: bool) -> (PartData, String) {
        (
            PartData::IO(Self { input: input }),
            if input { "Input" } else { "Output" }.to_string(),
        )
    }
}

pub struct Switch {
    toggle: bool,
    powered: bool,
}
impl Switch {
    pub fn new(toggle: bool) -> (PartData, String) {
        (
            PartData::Switch(Self {
                toggle: toggle,
                powered: false,
            }),
            "Switch".to_string(),
        )
    }
}

pub struct Label {
    size: Vec2,
}
impl Label {
    pub fn new() -> (PartData, String) {
        (
            PartData::Label(Self {
                size: Vec2::new(100.0, 25.0),
            }),
            "Label".to_string(),
        )
    }
}

pub enum PartData {
    Gate(Gate),
    Timer(Timer),
    Module(Module),
    IO(IO),
    Switch(Switch),
    Label(Label),
}

pub struct Part {
    id: u64,
    part_data: PartData,
    pos: Pos2,
    label: String,
}
impl Part {
    pub fn new(part: PartType, snapshot: &mut CanvasSnapshot, pos: Pos2) -> u64 {
        let (part_data, label): (PartData, String) = match part.clone() {
            PartType::Timer => Timer::new(),
            // PartType::Input => {}
            // PartType::Output => {}
            PartType::Button | PartType::Switch => Switch::new(part == PartType::Switch),
            // PartType::Label => {}
            _ => Gate::new(GateType::from_part_type(part)),
        };
        snapshot.next_id += 1;
        snapshot.parts.insert(
            snapshot.next_id,
            Self {
                id: snapshot.next_id,
                part_data: part_data,
                pos: pos,
                label: label,
            },
        );
        snapshot.next_id
    }
}
