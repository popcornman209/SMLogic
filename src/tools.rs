use eframe::egui::{Color32, Pos2};

use crate::AppState;
use crate::parts::{Part, PartType};

//current tool being used
#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    PlacePart(PartType),
    Paint,
    Connector,
}

impl Tool {
    pub const TOOLS: &[Option<Tool>] = &[None, Some(Tool::Paint), Some(Tool::Connector)];
}

pub fn tool_label(tool: &Option<Tool>) -> &'static str {
    match tool {
        None => "Select",
        Some(Tool::Paint) => "Paint Tool",
        Some(Tool::Connector) => "Connnector",
        _ => "???",
    }
}

impl AppState {
    pub fn handle_tool(&mut self, world_pos: Pos2, shift_held: bool) {
        match &self.active_tool {
            None => {}
            Some(Tool::PlacePart(part_type)) => {
                let part_id = Part::new(part_type.clone(), self, world_pos);
                self.select_part(part_id, shift_held);
            }
            Some(Tool::Paint) => {}
            Some(Tool::Connector) => {}
        }
    }

    pub fn select_part(&mut self, part_id: u64, shift_held: bool) {
        if !shift_held {
            self.selection.clear();
        }
        if !self.selection.contains(&part_id) {
            self.selection.push(part_id);
        }
    }
}
