use eframe::egui::Color32;

use crate::parts::PartType;

//current tool being used
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    PlacePart(PartType),
    PlaceModule(usize),
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
