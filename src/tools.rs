use crate::parts::{Part, PartType, Port};
use crate::state::{AppState, Selection};
use eframe::egui::Pos2;

#[derive(Clone, PartialEq)]
pub enum ConnectorMode {
    AllToAll,
}

#[derive(Clone, PartialEq)]
pub struct ConnectorData {
    pub selecting_inputs: bool,
    pub selected_inputs: Vec<Port>,
    pub selected_outputs: Vec<Port>,
    pub mode: ConnectorMode,
}

//current tool being used
#[derive(Clone, PartialEq)]
pub enum Tool {
    PlacePart(PartType),
    Paint,
    Connector(ConnectorData),
}

impl Tool {
    pub const TOOLS: &[Option<Tool>] = &[
        None,
        Some(Tool::Paint),
        Some(Tool::Connector(ConnectorData {
            selecting_inputs: true,
            selected_inputs: Vec::new(),
            selected_outputs: Vec::new(),
            mode: ConnectorMode::AllToAll,
        })),
    ];
}

pub fn tool_label(tool: &Option<Tool>) -> &'static str {
    match tool {
        None => "Select",
        Some(Tool::Paint) => "Paint Tool",
        Some(Tool::Connector(_)) => "Connnector",
        _ => "???",
    }
}

impl AppState {
    pub fn handle_tool(&mut self, world_pos: Pos2, shift_held: bool) {
        match self.active_tool.clone() {
            None | Some(Tool::Connector(_)) => {}
            Some(Tool::PlacePart(part_type)) => {
                self.push_undo();
                let part_id = Part::new(part_type, self, world_pos);
                self.reload_connection_counts();
                self.select_part(part_id, shift_held);
            }
            Some(Tool::Paint) => {
                for selection in self.selection.clone() {
                    if let Selection::Part(part_id) = selection {
                        if let Some(part) = self.canvas_snapshot.parts.get_mut(&part_id) {
                            part.color = self.current_paint_color.clone();
                        }
                    }
                }
                self.selection.clear();
            }
        }
    }

    pub fn select_part(&mut self, part_id: u64, shift_held: bool) {
        if !shift_held {
            self.selection.clear();
        }
        if !self.selection.contains(&Selection::Part(part_id)) {
            self.selection.push(Selection::Part(part_id));
        }
    }
    pub fn select_connection(&mut self, connection_id: usize, shift_held: bool) {
        if !shift_held {
            self.selection.clear();
        }
        if !self
            .selection
            .contains(&Selection::Connection(connection_id))
        {
            self.selection.push(Selection::Connection(connection_id));
        }
    }
}
