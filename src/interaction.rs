use crate::connections::{Connection, draw_connection};
use crate::egui::{Context, Key, Painter, PointerButton, Pos2, Rect, Vec2};
use crate::parts::PartData;
use crate::state::{AppState, InteractionState, Selection};

impl AppState {
    pub fn handle_input(&mut self, ctx: &Context, painter: &Painter, response: &egui::Response) {
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let in_canvas = response.hovered();
        let ctrl_held = ctx.input(|i| i.modifiers.ctrl);
        let shift_held = ctx.input(|i| i.modifiers.shift);

        // escape key
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.active_tool = None;
            self.interaction_state = InteractionState::Idle;
        }

        // backspace
        if (ctx.input(|i| i.key_pressed(Key::Backspace))
            | ctx.input(|i| i.key_pressed(Key::Delete)))
            && !ctx.wants_keyboard_input()
        {
            if !self.selection.is_empty() {
                self.push_undo();
                for selection in &self.selection {
                    if let Selection::Part(part_id) = selection {
                        self.canvas_snapshot.parts.remove(part_id);
                    }
                }
                let mut connections_to_remove: Vec<usize> = self
                    .selection
                    .iter()
                    .filter_map(|s| {
                        if let Selection::Connection(i) = s {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .collect();
                connections_to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for i in connections_to_remove {
                    self.canvas_snapshot.connections.remove(i);
                }

                self.selection.clear();
                self.reload_connection_counts();
            }
        }

        // undo/redo
        if ctx.input(|i| i.key_pressed(Key::Z)) {
            if ctrl_held && shift_held {
                self.redo()
            } else if ctrl_held {
                self.undo()
            }
        }

        // save/save as
        if ctx.input(|i| i.key_pressed(Key::S)) && ctrl_held {
            if shift_held {
                let mut dialog = rfd::FileDialog::new()
                    .add_filter("SM Logic", &["sml"])
                    .set_file_name("module.sml");
                if let Some(project_folder) = &self.project_folder {
                    dialog = dialog.set_directory(project_folder);
                }
                let file = dialog.save_file();
                if let Some(path) = file {
                    self.canvas_snapshot.save(path.clone());
                    self.current_module_path = Some(path.clone());
                    self.toasts.success(format!(
                        "Saved: {}",
                        if let Some(folder) = &self.project_folder {
                            path.strip_prefix(folder)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| path.to_string_lossy().to_string())
                        } else {
                            path.to_string_lossy().to_string()
                        }
                    ));
                }
            } else {
                if let Some(path) = self.current_module_path.clone() {
                    self.canvas_snapshot.save(path.clone());
                    self.toasts.success(format!(
                        "Saved: {}",
                        if let Some(folder) = &self.project_folder {
                            path.strip_prefix(folder)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| path.to_string_lossy().to_string())
                        } else {
                            path.to_string_lossy().to_string()
                        }
                    ));
                }
            }
        }

        // if mouse not in canvas do nothing else
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
                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Middle))
                    || ctx.input(|i| i.pointer.button_pressed(PointerButton::Secondary))
                {
                    self.interaction_state = InteractionState::Panning;
                    return;
                }

                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
                    let selected_port = self.port_at_pos(world_pos);
                    let selected_connection = self.connection_at_pos(world_pos);
                    let selected_part = self.part_at_pos(world_pos).map(|p| p.id);

                    if selected_port.is_some() {
                        self.connect_start = selected_port;
                        self.interaction_state = InteractionState::Connecting;
                        self.push_undo();
                    } else if let Some(connection) = selected_connection {
                        self.select_connection(connection, shift_held);
                    } else if let Some(part) = selected_part {
                        self.select_part(part, shift_held);
                        self.interaction_state = InteractionState::Dragging;
                        self.push_undo();
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
                        let rect: Rect = Rect::from_two_pos(start_pos, world_pos);
                        let parts = self.parts_in_rect(rect);
                        for part in parts {
                            self.selection.push(Selection::Part(part)); // select all parts in box
                        }
                        let connections = self.connections_in_rect(rect);
                        for connection in connections {
                            self.selection.push(Selection::Connection(connection))
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
                        for selection in self.selection.clone() {
                            if let Selection::Part(part_id) = selection {
                                if let Some(part) = self.canvas_snapshot.parts.get_mut(&part_id) {
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
                            draw_connection(self, world_pos, start_pos, painter, false);
                        } else {
                            draw_connection(self, start_pos, world_pos, painter, false);
                        }
                        if ctx.input(|i| i.pointer.button_released(PointerButton::Primary)) {
                            if let Some(port) = self.port_at_pos(world_pos) {
                                let (start_port, end_port) = if connect_start.input {
                                    (port, connect_start)
                                } else {
                                    (connect_start, port)
                                };
                                if start_port.input != end_port.input {
                                    let count =
                                        self.connection_counts.get(&end_port).copied().unwrap_or(0);
                                    if let Some(end_part) =
                                        self.canvas_snapshot.parts.get(&end_port.part)
                                    {
                                        if count < end_part.part_data.max_connections() {
                                            self.canvas_snapshot.connections.push(Connection {
                                                start: start_port,
                                                end: end_port,
                                                powered: false,
                                            });
                                            self.reload_connection_counts();
                                        }
                                    }
                                } else {
                                    self.undo_stack.pop();
                                }
                            } else {
                                self.undo_stack.pop();
                            }
                            self.interaction_state = InteractionState::Idle;
                        }
                    }
                }
            }
        }
    }
}
