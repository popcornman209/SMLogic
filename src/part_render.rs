use crate::AppState;
use crate::colors::{POWERED_COLOR, UNPOWERED_COLOR};
use crate::parts::{GATE_SIZE, Gate, GateType, IO, Label, Module, Part, PartData, Switch, Timer};
use eframe::epaint::PathShape;
use egui::{Color32, Context, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

const ICON_HEIGHT: f32 = 20.0;
const ICON_WIDTH: f32 = ICON_HEIGHT * 1.5;
const ICON_Y_SHIFT: f32 = 10.0;

impl AppState {
    pub fn draw_parts(&self, painter: Painter) {
        for part in self.canvas_snapshot.parts.values() {
            match &part.part_data {
                PartData::Gate(gate) => gate.draw(part, &painter, self),
                _ => {}
            }
        }
    }
}

pub fn draw_part_base(
    painter: &Painter,
    pos: Pos2,
    size: Vec2,
    color: Color32,
    label: String,
    label_offset: f32,
    powered: bool,
    resizable: bool,
    app_state: &AppState,
) {
    let rounding = 6.0 * app_state.zoom;
    let screen_rect: Rect = Rect::from_min_max(
        app_state.world_to_screen(pos),
        app_state.world_to_screen(pos + size),
    );

    // 1. Dark interior
    painter.rect_filled(
        screen_rect,
        rounding,
        if powered {
            POWERED_COLOR
        } else {
            UNPOWERED_COLOR
        },
    );

    // 2. Colored border (or white when selected)
    let border_width = 2.5 * app_state.zoom;
    painter.rect_stroke(
        screen_rect,
        rounding,
        Stroke::new(border_width, color),
        StrokeKind::Inside,
    );

    // 3. Label text
    let text_pos = Pos2::new(
        screen_rect.center().x,
        screen_rect.center().y + label_offset * app_state.zoom,
    );
    painter.text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        &label,
        egui::FontId::proportional(12.0 * app_state.zoom),
        Color32::WHITE,
    );
}

impl Gate {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        draw_part_base(
            painter,
            part.pos,
            GATE_SIZE,
            part.color,
            part.label.clone(),
            13.0,
            self.powered,
            false,
            app_state,
        );
        let stroke = Stroke::new(1.5 * app_state.zoom, Color32::WHITE);
        let mut center: Pos2 =
            app_state.world_to_screen(part.pos - Vec2::new(0.0, ICON_Y_SHIFT) + GATE_SIZE / 2.0);

        // "not" bubble circle thing
        if matches!(
            self.gate_type,
            GateType::Nand | GateType::Nor | GateType::Xnor
        ) {
            painter.circle_stroke(
                Pos2::new(
                    center.x + ((ICON_WIDTH / 2.0 - 1.5) * app_state.zoom),
                    center.y,
                ),
                2.5 * app_state.zoom,
                stroke,
            );
        } else {
            center.x += 3.0 * app_state.zoom;
        }

        let top = center.y - (ICON_HEIGHT / 2.0 * app_state.zoom);
        let bottom = center.y + (ICON_HEIGHT / 2.0 * app_state.zoom);
        let left = center.x - (ICON_WIDTH / 2.0 * app_state.zoom);

        match self.gate_type {
            GateType::And | GateType::Nand => {
                // D-shape body
                let mut points = vec![Pos2::new(left, top), Pos2::new(center.x, top)];

                let segments = 16;
                for i in 0..=segments {
                    let angle = -std::f32::consts::FRAC_PI_2
                        + std::f32::consts::PI * (i as f32 / segments as f32);
                    points.push(Pos2::new(
                        center.x + (ICON_HEIGHT / 2.0 * angle.cos() * app_state.zoom),
                        center.y + (ICON_HEIGHT / 2.0 * angle.sin() * app_state.zoom),
                    ));
                }
                points.push(Pos2::new(left, bottom));
                painter.add(PathShape::closed_line(points, stroke));
            }
            _ => {
                // OR/NOR/XOR/XNOR gate body - curved shape with pointed front
                let r = (ICON_HEIGHT / 2.0) * app_state.zoom;
                let segments = 16;
                let back_depth = r * 0.45;
                let right_x = center.x + (ICON_HEIGHT / 2.0) * app_state.zoom;

                let mut points = Vec::new();

                // Back curve - shield shape but scaled to back_depth
                let arc_scale = 0.9;
                let start_angle = std::f32::consts::PI * (-0.5) * arc_scale;
                let end_angle = std::f32::consts::PI * 0.5 * arc_scale;

                let back_start = Pos2::new(left + back_depth * start_angle.cos(), top);
                let back_end = Pos2::new(left + back_depth * end_angle.cos(), bottom);

                // Back curve - y linear from top to bottom, x uses cos for shield shape
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let angle = std::f32::consts::PI * (t - 0.5) * arc_scale;
                    points.push(Pos2::new(
                        left + back_depth * angle.cos(),
                        top + (bottom - top) * t,
                    ));
                }

                // Bottom curve from back_end to right point
                for i in 1..=segments {
                    let t = i as f32 / segments as f32;
                    let x = back_end.x + (right_x - back_end.x) * t;
                    let y = back_end.y + (center.y - back_end.y) * t.powi(2);
                    points.push(Pos2::new(x, y));
                }

                // Top curve from right point back to back_start
                for i in 1..segments {
                    let t = i as f32 / segments as f32;
                    let x = right_x + (back_start.x - right_x) * t;
                    let ease = t * (2.0 - t);
                    let y = center.y + (back_start.y - center.y) * ease;
                    points.push(Pos2::new(x, y));
                }

                painter.add(PathShape::closed_line(points, stroke));

                if matches!(self.gate_type, GateType::Xor | GateType::Xnor) {
                    let r = (ICON_HEIGHT / 2.0) * app_state.zoom * 0.9;
                    let segments = 12;
                    let offset = r * 0.25;

                    let mut shield = Vec::new();
                    for i in 0..=segments {
                        let t = i as f32 / segments as f32;
                        let angle = std::f32::consts::PI * (t - 0.5) * 0.9;
                        shield.push(Pos2::new(
                            left - 2.0 * app_state.zoom + r * angle.cos() * 0.6 - offset,
                            center.y + r * angle.sin(),
                        ));
                    }

                    painter.add(PathShape::line(shield, stroke));
                }
            }
        }
    }
}
