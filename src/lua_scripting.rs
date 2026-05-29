use std::path::PathBuf;

use crate::state::AppState;
use eframe::egui::Ui;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};

pub struct LuaScript {
    pub path: Option<PathBuf>,
    pub data: String,
}

// fn run_script(script: &str, app: &mut AppState) -> Result<(), mlua::Error> {
//     let lua = Lua::new();
//
//     lua.globals().set(
//         "add_part",
//         lua.create_function(|_, (part_type, x, y): (String, f32, f32)| Ok(part_id))?,
//     )?;
//
//     lua.globals().set(
//         "add_connection",
//         lua.create_function(|_, (from, to): (u64, u64)| Ok(()))?,
//     )?;
//
//     lua.load(script).exec()
// }

impl AppState {
    pub fn draw_lua_script(&mut self, ctx: &egui::Context) {
        let mut open = self.lua_script.is_some();
        let mut load_path: Option<PathBuf> = None;
        egui::Window::new("Lua Scripting")
            .open(&mut open)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(self.color_pallet.base)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                if let Some(mut lua_script) = self.lua_script.take() {
                    ui.horizontal(|ui| {
                        if ui.button("Save as").clicked() {
                            let mut dialog = rfd::FileDialog::new()
                                .add_filter("lua", &["lua"])
                                .set_file_name("script.lua");
                            if let Some(project_folder) = &self.project_folder {
                                dialog = dialog.set_directory(project_folder);
                            }
                            let file = dialog.save_file();
                            if let Some(path) = file {
                                if let Err(error) = std::fs::write(&path, lua_script.data.clone()) {
                                    self.toasts.error(format!("failed to save! {}", error));
                                } else {
                                    lua_script.path = Some(path);
                                }
                            }
                        }
                        if let Some(path) = &lua_script.path {
                            if ui.button("Save").clicked() {
                                if let Err(error) = std::fs::write(path, lua_script.data.clone()) {
                                    self.toasts.error(format!("failed to save! {}", error));
                                }
                            }
                        }
                        if ui.button("Open").clicked() {
                            let file = rfd::FileDialog::new()
                                .add_filter("lua", &["lua"])
                                .pick_file();
                            load_path = file;
                        }
                    });
                    ui.heading("Lua Script");
                    CodeEditor::default()
                        .id_source("code editor")
                        .with_rows(0)
                        .with_fontsize(14.0)
                        .with_theme(ColorTheme::SONOKAI)
                        .with_syntax(Syntax::lua())
                        .with_numlines(true)
                        .show(ui, &mut lua_script.data);
                    self.lua_script = Some(lua_script);
                }
            });
        if let Some(path) = load_path {
            self.load_lua(path);
        } else if open == false {
            self.lua_script = None;
        }
    }

    pub fn load_lua(&mut self, path: PathBuf) {
        let contents = std::fs::read_to_string(&path);
        if let Ok(data) = contents {
            self.lua_script = Some(LuaScript {
                path: Some(path),
                data: data,
            })
        } else if let Err(error) = contents {
            self.toasts
                .error(format!("failed to load lua file! {}", error));
        }
    }
}
