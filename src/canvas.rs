use eframe::egui::{self, Color32, Painter, Pos2, Rect, Sense, Stroke, Ui};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::colors::ColorPallet;
use crate::connections::draw_connection;
use crate::exporter::get_bp_folder;
use crate::lua_scripting::LuaScript;
use crate::parts::PartType;
use crate::state::{AppState, CanvasSnapshot, InteractionState, Selection, path_to_string};
use crate::tools::{Tool, tool_label};

const BASE_KEYBINDS: &[&str] = &[
    "and",
    "or",
    "xor",
    "nand",
    "nor",
    "xnor",
    "timer",
    "label",
    "input",
    "output",
    "paint",
    "connector",
    "simulator",
    "exporter",
    "simulator pause",
    "simulator tick",
    "rename",
];

impl AppState {
    pub fn draw_canvas(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
    ) -> (egui::Response, egui::Painter) {
        let available = ui.available_size();
        let (response, painter) =
            ui.allocate_painter(available, Sense::click_and_drag() | Sense::hover());
        let canvas_rect = response.rect;

        painter.rect_filled(canvas_rect, 0.0, self.color_pallet.grid);

        if self.show_grid {
            self.draw_grid(&painter, canvas_rect);
        }
        self.draw_parts(&painter);
        if !self.hide_connections {
            if self.draw_connections(&painter) {
                for i in (0..self.canvas_snapshot.connections.len()).rev() {
                    let remove = {
                        let connection = &self.canvas_snapshot.connections[i];
                        connection.start.pos(self).is_none() || connection.end.pos(self).is_none()
                    };
                    if remove {
                        self.canvas_snapshot.connections.remove(i);
                    }
                }
                self.reload_connection_counts();
            }
        }
        if self.selection.len() == 1 {
            if let Some(Selection::Part(part_id)) = self.selection.get(0) {
                for connection in &self.canvas_snapshot.connections {
                    if (&connection.start.part == part_id) | (&connection.end.part == part_id) {
                        let start_pos = connection.start.pos(self);
                        let end_pos = connection.end.pos(self);
                        if let (Some(start), Some(end)) = (start_pos, end_pos) {
                            draw_connection(self, start, end, &painter, true, false);
                        }
                    }
                }
            }
        }

        if let Some(Tool::Connector(mut connector_data)) = self.active_tool.clone() {
            self.draw_selected_connections(connector_data.clone(), &painter);
            if connector_data.status == String::new() {
                connector_data.connection_preview = connector_data.calculate_connections(self);
                if connector_data.status == String::new() {
                    connector_data.status = "ok".to_string();
                }
            }
            if connector_data.previewing {
                for connection in &connector_data.connection_preview {
                    let start_pos = connection.start.pos(self);
                    let end_pos = connection.end.pos(self);
                    if let (Some(start), Some(end)) = (start_pos, end_pos) {
                        draw_connection(self, start, end, &painter, true, false);
                    }
                }
            }
            self.active_tool = Some(Tool::Connector(connector_data));
        }

        if self.show_fps {
            self.draw_fps(ctx);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
        }

        if self.sim_state.is_some() {
            ctx.request_repaint();
        }

        self.toasts.show(ctx);
        (response, painter)
    }

    pub fn draw_grid(&self, painter: &Painter, canvas_rect: Rect) {
        let grid_spacing = 20.0_f32;
        let grid_stroke = Stroke::new(0.5, self.color_pallet.grid_lines);

        let world_min = self.screen_to_world(canvas_rect.min);
        let world_max = self.screen_to_world(canvas_rect.max);

        let start_x = (world_min.x / grid_spacing).floor() as i64;
        let end_x = (world_max.x / grid_spacing).ceil() as i64;
        for ix in start_x..=end_x {
            let wx = ix as f32 * grid_spacing;
            let sx = self.world_to_screen(Pos2::new(wx, 0.0)).x;
            painter.line_segment(
                [
                    Pos2::new(sx, canvas_rect.min.y),
                    Pos2::new(sx, canvas_rect.max.y),
                ],
                grid_stroke,
            );
        }
        let start_y = (world_min.y / grid_spacing).floor() as i64;
        let end_y = (world_max.y / grid_spacing).ceil() as i64;
        for iy in start_y..=end_y {
            let wy = iy as f32 * grid_spacing;
            let sy = self.world_to_screen(Pos2::new(0.0, wy)).y;
            painter.line_segment(
                [
                    Pos2::new(canvas_rect.min.x, sy),
                    Pos2::new(canvas_rect.max.x, sy),
                ],
                grid_stroke,
            );
        }
    }

