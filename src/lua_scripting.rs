use crate::state::AppState;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use mlua::Lua;
use std::path::PathBuf;

pub struct LuaScript {
    pub path: Option<PathBuf>,
    pub data: String,
    pub output: String,
}

impl AppState {
    fn run_script(&mut self) {
        if let Some(lua_script) = &mut self.lua_script {
            lua_script.output.clear();
            let script_data = lua_script.data.clone();
            let lua = Lua::new();

            let mut output = String::new();
            let result = lua.scope(|scope| {
                lua.globals().set(
                    "print",
                    scope.create_function_mut(|_, msg: String| {
                        output.push_str(&msg);
                        output.push('\n');
                        Ok(())
                    })?,
                )?;
                lua.load(&script_data).exec()
            });

            lua_script.output = output;
            if let Err(e) = result {
                lua_script.output.push_str(&format!("[error] {}\n", e));
            }
        }
    }

    pub fn draw_lua_script(&mut self, ctx: &egui::Context) {
        let mut open = self.lua_script.is_some();
        let mut load_path: Option<PathBuf> = None;
        let mut run = false;
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
                        if ui.button("New script").clicked() {
                            lua_script = LuaScript {
                                path: None,
                                data: String::new(),
                                output: String::new(),
                            }
                        }
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
                    ui.horizontal(|ui| {
                    if ui.button("Run").clicked() { run = true; }
                    if ui.button("Docs").clicked() {
                        let _ = open::that("https://github.com/popcornman209/SMLogic/blob/main/github_resources/lua_scripting.md");
                    }});
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for line in lua_script.output.lines() {
                                if line.starts_with("[error]") {
                                    ui.colored_label(egui::Color32::RED, line);
                                } else {
                                    ui.monospace(line);
                                }
                            }
                        });
                    self.lua_script = Some(lua_script);
                }
            });
        if run {
            self.run_script();
        }
        if let Some(path) = load_path {
            self.load_lua(path);
        } else if !open {
            self.lua_script = None;
        }
    }

    pub fn load_lua(&mut self, path: PathBuf) {
        let contents = std::fs::read_to_string(&path);
        if let Ok(data) = contents {
            self.lua_script = Some(LuaScript {
                path: Some(path),
                data: data,
                output: String::new(),
            })
        } else if let Err(error) = contents {
            self.toasts
                .error(format!("failed to load lua file! {}", error));
        }
    }
}
