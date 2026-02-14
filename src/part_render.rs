use crate::colors::{POWERED_COLOR, UNPOWERED_COLOR};
use crate::parts::{
    GATE_SIZE, Gate, GateType, IO, Label, Module, PORT_SIZE, Part, PartData, Port, SWITCH_SIZE,
    Switch, Timer,
};
use crate::state::{AppState, Selection, path_to_string};
use eframe::epaint::PathShape;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, TextEdit, Ui, Vec2};

const ICON_HEIGHT: f32 = 20.0;
const ICON_WIDTH: f32 = ICON_HEIGHT * 1.5;
const ICON_Y_SHIFT: f32 = 10.0;

const CONNECTION_COUNT_SHIFT: Vec2 = Vec2::new(8.0, 0.0);
const CONNECTION_COUNT_SIZE: f32 = 10.0;

const CONNECTION_LABEL_SHIFT: Vec2 = Vec2::new(8.0, 7.0);
const CONNECTION_LABEL_SIZE: f32 = 7.0;

impl AppState {
    pub fn draw_parts(&self, painter: &Painter) {
        for part in self.canvas_snapshot.parts.values() {
            match &part.part_data {
                PartData::Gate(gate) => gate.draw(part, painter, self),
                PartData::Timer(timer) => timer.draw(part, painter, self),
                PartData::Module(module) => module.draw(part, painter, self),
                PartData::IO(io) => io.draw(part, painter, self),
                PartData::Switch(switch) => switch.draw(part, painter, self),
                PartData::Label(label) => label.draw(part, painter, self),
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
    ports: Vec<Port>,
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

    if resizable {
        let handle_size = 8.0 * app_state.zoom;
        let br = screen_rect.right_bottom();
        let grip_stroke = Stroke::new(1.5 * app_state.zoom, Color32::from_gray(160));
        painter.line_segment(
            [
                Pos2::new(br.x - handle_size * 0.3, br.y),
                Pos2::new(br.x, br.y - handle_size * 0.3),
            ],
            grip_stroke,
        );
        painter.line_segment(
            [
                Pos2::new(br.x - handle_size * 0.65, br.y),
                Pos2::new(br.x, br.y - handle_size * 0.65),
            ],
            grip_stroke,
        );
        painter.line_segment(
            [
                Pos2::new(br.x - handle_size, br.y),
                Pos2::new(br.x, br.y - handle_size),
            ],
            grip_stroke,
        );
    }

    for port in ports {
        if let Some(pos) = port.pos(app_state) {
            painter.circle_filled(
                app_state.world_to_screen(pos),
                PORT_SIZE * app_state.zoom,
                Color32::from_gray(150),
            );
            if let Some(connection_count) = app_state.connection_counts.get(&port) {
                if port.input {
                    painter.text(
                        app_state.world_to_screen(pos + CONNECTION_COUNT_SHIFT),
                        Align2::LEFT_CENTER,
                        connection_count,
                        FontId::new(
                            CONNECTION_COUNT_SIZE * app_state.zoom,
                            egui::FontFamily::Proportional,
                        ),
                        Color32::WHITE,
                    );
                } else {
                    painter.text(
                        app_state.world_to_screen(pos - CONNECTION_COUNT_SHIFT),
                        Align2::RIGHT_CENTER,
                        connection_count,
                        FontId::new(
                            CONNECTION_COUNT_SIZE * app_state.zoom,
                            egui::FontFamily::Proportional,
                        ),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

impl Gate {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        // skip rendering if off-screen
        let screen_rect = Rect::from_min_max(
            app_state.world_to_screen(part.pos),
            app_state.world_to_screen(part.pos + GATE_SIZE),
        );
        if !painter.clip_rect().intersects(screen_rect) {
            return;
        }

        //draw main base & outline
        draw_part_base(
            painter,
            part.pos,
            GATE_SIZE,
            if app_state.selection.contains(&Selection::Part(part.id)) {
                app_state.color_pallet.selection
            } else {
                part.color
            },
            part.label.clone(),
            13.0,
            self.powered,
            false,
            part.get_ports(),
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
                // D shaped body
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
                // OR/NOR/XOR/XNOR gate body, curved pointy shape )>
                let r = (ICON_HEIGHT / 2.0) * app_state.zoom;
                let segments = 16;
                let back_depth = r * 0.45;
                let right_x = center.x + (ICON_HEIGHT / 2.0) * app_state.zoom;

                let mut points = Vec::new();

                // back curve
                let arc_scale = 0.9;
                let start_angle = std::f32::consts::PI * (-0.5) * arc_scale;
                let end_angle = std::f32::consts::PI * 0.5 * arc_scale;

                let back_start = Pos2::new(left + back_depth * start_angle.cos(), top);
                let back_end = Pos2::new(left + back_depth * end_angle.cos(), bottom);

                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let angle = std::f32::consts::PI * (t - 0.5) * arc_scale;
                    points.push(Pos2::new(
                        left + back_depth * angle.cos(),
                        top + (bottom - top) * t,
                    ));
                }

                // bottom curve from back_end to right point
                for i in 1..=segments {
                    let t = i as f32 / segments as f32;
                    let x = back_end.x + (right_x - back_end.x) * t;
                    let y = back_end.y + (center.y - back_end.y) * t.powi(2);
                    points.push(Pos2::new(x, y));
                }

                // top curve from right point back to back_start
                for i in 1..segments {
                    let t = i as f32 / segments as f32;
                    let x = right_x + (back_start.x - right_x) * t;
                    let ease = t * (2.0 - t);
                    let y = center.y + (back_start.y - center.y) * ease;
                    points.push(Pos2::new(x, y));
                }

                painter.add(PathShape::closed_line(points, stroke));

                // xor/ xnor "shield" thing, like ))>
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

    pub fn draw_properties(&mut self, ui: &mut Ui, app_state: &mut AppState, label: &mut String) {
        egui::ComboBox::from_label("Type")
            .selected_text(self.gate_type.to_label())
            .show_ui(ui, |ui| {
                for lgt in GateType::TYPES {
                    if ui
                        .selectable_label(lgt.clone() == self.gate_type, lgt.to_label())
                        .clicked()
                    {
                        if lgt.clone() != self.gate_type {
                            app_state.push_undo();
                            if *label == self.gate_type.to_label() {
                                *label = lgt.to_label();
                            }
                            self.gate_type = lgt.clone();
                        }
                    }
                }
            });
    }
}

impl Timer {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        // skip rendering if off screen
        let screen_rect = Rect::from_min_max(
            app_state.world_to_screen(part.pos),
            app_state.world_to_screen(part.pos + GATE_SIZE),
        );
        if !painter.clip_rect().intersects(screen_rect) {
            return;
        }

        //draw main base & outline
        draw_part_base(
            painter,
            part.pos,
            GATE_SIZE,
            if app_state.selection.contains(&Selection::Part(part.id)) {
                app_state.color_pallet.selection
            } else {
                part.color
            },
            part.label.clone(),
            13.0,
            *self.buffer.last().unwrap_or(&false),
            false,
            part.get_ports(),
            app_state,
        );

        let stroke = Stroke::new(1.5 * app_state.zoom, Color32::WHITE);
        let center: Pos2 =
            app_state.world_to_screen(part.pos - Vec2::new(0.0, ICON_Y_SHIFT) + GATE_SIZE / 2.0);

        let top = center.y - (ICON_HEIGHT / 2.0 * app_state.zoom);
        let bottom = center.y + (ICON_HEIGHT / 2.0 * app_state.zoom);
        let half_width = (ICON_HEIGHT / 2.0 * 0.7) * app_state.zoom;

        // hourglass shape
        let top_left = Pos2::new(center.x - half_width, top);
        let top_right = Pos2::new(center.x + half_width, top);
        let bottom_left = Pos2::new(center.x - half_width, bottom);
        let bottom_right = Pos2::new(center.x + half_width, bottom);

        let points = vec![
            top_left,
            top_right,
            center,
            bottom_right,
            bottom_left,
            center,
            top_left,
        ];
        painter.add(PathShape::line(points, stroke));

        // Top and bottom caps
        painter.line_segment([top_left, top_right], stroke);
        painter.line_segment([bottom_left, bottom_right], stroke);
    }

    pub fn draw_properties(&mut self, ui: &mut Ui, app_state: &mut AppState) {
        let mut secs = self.secs;
        let mut ticks = self.ticks;

        ui.horizontal(|ui| {
            ui.label("Seconds:");
            ui.add(egui::DragValue::new(&mut secs).range(0..=59));
        });
        ui.horizontal(|ui| {
            ui.label("Ticks:");
            ui.add(egui::DragValue::new(&mut ticks).range(0..=40));
        });

        if secs != self.secs || ticks != self.ticks {
            app_state.push_undo();
            self.secs = secs;
            self.ticks = ticks;
        }

        let total_ticks = secs as u32 * 40 + ticks as u32;
        let total_secs = secs as f64 + ticks as f64 * 0.025;
        ui.label(format!("Total: {:.3}s ({}t)", total_secs, total_ticks));
    }
}

impl Module {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        // skip rendering if off-screen
        let screen_rect = Rect::from_min_max(
            app_state.world_to_screen(part.pos),
            app_state.world_to_screen(part.pos + GATE_SIZE),
        );
        if !painter.clip_rect().intersects(screen_rect) {
            return;
        }

        //draw main base & outline
        draw_part_base(
            painter,
            part.pos,
            self.size,
            if app_state.selection.contains(&Selection::Part(part.id)) {
                app_state.color_pallet.selection
            } else {
                part.color
            },
            part.label.clone(),
            0.0,
            false,
            true,
            part.get_ports(),
            app_state,
        );

        let ports = part.connections_pos_with_id();
        for (pos, _, port_id) in ports {
            if let Some(port) = port_id {
                if let Some(label) = self.inputs.get(&port) {
                    painter.text(
                        app_state.world_to_screen(Pos2::new(
                            pos.x + CONNECTION_LABEL_SHIFT.x,
                            pos.y + CONNECTION_LABEL_SHIFT.y,
                        )),
                        Align2::LEFT_CENTER,
                        label,
                        FontId::new(
                            CONNECTION_LABEL_SIZE * app_state.zoom,
                            egui::FontFamily::Proportional,
                        ),
                        Color32::WHITE,
                    );
                } else if let Some(label) = self.outputs.get(&port) {
                    painter.text(
                        app_state.world_to_screen(Pos2::new(
                            pos.x - CONNECTION_LABEL_SHIFT.x,
                            pos.y + CONNECTION_LABEL_SHIFT.y,
                        )),
                        Align2::RIGHT_CENTER,
                        label,
                        FontId::new(
                            CONNECTION_LABEL_SIZE * app_state.zoom,
                            egui::FontFamily::Proportional,
                        ),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
    pub fn draw_properties(&mut self, ui: &mut Ui, app_state: &mut AppState) {
        ui.label(format!(
            "File Path: {}",
            path_to_string(self.path.clone(), app_state.project_folder.clone())
        ));
        if ui.button("Change File").clicked() {
            app_state.push_undo();
            let file = rfd::FileDialog::new()
                .add_filter("SM Logic", &["sml"])
                .pick_file();
            if let Some(path) = file {
                self.path = path;
            }
            self.reload(app_state.project_folder.clone(), &mut app_state.toasts);
        }
        app_state.push_undo();
        if ui.button("Reload File").clicked() {
            self.reload(app_state.project_folder.clone(), &mut app_state.toasts);
        }
    }
}

impl IO {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        // skip rendering if off-screen
        let screen_rect = Rect::from_min_max(
            app_state.world_to_screen(part.pos),
            app_state.world_to_screen(part.pos + GATE_SIZE),
        );
        if !painter.clip_rect().intersects(screen_rect) {
            return;
        }

        //draw main base & outline
        draw_part_base(
            painter,
            part.pos,
            GATE_SIZE,
            if app_state.selection.contains(&Selection::Part(part.id)) {
                app_state.color_pallet.selection
            } else {
                part.color
            },
            part.label.clone(),
            0.0,
            false,
            false,
            part.get_ports(),
            app_state,
        );
    }
}

impl Switch {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        // skip rendering if off-screen
        let screen_rect = Rect::from_min_max(
            app_state.world_to_screen(part.pos),
            app_state.world_to_screen(part.pos + GATE_SIZE),
        );
        if !painter.clip_rect().intersects(screen_rect) {
            return;
        }

        //draw main base & outline
        draw_part_base(
            painter,
            part.pos,
            SWITCH_SIZE,
            if app_state.selection.contains(&Selection::Part(part.id)) {
                app_state.color_pallet.selection
            } else {
                part.color
            },
            part.label.clone(),
            0.0,
            self.powered,
            false,
            part.get_ports(),
            app_state,
        );
    }
}

impl Label {
    pub fn draw(&self, part: &Part, painter: &Painter, app_state: &AppState) {
        // skip rendering if off-screen
        let screen_rect = Rect::from_min_max(
            app_state.world_to_screen(part.pos),
            app_state.world_to_screen(part.pos + GATE_SIZE),
        );
        if !painter.clip_rect().intersects(screen_rect) {
            return;
        }

        //draw main base & outline
        draw_part_base(
            painter,
            part.pos,
            self.size,
            if app_state.selection.contains(&Selection::Part(part.id)) {
                app_state.color_pallet.selection
            } else {
                part.color
            },
            part.label.clone(),
            0.0,
            false,
            true,
            Vec::new(),
            app_state,
        );
    }
}

impl Part {
    pub fn draw_properties(&mut self, ui: &mut Ui, app_state: &mut AppState) {
        match &mut self.part_data {
            PartData::Gate(gate) => gate.draw_properties(ui, app_state, &mut self.label),
            PartData::Timer(timer) => timer.draw_properties(ui, app_state),
            PartData::Module(module) => module.draw_properties(ui, app_state),
            _ => {}
        }
        ui.horizontal(|ui| {
            if matches!(self.part_data, PartData::Label(_)) {
                ui.label("Text:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.label)
                        .desired_width(100.0)
                        .desired_rows(4),
                );
            } else {
                ui.label("Label:");
                ui.add(TextEdit::singleline(&mut self.label).desired_width(100.0));
            }
        });
    }
}
