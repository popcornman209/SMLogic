use crate::connections::{Connection, draw_connection};
use crate::egui::{Context, Key, Painter, PointerButton, Pos2, Rect, Vec2};
use crate::state::{AppState, InteractionState, Selection};

impl AppState {
    pub fn handle_input(&mut self, ctx: &Context, painter: &Painter, response: &egui::Response) {
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let in_canvas = response.hovered();
        // escape key
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.active_tool = None;
            self.interaction_state = InteractionState::Idle;
        }

        if ctx.input(|i| i.key_pressed(Key::Backspace)) {
            for selection in self.selection.clone() {
                match selection {
                    Selection::Part(part_id) => {
                        self.canvas_snapshot.parts.remove(&part_id);
                    }
                    Selection::Connection(connection) => {
                        self.canvas_snapshot.connections.remove(connection);
                    }
                };
            }
            self.selection.clear();
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
        let shift_held = ctx.input(|i| i.modifiers.shift);

        match self.interaction_state {
            InteractionState::Idle => {
                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Middle))
                    || ctx.input(|i| i.pointer.button_pressed(PointerButton::Secondary))
                {
                    self.interaction_state = InteractionState::Panning;
                    return;
                }

                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
                    let selected_port = self.port_at_pos(world_pos);
                    let selected_part = self.part_at_pos(world_pos).map(|p| p.id);

                    if selected_port.is_some() {
                        self.connect_start = selected_port;
                        self.interaction_state = InteractionState::Connecting;
                    } else if let Some(part) = selected_part {
                        self.select_part(part, shift_held);
                        self.interaction_state = InteractionState::Dragging;
                    } else if self.active_tool.is_none() {
                        self.box_select_start = Some(world_pos);
                        self.interaction_state = InteractionState::BoxSelecting;
                    } else {
                        self.handle_tool(world_pos, shift_held);
                    }
                }
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
            InteractionState::BoxSelecting => {
                if let Some(start_pos) = self.box_select_start {
                    self.draw_box_selection(painter, ctx);
                    if ctx.input(|i| i.pointer.button_released(PointerButton::Primary)) {
                        if !shift_held {
                            self.selection.clear();
                        }
                        let parts = self.parts_in_rect(Rect::from_two_pos(start_pos, world_pos));
                        for part in parts {
                            self.selection.push(Selection::Part(part)); // select all parts in box
                        }
                        self.interaction_state = InteractionState::Idle;
                    }
                }
            }
            InteractionState::Dragging => {
                let delta = ctx.input(|i| i.pointer.delta()) / self.zoom;

                for selection in &self.selection {
                    if let Selection::Part(part_id) = selection {
                        if let Some(part) = self.canvas_snapshot.parts.get_mut(part_id) {
                            part.pos += delta;
                        }
                    }
                }

                if ctx.input(|i| i.pointer.button_released(PointerButton::Primary)) {
                    if self.snap_to_grid {
                        for selection in &self.selection {
                            if let Selection::Part(part_id) = selection {
                                if let Some(part) = self.canvas_snapshot.parts.get_mut(part_id) {
                                    part.snap_pos();
                                }
                            }
                        }
                    }
                    self.interaction_state = InteractionState::Idle;
                }
            }
            InteractionState::Connecting => {
                if let Some(connect_start) = self.connect_start.clone() {
                    if let Some(start_pos) = connect_start.pos(self) {
                        if connect_start.input {
                            draw_connection(self, world_pos, start_pos, painter);
                        } else {
                            draw_connection(self, start_pos, world_pos, painter);
                        }
                        if ctx.input(|i| i.pointer.button_released(PointerButton::Primary)) {
                            if let Some(port) = self.port_at_pos(world_pos) {
                                if !connect_start.input && port.input {
                                    self.canvas_snapshot.connections.push(Connection {
                                        start: connect_start,
                                        end: port,
                                        powered: false,
                                    })
                                } else if connect_start.input && !port.input {
                                    self.canvas_snapshot.connections.push(Connection {
                                        start: port,
                                        end: connect_start,
                                        powered: false,
                                    })
                                }
                            }
                            self.interaction_state = InteractionState::Idle;
                        }
                    }
                }
            }
        }
    }
}
