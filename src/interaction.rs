use crate::connections::{Connection, draw_connection};
use crate::egui::{Context, Key, Painter, PointerButton, Rect, Vec2};
use crate::parts::PartData;
use crate::parts::PartType;
use crate::state::{AppState, InteractionState, Selection};
use crate::tools::{ConnectorData, Tool};

const FOCUS_KEYS: &[egui::Key] = &[
    egui::Key::Tab,
    egui::Key::Escape,
    egui::Key::ArrowUp,
    egui::Key::ArrowDown,
    egui::Key::ArrowLeft,
    egui::Key::ArrowRight,
];

fn get_input(ctx: &Context, key_opt: Option<Key>) -> bool {
    if let Some(key) = key_opt {
        if ctx.input(|i| i.key_pressed(key) && !i.modifiers.ctrl && !i.modifiers.alt) {
            return true;
        }
    }
    false
}

impl AppState {
    pub fn handle_input(&mut self, ctx: &Context, painter: &Painter, response: &egui::Response) {
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let in_canvas = response.hovered();
        let ctrl_held = ctx.input(|i| i.modifiers.ctrl);
        let shift_held = ctx.input(|i| i.modifiers.shift);

        if let Some(action) = &self.rebinding.clone() {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = event
                    {
                        if *key == Key::Escape {
                            self.config.keybinds.insert(action.clone(), None);
                            self.rebinding = None;
                        } else {
                            self.config.keybinds.insert(action.clone(), Some(*key));
                            self.rebinding = None;
                        }
                        self.config.save();
                    }
                }
            });
            return; // dont process normal keybinds while rebinding
        }

        // escape key
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.switch_to_tool(None);
        } else if ctx.wants_keyboard_input() {
            // a text field is focused, skip all keybinds below
        } else if get_input(ctx, self.config.keybinds.get("paint").copied().flatten()) {
            self.switch_to_tool(Some(Tool::Paint));
        } else if get_input(
            ctx,
            self.config.keybinds.get("connector").copied().flatten(),
        ) {
            self.switch_to_tool(Some(Tool::Connector(ConnectorData::NEW)));
        } else if get_input(
            ctx,
            self.config.keybinds.get("simulator").copied().flatten(),
        ) {
            self.switch_to_tool(Some(Tool::Simulator));
            self.start_simulation();
        } else if get_input(ctx, self.config.keybinds.get("exporter").copied().flatten()) {
            self.switch_to_tool(Some(Tool::Exporter(self.config.export_settings.clone())));
        } else if get_input(ctx, self.config.keybinds.get("and").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::And)));
        } else if get_input(ctx, self.config.keybinds.get("or").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Or)));
        } else if get_input(ctx, self.config.keybinds.get("xor").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Xor)));
        } else if get_input(ctx, self.config.keybinds.get("input").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Input)));
        } else if get_input(ctx, self.config.keybinds.get("timer").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Timer)));
        } else if get_input(ctx, self.config.keybinds.get("nand").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Nand)));
        } else if get_input(ctx, self.config.keybinds.get("nor").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Nor)));
        } else if get_input(ctx, self.config.keybinds.get("xnor").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Xnor)));
        } else if get_input(ctx, self.config.keybinds.get("output").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Output)));
        } else if get_input(ctx, self.config.keybinds.get("label").copied().flatten()) {
            self.switch_to_tool(Some(Tool::PlacePart(PartType::Label)));
        } else {
            for script in self.config.pinned_scripts.clone() {
                if get_input(
                    ctx,
                    self.config
                        .keybinds
                        .get(script.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                        .copied()
                        .flatten(),
                ) {
                    self.load_lua(script);
                }
            }
        }

        if (!matches!(
            self.active_tool,
            Some(Tool::Simulator) | Some(Tool::Exporter(_))
        )) && get_input(ctx, self.config.keybinds.get("rename").copied().flatten())
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
                    self.has_unsaved_changes = false;
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
                    self.has_unsaved_changes = false;
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
            let new_zoom = (old_zoom * factor).clamp(0.025, 6.0);

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
                        self.push_undo();
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
                            if self.add_connection(
                                Connection {
                                    start: start_port,
                                    end: *end_port,
                                },
                                true,
                            ) == false
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

    // spent ages trying to figure this out, eventually gave up and claude took 20 years to get this
    // working :/
    pub fn catch_ui_keys(&mut self, ctx: &Context) {
        if self.active_tool == Some(crate::tools::Tool::Simulator) && !ctx.wants_keyboard_input() {
            let pause_key = self
                .config
                .keybinds
                .get("simulator pause")
                .copied()
                .flatten();
            let tick_key = self
                .config
                .keybinds
                .get("simulator tick")
                .copied()
                .flatten();

            let mut do_pause = false;
            let mut do_tick = false;

            ctx.input_mut(|i| {
                if pause_key.is_some_and(|k| i.consume_key(egui::Modifiers::NONE, k)) {
                    do_pause = true;
                }
                if tick_key.is_some_and(|k| i.consume_key(egui::Modifiers::NONE, k)) {
                    do_tick = true;
                }
            });

            let consumed_a_focus_key = FOCUS_KEYS
                .iter()
                .any(|k| (do_pause && pause_key == Some(*k)) || (do_tick && tick_key == Some(*k)));
            if consumed_a_focus_key {
                ctx.memory_mut(|mem| mem.move_focus(egui::FocusDirection::None));
            }

            if do_pause {
                if let Some(sim_state) = &self.sim_state {
                    let mut state = sim_state.lock();
                    state.running = !state.running;
                }
            }
            if do_tick {
                if let Some(sim_state) = &self.sim_state {
                    let mut state = sim_state.lock();
                    state.step = true;
                }
            }
        }
    }
}
