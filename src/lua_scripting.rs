use crate::{
    colors::DEFAULT_GATE_COLOR,
    connections::Connection,
    parts::{Part, PartData, PartType, Port},
    state::AppState,
};
use egui::{Color32, Pos2};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use mlua::Lua;
use std::cell::RefCell;
use std::path::PathBuf;

pub struct LuaScript {
    pub path: Option<PathBuf>,
    pub data: String,
    pub output: String,
}

impl AppState {
    fn run_script(&mut self) {
        if let Some(mut lua_script) = self.lua_script.take() {
            lua_script.output.clear();
            let script_data = lua_script.data.clone();
            let project_folder = self.project_folder.clone();
            let lua = Lua::new();

            let resolve = |path: String| -> std::path::PathBuf {
                let p = std::path::PathBuf::from(&path);
                if p.is_relative() {
                    if let Some(ref base) = project_folder {
                        return base.join(p);
                    }
                }
                p
            };
            self.push_undo();
            let app_cell = RefCell::new(&mut *self);
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
                lua.globals().set(
                    "read_file",
                    scope.create_function(|_, path: String| {
                        std::fs::read_to_string(resolve(path)).map_err(mlua::Error::external)
                    })?,
                )?;

                lua.globals().set(
                    "read_bytes",
                    scope.create_function(|_, path: String| {
                        std::fs::read(resolve(path)).map_err(mlua::Error::external)
                    })?,
                )?;
                lua.globals().set(
                    "create_gate",
                    scope.create_function_mut(
                        |_, (gate_type, x, y, opts): (String, f32, f32, Option<mlua::Table>)| {
                            let color = if let Some(hex) =
                                opts.as_ref().and_then(|t| t.get::<String>("color").ok())
                            {
                                Color32::from_hex(&hex).map_err(|e| {
                                    mlua::Error::runtime(format!("invalid color: {:?}", e))
                                })?
                            } else {
                                DEFAULT_GATE_COLOR
                            };
                            let important = opts
                                .as_ref()
                                .and_then(|t| t.get::<bool>("important").ok())
                                .unwrap_or(false);
                            let part_type = match gate_type.to_lowercase().as_str() {
                                "and" => PartType::And,
                                "or" => PartType::Or,
                                "xor" => PartType::Xor,
                                "nand" => PartType::Nand,
                                "nor" => PartType::Nor,
                                "xnor" => PartType::Xnor,
                                _ => return Err(mlua::Error::runtime("invalid gate type")),
                            };
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(part_type, &mut **app, Pos2::new(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                if let PartData::Gate(data) = &mut part.part_data {
                                    data.important = important;
                                }
                            }
                            Ok(id)
                        },
                    )?,
                )?;
                lua.globals().set(
                    "create_timer",
                    scope.create_function_mut(
                        |_, (secs, ticks, x, y, opts): (u8, u8, f32, f32, Option<mlua::Table>)| {
                            let color = if let Some(hex) =
                                opts.as_ref().and_then(|t| t.get::<String>("color").ok())
                            {
                                Color32::from_hex(&hex).map_err(|e| {
                                    mlua::Error::runtime(format!("invalid color: {:?}", e))
                                })?
                            } else {
                                DEFAULT_GATE_COLOR
                            };
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(PartType::Timer, &mut **app, Pos2::new(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                if let PartData::Timer(data) = &mut part.part_data {
                                    data.secs = secs;
                                    data.ticks = ticks;
                                }
                            }
                            Ok(id)
                        },
                    )?,
                )?;
                lua.globals().set(
                    "add_connection",
                    scope.create_function_mut(|_, (from_id, to_id): (u64, u64)| {
                        let connection = Connection {
                            start: Port {
                                part: from_id,
                                input: false,
                                port_id: None,
                            },
                            end: Port {
                                part: to_id,
                                input: true,
                                port_id: None,
                            },
                        };
                        app_cell.borrow_mut().add_connection(connection);
                        Ok(())
                    })?,
                )?;
                lua.globals().set(
                    "get_part",
                    scope.create_function_mut(|_, (id): (u64)| {
                        if let Some(part) = app_cell.borrow().canvas_snapshot.parts.get(&id) {
                            Ok(())
                        } else {
                            Err(mlua::Error::runtime(format!("unable to find part {}!", id)))
                        }
                    })?,
                )?;
                lua.load(&script_data).exec()
            });
            drop(app_cell);

            lua_script.output = output;
            if let Err(e) = result {
                lua_script.output.push_str(&format!("[error] {}\n", e));
            }
            self.lua_script = Some(lua_script);
        }
    }

    pub fn draw_lua_script(&mut self, ctx: &egui::Context) {
        let mut open = self.lua_script.is_some();
        let mut load_path: Option<PathBuf> = None;
        let mut run = false;
        egui::Window::new("Lua Scripting")
            .open(&mut open)
            .default_width(600.0)
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
