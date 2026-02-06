use crate::egui::{Context, Key, PointerButton, Pos2, Rect, Vec2};
use crate::state::{AppState, InteractionState};
use crate::tools::Tool;

impl AppState {
    pub fn handle_input(&mut self, ctx: &Context, response: &egui::Response) {
        let canvas_rect = response.rect;
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let in_canvas = response.hovered();
        // escape key
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.active_tool = None;
            self.interaction_state = InteractionState::Idle;
            return;
        }

        if !in_canvas {
            return;
        }

        //scroll zoom
        let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.5 {
            let factor = 1.0 + scroll * 0.002;
            let old_zoom = self.zoom;
            let new_zoom = (old_zoom * factor).clamp(0.15, 6.0);

            if let Some(cursor_screen) = pointer_pos {
                // keep cursor in same real world position
                let cursor_world = self.screen_to_world(cursor_screen);
                self.zoom = new_zoom;
                self.pan_offset = Vec2::new(
                    cursor_world.x - cursor_screen.x / new_zoom,
                    cursor_world.y - cursor_screen.y / new_zoom,
                );
            } else {
                self.zoom = new_zoom;
            }
        }

        let Some(screen_pos) = pointer_pos else {
            return;
        };
        let world_pos = self.screen_to_world(screen_pos);

        match self.interaction_state {
            InteractionState::Idle => {
                self.handle_idle(ctx, screen_pos, world_pos);
            }
            InteractionState::Panning => {
                let delta = ctx.input(|i| i.pointer.delta());
                self.pan_offset -= delta / self.zoom;
                if ctx.input(|i| {
                    i.pointer.button_released(PointerButton::Middle)
                        || i.pointer.button_released(PointerButton::Secondary)
                }) {
                    self.interaction_state = InteractionState::Idle;
                }
            }
        }
    }

    fn handle_idle(&mut self, ctx: &Context, screen_pos: Pos2, world_pos: Pos2) {
        if ctx.input(|i| i.pointer.button_pressed(PointerButton::Middle))
            || ctx.input(|i| i.pointer.button_pressed(PointerButton::Secondary))
        {
            self.interaction_state = InteractionState::Panning;
            return;
        }
        if ctx.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
            let shift = ctx.input(|i| i.modifiers.shift);

            self.handle_tool(world_pos);
        }
    }
}
