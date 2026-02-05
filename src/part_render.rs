use crate::colors::{POWERED_COLOR, UNPOWERED_COLOR};
use crate::parts::{Gate, GateType, IO, Label, Module, Part, Switch, Timer};
use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

pub fn draw_gate_base(
    painter: &Painter,
    pos: Pos2,
    size: Vec2,
    color: Color32,
    powered: bool,
    resizable: bool,
    zoom: f32,
) {
    let rounding = 6.0 * zoom;
    let screen_rect: Rect = Rect::from_min_max(pos, pos + size);

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
    let border_width = 2.5 * zoom;
    painter.rect_stroke(
        screen_rect,
        rounding,
        Stroke::new(border_width, color),
        StrokeKind::Inside,
    );
}