    pub fn draw_box_selection(&self, painter: &Painter, ctx: &egui::Context) {
        if let InteractionState::BoxSelecting(selection) = self.interaction_state {
            let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
            if let Some(pointer) = pointer_pos {
                let mut color = self.color_pallet.selection;
                color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);
                painter.rect_filled(
                    Rect::from_two_pos(self.world_to_screen(selection), pointer),
                    0.0,
                    color,
                );
            }
        }
    }

    pub fn draw_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("toolbar")
            .resizable(false)
            .exact_width(160.0)
            .frame(
                egui::Frame::new()
                    .fill(self.color_pallet.base)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                //self.color_pallet.ui_apply(ui);
                // main parts
                ui.heading("Parts");
                ui.separator();
                for part in PartType::MAIN_PARTS {
                    // loop through all main parts
                    let selected = self.active_tool == Some(Tool::PlacePart(part.clone())); // if its
                    // selected
                    let label = format!("{}", part.label()); // label, will add keybind later
                    if ui.selectable_label(selected, &label).clicked() {
                        // if it was clicked
                        if selected {
                            self.active_tool = None; // go back to selecting if toggled off
                        } else {
                            self.active_tool = Some(Tool::PlacePart(part.clone())); // switch to part if
                            // selected
                        }
                        if self.active_tool == Some(Tool::Simulator) {
                            self.end_simulation();
                        }
                    }
                }

                // io parts
                ui.heading("I/O Parts");
                ui.separator();
                for part in PartType::IO_PARTS {
                    // loop through all main parts
                    let selected = self.active_tool == Some(Tool::PlacePart(part.clone()));
                    let label = format!("{}", part.label()); // label, will add keybind later
                    if ui.selectable_label(selected, &label).clicked() {
                        //if clicked
                        if selected {
                            self.active_tool = None;
                        } else {
                            self.active_tool = Some(Tool::PlacePart(part.clone()));
                        }
                        if self.active_tool == Some(Tool::Simulator) {
                            self.end_simulation();
                        }
                    }
                }

                // tools
                ui.heading("Tools");
                ui.separator();
                for tool in Tool::TOOLS {
                    let selected = match (&self.active_tool, tool) {
                        (Some(a), Some(b)) => {
                            std::mem::discriminant(a) == std::mem::discriminant(b)
                        }
                        (None, None) => true,
                        _ => false,
                    };
                    let label = format!("{}", tool_label(&tool)); // TODO add keybind
                    if ui.selectable_label(selected, &label).clicked() {
                        if self.active_tool == Some(Tool::Simulator) {
                            self.end_simulation();
                        }
                        if selected {
                            self.active_tool = None;
                            if self.active_tool == Some(Tool::Simulator) {
                                self.end_simulation();
                            }
                        } else {
                            self.active_tool = if matches!(tool, Some(Tool::Exporter(_))) {
                                Some(Tool::Exporter(self.config.export_settings.clone()))
                            } else {
                                tool.clone()
                            };
                            if matches!(tool, Some(Tool::Connector(_)) | Some(Tool::Simulator)) {
                                self.selection.clear();
                            }
                            if matches!(tool, Some(Tool::Simulator)) {
                                self.start_simulation();
                            } else {
                                if self.active_tool == Some(Tool::Simulator) {
                                    self.end_simulation();
                                }
                            }
                        }
                    }
                }

                // lua scripting
                ui.heading("Lua Scripting");
                ui.separator();
                if ui.button("Open editor").clicked() {
                    if self.lua_script.is_none() {
                        self.lua_script = Some(LuaScript {
                            path: None,
                            data: String::new(),
                            output: String::new(),
                        });
                    }
                }
                for script in self.config.pinned_scripts.clone() {
                    let selected = self
                        .lua_script
                        .as_ref()
                        .is_some_and(|s| s.path.as_ref() == Some(&script));
                    if ui
                        .selectable_label(
                            selected,
                            script.file_name().and_then(|n| n.to_str()).unwrap_or("err"),
                        )
                        .clicked()
                    {
                        self.load_lua(script.clone());
                    }
                }

                // settings button
                ui.separator();
                if ui.button("Settings").clicked() {
                    self.settings_open = !self.settings_open;
                }
                ui.checkbox(&mut self.hide_connections, "Hide connections");

                // tool settings
                self.draw_sidebar_tool_properties(ui);

                // properties
                if self.selection.len() == 1 {
                    if let Selection::Part(part_id) = self.selection[0] {
                        if let Some(mut part) = self.canvas_snapshot.parts.remove(&part_id) {
                            ui.heading("Properties");
                            ui.separator();
                            part.draw_properties(ui, self);
                            self.canvas_snapshot.parts.insert(part_id, part);
                        }
                    }
                }
            });
    }

    pub fn draw_footer(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("module_browser")
            .resizable(false)
            .min_height(64.0)
            .show(ctx, |ui| {
                // Top row: project info + actions
                ui.horizontal(|ui| {
                    if let Some(folder) = &self.project_folder {
                        ui.label(format!(
                            "Project: {}",
                            folder
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| folder.to_string_lossy().to_string())
                        ));
                    } else {
                        ui.label("No project folder");
                    }
                    if ui.button("Change...").clicked() {
                        let folder = rfd::FileDialog::new().pick_folder();
                        if let Some(folder_option) = folder.clone() {
                            self.project_folder = folder;
                            self.project_sub_folder = None;
                            self.reload_project_folder();
                            self.config.last_project = Some(folder_option);
                            self.config.save();
                        }
                    }
                    if let Some(project_folder) = self.project_folder.clone() {
                        let active_folder = if let Some(sub_folder) = &self.project_sub_folder {
                            project_folder.join(sub_folder)
                        } else {
                            project_folder
                        };
                        if ui
                            .button("Open folder")
                            .on_hover_text("Opens folder in file explorer")
                            .clicked()
                        {
                            let _ = open::that(&active_folder);
                        }
                    }

                    ui.separator();
                    if ui.button("Clear Canvas").clicked() {
                        if self.are_you_sure() {
                            self.canvas_snapshot = CanvasSnapshot {
                                connections: Vec::new(),
                                parts: HashMap::new(),
                                next_id: 0,
                            };
                            self.connection_counts.clear();
                            self.current_module_path = None;
                            self.toasts.success("Cleared canvas");
                            self.end_simulation();
                        }
                    }
                    if ui.button("Save").clicked() {
                        let file = if self.current_module_path.is_none() {
                            let mut dialog = rfd::FileDialog::new()
                                .add_filter("SM Logic", &["sml"])
                                .set_file_name("module.sml");
                            if let Some(project_folder) = &self.project_folder {
                                dialog = dialog.set_directory(project_folder);
                            }
                            dialog.save_file()
                        } else {
                            self.current_module_path.clone()
                        };
                        if let Some(path) = file {
                            if let Err(e) = self.canvas_snapshot.save(path.clone()) {
                                self.toasts.error(format!("Failed to save: {}", e));
                            } else {
                                self.has_unsaved_changes = false;
                                self.current_module_path = Some(path.clone());
                                self.toasts.success(format!(
                                    "Saved: {}",
                                    path_to_string(path, self.project_folder.clone())
                                ));
                            }
                        }
                    }
                    if ui.button("Save As").clicked() {
                        let mut dialog = rfd::FileDialog::new()
                            .add_filter("SM Logic", &["sml"])
                            .set_file_name("module.sml");
                        if let Some(project_folder) = &self.project_folder {
                            dialog = dialog.set_directory(project_folder);
                        }
                        if let Some(path) = dialog.save_file() {
                            if let Err(e) = self.canvas_snapshot.save(path.clone()) {
                                self.toasts.error(format!("Failed to save: {}", e));
                            } else {
                                self.has_unsaved_changes = false;
                                self.current_module_path = Some(path.clone());
                                self.toasts.success(format!(
                                    "Saved: {}",
                                    path_to_string(path, self.project_folder.clone())
                                ));
                            }
                        }
                    }
                    if ui.button("Import").clicked() {
                        let file = rfd::FileDialog::new()
                            .add_filter("SM Logic", &["sml"])
                            .pick_file();
                        if let Some(path) = file {
                            self.active_tool = Some(Tool::PlacePart(PartType::Module(path)));
                        }
                    }
                    if ui.button("Open").clicked() && self.are_you_sure() {
                        let file = rfd::FileDialog::new()
                            .add_filter("SM Logic", &["sml"])
                            .pick_file();
                        if let Some(path) = file {
                            self.open_file(path);
                            self.end_simulation();
                        }
                    }

                    if let Some(path) = self.current_module_path.clone() {
                        ui.separator();
                        ui.label(format!(
                            "Editing: {}",
                            path_to_string(path, self.project_folder.clone())
                        ));
                    }
                });
                ui.separator();

                if self.project_folder.is_none() {
                    ui.label("Set a project folder to browse modules.");
                } else if self.current_folder_files.is_empty() && self.project_sub_folder.is_none()
                {
                    ui.label("No modules found. Save a module to see it here.");
                } else {
                    if let Some(path) = &self.project_sub_folder {
                        if let Some(project_path) = &self.project_folder {
                            ui.label(
                                path.strip_prefix(project_path)
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|_| path.to_string_lossy().to_string()),
                            );
                        }
                    }

                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // ".." button to go up
                            if let Some(sub_folder) = self.project_sub_folder.as_mut() {
                                if ui.button("../").clicked() {
                                    sub_folder.pop();
                                    self.reload_project_folder();
                                }
                            }
                            if let Some(project_folder) = self.project_folder.clone() {
                                for path in self.current_folder_files.clone() {
                                    let mut label = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                                    if path.is_dir() {
                                        label.push('/');
                                    }

                                    let active = self.active_tool
                                        == Some(Tool::PlacePart(PartType::Module(path.clone())));
                                    if ui.selectable_label(active, label).clicked() {
                                        if path.is_dir() {
                                            self.project_sub_folder = Some(
                                                path.strip_prefix(&project_folder)
                                                    .expect("path not under project folder")
                                                    .to_path_buf(),
                                            );
                                            self.reload_project_folder();
                                        } else if path.is_file() {
                                            if path.extension().is_some_and(|ext| ext == "sml") {
                                                if active && self.are_you_sure() {
                                                    self.open_file(path);
                                                    self.active_tool = None;
                                                    self.end_simulation();
                                                } else {
                                                    self.active_tool = Some(Tool::PlacePart(
                                                        PartType::Module(path.clone()),
                                                    ));
                                                }
                                            } else if path
                                                .extension()
                                                .is_some_and(|ext| ext == "lua")
                                            {
                                                self.load_lua(path);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    });
                }
            });
    }

    pub fn draw_fps(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        let idle = dt > 0.1;
        let fps = if idle { 0 } else { (1.0 / dt).round() as u16 };

        ctx.request_repaint_after(std::time::Duration::from_millis(if idle {
            1000
        } else {
            150
        }));

        egui::Area::new(egui::Id::new("fps_overlay"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
            .show(ctx, |ui: &mut egui::Ui| {
                ui.set_min_width(55.0);
                ui.label(format!("{} FPS", fps));
            });
    }

    pub fn draw_settings(&mut self, ctx: &egui::Context) {
        let mut open = self.settings_open;
        egui::Window::new("Settings")
            .open(&mut open)
            .resizable(false)
            .default_width(300.0)
            .frame(
                egui::Frame::new()
                    .fill(self.color_pallet.base)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                //self.color_pallet.ui_apply(ui);
                ui.heading("General");
                ui.separator();
                if ui
                    .checkbox(
                        &mut self.config.middle_click_deletes,
                        "Middle click deletes",
                    )
                    .changed()
                {
                    self.config.save();
                }
                if ui.checkbox(&mut self.show_grid, "Grid").changed() {
                    self.config.show_grid = self.show_grid;
                    self.config.save();
                }
                if ui
                    .checkbox(&mut self.snap_to_grid, "Snap to Grid")
                    .changed()
                {
                    self.config.snap_to_grid = self.snap_to_grid;
                    self.config.save();
                }
                if ui
                    .checkbox(&mut self.show_connection_count, "Show Connection Count")
                    .changed()
                {
                    self.config.show_connection_count = self.show_connection_count;
                    self.config.save();
                }
                if ui
                    .checkbox(&mut self.round_connections, "Round connection corners")
                    .changed()
                {
                    self.config.round_connections = self.round_connections;
                    self.config.save();
                }
                if ui.checkbox(&mut self.show_fps, "Show FPS").changed() {
                    self.config.show_fps = self.show_fps;
                    self.config.save();
                }

                //color scheme
                let current_colorpallet = self.color_pallet.label.clone();
                egui::ComboBox::from_label("Type")
                    .selected_text(current_colorpallet.clone())
                    .show_ui(ui, |ui| {
                        for clrplt in ColorPallet::COLOR_SCHEMES {
                            if ui
                                .selectable_label(*clrplt == current_colorpallet, *clrplt)
                                .clicked()
                            {
                                let new_pallet = match *clrplt {
                                    "Custom" => ColorPallet {
                                        label: Cow::Borrowed("Custom"),
                                        ..self.color_pallet.clone()
                                    },
                                    _ => ColorPallet::from_label(clrplt),
                                };
                                self.color_pallet = new_pallet.clone();
                                self.config.color_pallet = new_pallet.clone();
                                self.config.save();
                                self.color_pallet.apply_theme(ctx);
                            }
                        }
                    });
                // custom colorscheme options
                if current_colorpallet == "Custom" {
                    ui.collapsing("Custom colors", |ui| {
                        let mut changed = false;

                        macro_rules! color_picker {
                            ($label:expr, $color:expr) => {
                                ui.horizontal(|ui| {
                                    ui.label($label);
                                    let mut color = $color;
                                    if ui.color_edit_button_srgba(&mut color).changed() {
                                        $color = color;
                                        changed = true;
                                    }
                                });
                            };
                        }

                        color_picker!("Grid", self.color_pallet.grid);
                        color_picker!("Grid Lines", self.color_pallet.grid_lines);
                        color_picker!("Text", self.color_pallet.text);
                        color_picker!("Base", self.color_pallet.base);
                        color_picker!("Button", self.color_pallet.button);
                        color_picker!("Button Hover", self.color_pallet.button_hover);
                        color_picker!("Button Pushed", self.color_pallet.button_pushed);
                        color_picker!("Selection", self.color_pallet.selection);
                        color_picker!("Selection Text", self.color_pallet.selection_text);

                        if changed {
                            self.config.color_pallet = self.color_pallet.clone();
                            self.config.save();
                            self.color_pallet.apply_theme(ctx);
                        }
                    });
                }
                ui.heading("Exporting");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Blueprint Folder:");
                    let path_text = self
                        .bp_folder
                        .as_ref()
                        .map(|p| {
                            let s = p.to_string_lossy();
                            if s.len() > 40 {
                                format!("...{}", &s[s.len() - 40..])
                            } else {
                                s.to_string()
                            }
                        })
                        .unwrap_or_else(|| "Not found".to_string());
                    ui.label(path_text);
                });
                ui.horizontal(|ui| {
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.bp_folder = Some(path.clone());
                            self.config.bp_folder = Some(path);
                            self.config.save();
                        }
                    }
                    if ui.button("Auto-detect").clicked() {
                        self.bp_folder = get_bp_folder();
                        self.config.bp_folder = self.bp_folder.clone();
                        self.config.save();
                    }
                });
                ui.collapsing("Keybinds", |ui| {
                    let keybinds: Vec<&str> = BASE_KEYBINDS
                        .iter()
                        .copied()
                        .chain(
                            self.config
                                .pinned_scripts
                                .iter()
                                .filter_map(|p| p.file_name()?.to_str()),
                        )
                        .collect(); // ai told me how to do this idk how people learn this shit
                    for bind in keybinds {
                        ui.horizontal(|ui| {
                            ui.label(format!("{bind}:"));
                            if ui
                                .button(if self.rebinding.as_deref() == Some(bind) {
                                    "...".to_string()
                                } else {
                                    self.config
                                        .keybinds
                                        .get(bind)
                                        .copied()
                                        .flatten()
                                        .map(|k| format!("{:?}", k))
                                        .unwrap_or_else(|| "None".to_string())
                                })
                                .clicked()
                            {
                                self.rebinding = Some(bind.to_string());
                            }
                        });
                    }
                });
            });
        self.settings_open = open;
    }

    pub fn are_you_sure(&self) -> bool {
        if !self.has_unsaved_changes {
            return true;
        }
        rfd::MessageDialog::new()
            .set_title("Unsaved Changes")
            .set_description("You have unsaved changes. Continue anyway?")
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
    }
}
