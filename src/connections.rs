use crate::parts::Port;
use crate::state::{AppState, Selection};
use eframe::epaint::PathShape;
use egui::{Color32, Painter, Pos2, Rect, Stroke};
use serde::{Deserialize, Serialize};

pub const WIRE_WIDTH: f32 = 2.0;

#[derive(Clone, Deserialize, Serialize)]
pub struct Connection {
    pub start: Port,
    pub end: Port,
    pub powered: bool,
}

pub fn dist_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;

    let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
    let closest = a + ab * t;
    p.distance(closest)
}
pub fn closest_point_to_rect(rect: Rect, point: Pos2) -> Pos2 {
    let x = point.x.clamp(rect.left(), rect.right());
    let y = point.y.clamp(rect.top(), rect.bottom());
    Pos2::new(x, y)
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

fn route_world_to_screen(app_state: &AppState, route: &mut Vec<Pos2>) {
    for pos in route.iter_mut() {
        *pos = app_state.world_to_screen(*pos);
    }
}

pub fn draw_connection(
    app_state: &AppState,
    start_pos: Pos2,
    end_pos: Pos2,
    painter: &Painter,
    selected: bool,
) {
    let color: Color32 = if selected {
        app_state.color_pallet.selection
    } else {
        Color32::from_rgb(180, 180, 220)
    };

    let mut route = compute_wire_route(start_pos, end_pos);
    route_world_to_screen(app_state, &mut route);
    let stroke = Stroke::new(WIRE_WIDTH * app_state.zoom, color);
    painter.add(PathShape::line(route, stroke));
}

impl AppState {
    pub fn draw_connections(&self, painter: &Painter) -> bool {
        let mut repair = false;
        for (i, connection) in self.canvas_snapshot.connections.iter().enumerate() {
            let start_pos = connection.start.pos(self);
            let end_pos = connection.end.pos(self);
            if let (Some(start), Some(end)) = (start_pos, end_pos) {
                draw_connection(
                    self,
                    start,
                    end,
                    painter,
                    self.selection.contains(&Selection::Connection(i)),
                );
            } else {
                repair = true
            }
        }
        repair
    }
}
