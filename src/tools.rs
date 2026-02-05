use eframe::egui::{Color32, Pos2};

use crate::AppState;
use crate::parts::{Part, PartType};

//current tool being used
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    PlacePart(PartType),
    PlaceModule(&'static str),
    Paint,
    Connector,
}

impl Tool {
    pub const TOOLS: &[Option<Tool>] = &[None, Some(Tool::Paint), Some(Tool::Connector)];
}

pub fn tool_label(tool: Option<Tool>) -> &'static str {
    match tool {
        None => "Select",
        Some(Tool::Paint) => "Paint Tool",
        Some(Tool::Connector) => "Connnector",
        _ => "???",
    }
}

impl AppState {
    pub fn handle_tool(&mut self, pos: Pos2) {
        match self.active_tool {
            Some(Tool::PlacePart(part_type)) => {
                println!("{}", Part::new(part_type, &mut self.canvas_snapshot, pos));
            }
            Some(Tool::PlaceModule(id)) => {}
            Some(Tool::Paint) => {}
            Some(Tool::Connector) => {}
            None => {}
        }
    }
}
