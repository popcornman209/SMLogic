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
    pub fn handle_tool(&mut self, world_pos: Pos2, shift_held: bool) {
        match self.active_tool {
            None => {
                if let Some(part) = self.part_at_pos(world_pos) {
                    self.handle_selection(part.id, shift_held);
                } else if !shift_held {
                    self.selection.clear()
                }
            }
            Some(Tool::PlacePart(part_type)) => match self.part_at_pos(world_pos) {
                None => {
                    let part_id = Part::new(
                        part_type,
                        &mut self.canvas_snapshot,
                        world_pos,
                        self.snap_to_grid,
                    );
                    self.handle_selection(part_id, shift_held);
                }
                Some(part) => self.handle_selection(part.id, shift_held),
            },
            Some(Tool::PlaceModule(path)) => {}
            Some(Tool::Paint) => {}
            Some(Tool::Connector) => {}
        }
    }

    fn handle_selection(&mut self, part_id: u64, shift_held: bool) {
        if !shift_held {
            self.selection.clear();
        }
        self.selection.push(part_id);
    }
}
