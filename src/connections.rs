use crate::AppState;
use crate::parts::Port;
use eframe::epaint::PathShape;
use egui::{Color32, Painter, Pos2, Stroke};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Connection {
    pub start: Port,
    pub end: Port,
    pub powered: bool,
}

pub fn compute_wire_route(start: Pos2, end: Pos2, zoom: f32) -> Vec<Pos2> {
    if end.x > start.x + 30.0 * zoom {
        let mid_x = (start.x + end.x) / 2.0;
        vec![
            start,
            Pos2::new(mid_x, start.y),
            Pos2::new(mid_x, end.y),
            end,
        ]
    } else {
        let offset = 20.0 * zoom;
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

pub fn draw_connection(app_state: &AppState, start_pos: Pos2, end_pos: Pos2, painter: &Painter) {
    let start = app_state.world_to_screen(start_pos);
    let end = app_state.world_to_screen(end_pos);
    let route = compute_wire_route(start, end, app_state.zoom);
    let stroke = Stroke::new(2.0 * app_state.zoom, Color32::from_rgb(180, 180, 220));
    painter.add(PathShape::line(route, stroke));
}

impl AppState {
    pub fn draw_connections(&self, painter: &Painter) -> bool {
        let mut repair = false;
        for connection in &self.canvas_snapshot.connections {
            let start_pos = connection.start.pos(self);
            let end_pos = connection.end.pos(self);
            if let (Some(start), Some(end)) = (start_pos, end_pos) {
                draw_connection(self, start, end, painter);
            } else {
                repair = true
            }
        }
        repair
    }
}
