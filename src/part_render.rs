use crate::AppState;
use crate::colors::{POWERED_COLOR, UNPOWERED_COLOR};
use crate::parts::{Gate, GateType, IO, Label, Module, Part, PartData, Switch, Timer};
use egui::{Color32, Context, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

pub const GATE_SIZE: Vec2 = Vec2::new(60.0, 60.0);

impl AppState {
    pub fn draw_parts(&self, painter: Painter) {
        for part in self.canvas_snapshot.parts.values() {
            match &part.part_data {
                PartData::Gate(gate) => Gate::draw(part, gate, &painter, self),
                _ => {}
            }
        }
    }
}

pub fn draw_gate_base(
    painter: &Painter,
    pos: Pos2,
    size: Vec2,
    color: Color32,
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
}

impl Gate {
    pub fn draw(part: &Part, gate: &Gate, painter: &Painter, app_state: &AppState) {
        draw_gate_base(
            painter,
            part.pos,
            GATE_SIZE,
            part.color,
            gate.powered,
            false,
            app_state,
        )
    }
}
