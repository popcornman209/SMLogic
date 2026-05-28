use crate::connections::{Connection, draw_connection};
use crate::egui::{Context, Key, Painter, PointerButton, Rect, Vec2};
use crate::parts::PartData;
use crate::parts::PartType;
use crate::state::{AppState, InteractionState, Selection};
use crate::tools::{ConnectorData, Tool};

impl AppState {
    pub fn handle_input(&mut self, ctx: &Context, painter: &Painter, response: &egui::Response) {
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let in_canvas = response.hovered();
        let ctrl_held = ctx.input(|i| i.modifiers.ctrl);
        let shift_held = ctx.input(|i| i.modifiers.shift);

        // escape key
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.switch_to_tool(None);
        } else if ctx.wants_keyboard_input() {
            // a text field is focused, skip all keybinds below
        } else if ctx.input(|i| i.key_pressed(Key::Num1)) {
            self.switch_to_tool(Some(Tool::Paint));
        } else if ctx.input(|i| i.key_pressed(Key::Num2)) {
            self.switch_to_tool(Some(Tool::Connector(ConnectorData::NEW)));
        } else if ctx.input(|i| i.key_pressed(Key::Num3)) {
            self.switch_to_tool(Some(Tool::Simulator));
            self.start_simulation();
        } else if ctx.input(|i| i.key_pressed(Key::Num4)) {
            self.switch_to_tool(Some(Tool::Exporter(self.config.export_settings.clone())));
        } else if ctx.input(|i| i.key_pressed(Key::Q)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::And)));
        } else if ctx.input(|i| i.key_pressed(Key::W)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Or)));
        } else if ctx.input(|i| i.key_pressed(Key::E)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Xor)));
        } else if ctx.input(|i| i.key_pressed(Key::R)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Input)));
        } else if ctx.input(|i| i.key_pressed(Key::T)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Timer)));
        } else if ctx.input(|i| i.key_pressed(Key::A)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Nand)));
        } else if ctx.input(|i| i.key_pressed(Key::S)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Nor)));
        } else if ctx.input(|i| i.key_pressed(Key::D)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Xnor)));
        } else if ctx.input(|i| i.key_pressed(Key::F)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Output)));
        } else if ctx.input(|i| i.key_pressed(Key::G)) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Label)));
        }

        if self.active_tool == Some(Tool::Simulator) {
            if ctx.input(|i| i.key_pressed(Key::Space)) {
                if let Some(sim_state) = &self.sim_state {
                    let mut state = sim_state.lock();
                    state.running = !state.running;
                }
            } else if ctx.input(|i| i.key_pressed(Key::Tab)) {
                if let Some(sim_state) = &self.sim_state {
                    let mut state = sim_state.lock();
                    state.step = true;
                }
            }
        } else if (!matches!(self.active_tool, Some(Tool::Exporter(_))))
            && ctx.input(|i| i.key_pressed(Key::Tab))
        {
            if self.selection.len() == 1 {
                self.request_rename = true;
            }
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

        match &self.interaction_state {
            InteractionState::Idle => {
                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Secondary)) {
                    self.interaction_state = InteractionState::Panning;
                    return;
                }
                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Middle)) {
                    if let Some(part_id) = self.part_at_pos(world_pos).map(|p| p.id) {
                        self.push_undo();
                        self.canvas_snapshot.parts.remove(&part_id);
                        self.reload_connection_counts();
                    } else if let Some(connection) = self.connection_at_pos(world_pos) {
                        self.push_undo();
                        self.canvas_snapshot.connections.remove(connection);
                        self.reload_connection_counts();
                    }
                }

                if ctx.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
                    let selected_resize = self.resize_at_pos(world_pos);
                    let selected_port = self.port_at_pos(world_pos);
                    let selected_connection = self.connection_at_pos(world_pos);
                    let selected_part = self.part_at_pos(world_pos).map(|p| p.id);

                    if let Some(port) = selected_port
                        && self.active_tool != Some(Tool::Simulator)
                    {
                        // connecting
                        if let Some(Tool::Connector(ref mut connector_data)) = self.active_tool {
                            connector_data.toggle_select_port(port);
                        } else {
                            self.interaction_state = InteractionState::Connecting(port);
                        }
                    } else if let Some(part) = selected_resize {
                        // resizing
                        self.interaction_state = InteractionState::Resizing(part.id);
                    } else if let Some(connection) = selected_connection
                        && self.active_tool != Some(Tool::Simulator)
                    {
                        // selected wire
                        self.select_connection(connection, shift_held);
                    } else if let Some(part) = selected_part
                        && !matches!(
                            self.active_tool,
                            Some(Tool::Simulator) | Some(Tool::Connector(_))
                        )
                    {
                        // selected part
                        self.select_part(part, shift_held);
                        self.push_undo();
                        if self.active_tool == Some(Tool::Paint) {
                            self.handle_tool(world_pos, shift_held);
                        } else {
                            self.interaction_state = InteractionState::Dragging;
                        }
                    } else if matches!(
                        self.active_tool,
                        None | Some(Tool::Paint) | Some(Tool::Connector(_))
                    ) {
                        // clicked on nothing & the tool uses box selection in some way
                        self.interaction_state = InteractionState::BoxSelecting(world_pos);
                    } else {
                        // is using tool
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
            InteractionState::BoxSelecting(start_pos) => {
                self.draw_box_selection(painter, ctx);
                if ctx.input(|i| i.pointer.button_released(PointerButton::Primary)) {
                    let rect: Rect = Rect::from_two_pos(start_pos.clone(), world_pos);
                    let ports = self.ports_in_rect(rect);
                    if let Some(Tool::Connector(ref mut connector_data)) = self.active_tool {
                        for port in ports {
                            connector_data.toggle_select_port(port);
                        }
                    } else {
                        if !shift_held {
                            self.selection.clear();
                        }

                        let parts = self.parts_in_rect(rect);
                        for part in parts {
                            self.select_part(part, true) // select all parts in box
                        }
                        let connections = self.connections_in_rect(rect);
                        for connection in connections {
                            self.select_connection(connection, true)
                        }
                    }
                    self.interaction_state = InteractionState::Idle;
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
            InteractionState::Resizing(part_id) => {
                let delta = ctx.input(|i| i.pointer.delta()) / self.zoom;

                if let Some(part) = self.canvas_snapshot.parts.get_mut(&part_id) {
                    match &mut part.part_data {
                        PartData::Module(module) => module.size += delta,
                        PartData::Label(label) => label.size += delta,
                        _ => {}
                    }
                }
                if ctx.input(|i| i.pointer.button_released(PointerButton::Primary)) {
                    if let Some(part) = self.canvas_snapshot.parts.get_mut(&part_id) {
                        if self.snap_to_grid {
                            match &mut part.part_data {
                                PartData::Module(module) => {
                                    module.size = Vec2::new(
                                        (module.size.x / 20.0).round() * 20.0,
                                        (module.size.y / 20.0).round() * 20.0,
                                    )
                                }
                                PartData::Label(label) => {
                                    label.size = Vec2::new(
                                        (label.size.x / 20.0).round() * 20.0,
                                        (label.size.y / 20.0).round() * 20.0,
                                    )
                                }
                                _ => {}
                            }
                        }
                        match &mut part.part_data {
                            PartData::Module(module) => {
                                if module.size.x < module.min_size.x {
                                    module.size.x = module.min_size.x
                                };
                                if module.size.y < module.min_size.y {
                                    module.size.y = module.min_size.y
                                };
                            }
                            _ => {}
                        }
                    }
                    self.interaction_state = InteractionState::Idle;
                }
            }
            InteractionState::Connecting(connect_start) => {
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
                                (connect_start.clone(), &port)
                            };
                            if self.add_connection(Connection {
                                start: start_port,
                                end: *end_port,
                            }) == false
                            {
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

    fn switch_to_tool(&mut self, tool: Option<Tool>) {
        if self.active_tool == Some(Tool::Simulator) {
            self.end_simulation();
        }
        self.active_tool = tool;
        self.interaction_state = InteractionState::Idle;
    }
}
