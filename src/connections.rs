use crate::parts::{PartData, Port};
use crate::state::{AppState, Selection};
use eframe::epaint::PathShape;
use egui::{Color32, Painter, Pos2, Stroke};
use serde::{Deserialize, Serialize};

pub const WIRE_WIDTH: f32 = 2.0;

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Connection {
    pub start: Port,
    pub end: Port,
}

pub fn dist_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;

    let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
    let closest = a + ab * t;
    p.distance(closest)
}

const CORNER_RADIUS: f32 = 10.0;

// THE BELOW FUNCTION IS AI GENERATED (i dont know how to do the math behind that)
/// Replaces each sharp corner with a small quadratic bezier curve (3 intermediate points).
fn round_route(points: Vec<Pos2>) -> Vec<Pos2> {
    if points.len() < 3 {
        return points;
    }
    let mut out = Vec::with_capacity(points.len() + points.len() * 4);
    out.push(points[0]);
    for i in 1..points.len() - 1 {
        let prev = points[i - 1];
        let curr = points[i];
        let next = points[i + 1];
        let d1 = (curr - prev).normalized();
        let d2 = (next - curr).normalized();
        let r = CORNER_RADIUS
            .min(curr.distance(prev) * 0.5)
            .min(curr.distance(next) * 0.5);
        let p1 = curr - d1 * r; // approach point
        let p2 = curr + d2 * r; // exit point
        out.push(p1);
        for j in 1u8..=3 {
            let t = j as f32 / 4.0;
            let u = 1.0 - t;
            out.push(Pos2::new(
                u * u * p1.x + 2.0 * u * t * curr.x + t * t * p2.x,
                u * u * p1.y + 2.0 * u * t * curr.y + t * t * p2.y,
            ));
        }
        out.push(p2);
    }
    out.push(*points.last().unwrap());
    out
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

    let mut route = if app_state.round_connections {
        round_route(compute_wire_route(start_pos, end_pos))
    } else {
        compute_wire_route(start_pos, end_pos)
    };
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

    pub fn add_connection(&mut self, connection: Connection, reload: bool) -> bool {
        if connection.start.input != connection.end.input {
            // cant connect two inputs or outputs
            let count = self
                .connection_counts
                .get(&connection.end)
                .copied()
                .unwrap_or(0);
            if let Some(end_part) = self.canvas_snapshot.parts.get(&connection.end.part) {
                if count < end_part.part_data.max_connections() {
                    // dont connect more than the max
                    if let Some(start_part) = self.canvas_snapshot.parts.get(&connection.start.part)
                    {
                        if !(matches!(start_part.part_data, PartData::IO(_))
                            && matches!(end_part.part_data, PartData::IO(_)))
                        {
                            // connecting a input directly to an output makes it too complicated
                            if (!self.canvas_snapshot.connections.iter().any(|c| {
                                c.start.part == connection.end.part
                                    && c.end.part == connection.start.part
                            })) | matches!(start_part.part_data, PartData::Module(_))
                            {
                                // two gates cannot connect in a loop
                                if reload {
                                    self.canvas_snapshot.connections.push(connection);
                                    self.reload_connection_counts();
                                } else {
                                    *self.connection_counts.entry(connection.start.clone()).or_default() += 1;
                                    *self.connection_counts.entry(connection.end.clone()).or_default() += 1;
                                    self.canvas_snapshot.connections.push(connection);
                                }
                                return true;
                            } else {
                                self.toasts
                                    .error("cant connect 2 gates in a loop! (not sure why)");
                            }
                        } else {
                            self.toasts
                                .error("cant connect an input directly to an output!");
                        }
                    }
                } else {
                    self.toasts.error("part reached max connections!");
                }
            }
        }
        false
    }
}
