use crate::AppState;
use crate::parts::Part;
use egui::{Color32, Painter, Pos2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Connection {
    start: u64,
    start_port: Option<u64>,
    end: u64,
    end_port: Option<u64>,
}

fn draw_connection(app_state: &AppState, start_pos: Pos2, end_pos: Pos2) {
    let start = app_state.world_to_screen(start_pos);
    let end = app_state.world_to_screen(end_pos);
}

impl AppState {
    pub fn draw_connections(&self) -> bool {
        let mut repair = false;
        for connection in &self.canvas_snapshot.connections {
            let start_pos =
                self.canvas_snapshot.parts[&connection.start].output_pos(connection.start_port);
            let end_pos =
                self.canvas_snapshot.parts[&connection.end].input_pos(connection.end_port);
            if let (Some(start), Some(end)) = (start_pos, end_pos) {
                draw_connection(self, start, end);
            } else {
                repair = true
            }
        }
        repair
    }

    pub fn compute_wire_route(start: Pos2, end: Pos2) -> Vec<Pos2> {
        if end.x > start.x + 30.0 {
            let mid_x = (start.x + end.x) / 2.0;
            vec![
                start,
                Pos2::new(mid_x, start.y),
                Pos2::new(mid_x, end.y),
                end,
            ]
        } else {
            let offset = 20.0;
            let mid_y = if (start.y - end.y).abs() < 1.0 {
                start.y - 40.0
            } else {
                (start.y + end.y) / 2.0
            };

            vec![
                start,
                Pos2::new(start.x + offset, start.y),
                Pos2::new(start.x + offset, mid_y),
                Pos2::new(end.x - offset, mid_y),
                Pos2::new(end.x - offset, end.y),
                end,
            ]
        }
    }
}
