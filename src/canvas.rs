use eframe::egui::{self, Color32, Painter, Pos2, Rect, Sense, Stroke, Ui};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::colors::ColorPallet;
use crate::parts::PartType;
use crate::state::{AppState, CanvasSnapshot};
use crate::tools::{Tool, tool_label};

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
        self.draw_connections(&painter);

        if self.show_fps {
            self.draw_fps(ctx);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

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
        if let Some(selection) = self.box_select_start {
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
                    }
                }

                // tools
                ui.heading("Tools");
                ui.separator();
                for tool in Tool::TOOLS {
                    let selected = self.active_tool == tool.clone();
                    let label = format!("{}", tool_label(&tool)); // TODO add keybind
                    if ui.selectable_label(selected, &label).clicked() {
                        if selected {
                            self.active_tool = None;
                        } else {
                            self.active_tool = tool.clone();
                        }
                    }
                }

                // settings button
                ui.separator();
                if ui.button("Settings").clicked() {
                    self.settings_open = !self.settings_open;
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
                        if folder.is_some() {
                            self.project_folder = folder;
                        }
                    }

                    ui.separator();
                    if ui.button("New Module").clicked() {
                        self.canvas_snapshot = CanvasSnapshot {
                            connections: Vec::new(),
                            parts: HashMap::new(),
                            next_id: 0,
                        };
                    }
                    if let Some(path) = self.current_module_path.clone() {
                        if ui.button("Save").clicked() {
                            self.canvas_snapshot.save(path);
                        }
                    }
                    if ui.button("Save As...").clicked() {
                        let mut dialog = rfd::FileDialog::new()
                            .add_filter("SM Logic", &["sml"])
                            .set_file_name("module.sml");
                        if let Some(project_folder) = &self.project_folder {
                            dialog = dialog.set_directory(project_folder);
                        }
                        let file = dialog.save_file();
                        if let Some(path) = file {
                            self.canvas_snapshot.save(path);
                        }
                    }
                    if ui.button("Import...").clicked() {
                        let mut dialog = rfd::FileDialog::new().add_filter("SM Logic", &["sml"]);
                        if let Some(project_folder) = &self.project_folder {
                            dialog = dialog.set_directory(project_folder);
                        }
                        let file = dialog.pick_file();
                        if let Some(path) = file {
                            self.active_tool = Some(Tool::PlacePart(PartType::Module(path)));
                        }
                    }

                    if let Some(path) = &self.current_module_path {
                        ui.separator();
                        let display_path = if let Some(folder) = &self.project_folder {
                            path.strip_prefix(folder)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| path.to_string_lossy().to_string())
                        } else {
                            path.to_string_lossy().to_string()
                        };
                        ui.label(format!("Editing: {}", display_path));
                    }
                })
            });
    }

    pub fn draw_fps(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.unstable_dt);
        let idle = dt > 0.1;
        let fps = if idle { 0 } else { (1.0 / dt).round() as u16 };

        // ctx.request_repaint_after(std::time::Duration::from_millis(if idle {
        //     500
        // } else {
        //     150
        // }));

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
                if ui.checkbox(&mut self.show_arrows, "Wire Arrows").changed() {
                    self.config.show_arrows = self.show_arrows;
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
                    ui.heading("Custom Colors");
                    ui.separator();

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
                    }
                }
            });
        self.settings_open = open;
    }
}
