use crate::{
    colors::DEFAULT_GATE_COLOR,
    connections::Connection,
    parts::{GATE_SIZE, GateType, Part, PartData, PartType, Port},
    state::{AppState, Selection},
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

impl PartType {
    fn gate_from_string(gate_type: String) -> mlua::Result<Self> {
        match gate_type.to_lowercase().as_str() {
            "and" => Ok(PartType::And),
            "or" => Ok(PartType::Or),
            "xor" => Ok(PartType::Xor),
            "nand" => Ok(PartType::Nand),
            "nor" => Ok(PartType::Nor),
            "xnor" => Ok(PartType::Xnor),
            _ => return Err(mlua::Error::runtime("invalid gate type")),
        }
    }
}

fn get_position(x: f32, y: f32) -> Pos2 {
    Pos2::new(x + GATE_SIZE.x / 2.0, y + GATE_SIZE.y / 2.0)
}

fn get_color(opts: &Option<mlua::Table>) -> mlua::Result<Color32> {
    return if let Some(hex) = opts.as_ref().and_then(|t| t.get::<String>("color").ok()) {
        Color32::from_hex(&hex).map_err(|e| mlua::Error::runtime(format!("invalid color: {:?}", e)))
    } else {
        Ok(DEFAULT_GATE_COLOR)
    };
}

fn set_opts(opts: &Option<mlua::Table>, part: &mut Part) -> mlua::Result<()> {
    let x_opt = opts.as_ref().and_then(|t| t.get::<f32>("x").ok());
    let y_opt = opts.as_ref().and_then(|t| t.get::<f32>("y").ok());
    let pos_opt = if let Some(x) = x_opt
        && let Some(y) = y_opt
    {
        Some(Pos2::new(x, y))
    } else {
        None
    };
    let color_opt = if let Some(hex) = opts.as_ref().and_then(|t| t.get::<String>("color").ok()) {
        Some(
            Color32::from_hex(&hex)
                .map_err(|e| mlua::Error::runtime(format!("invalid color: {:?}", e)))?,
        )
    } else {
        None
    };
    let label_opt = opts.as_ref().and_then(|t| t.get::<String>("label").ok());
    if let Some(pos) = pos_opt {
        part.pos = pos;
    }
    if let Some(label) = label_opt {
        part.label = label;
    }
    if let Some(color) = color_opt {
        part.color = color
    }
    Ok(())
}

fn get_part(part: &Part, lua: &Lua) -> mlua::Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("id", part.id)?;
    t.set("x", part.pos.x)?;
    t.set("y", part.pos.y)?;
    t.set("label", part.label.clone())?;
    t.set(
        "color",
        part.color
            .to_opaque()
            .to_hex()
            .strip_suffix("ff")
            .ok_or_else(|| mlua::Error::runtime("failed to convert color to hex"))?,
    )?;
    match &part.part_data {
        PartData::Gate(gate) => {
            t.set("type", gate.gate_type.to_label())?;
            t.set("important", gate.important)?;
        }
        PartData::Timer(timer) => {
            t.set("type", "timer")?;
            t.set("seconds", timer.secs)?;
            t.set("ticks", timer.ticks)?;
        }
        PartData::IO(io) => {
            t.set("type", "io")?;
            t.set("input", io.input)?;
        }
        PartData::Module(_module) => {
            t.set("type", "module")?;
        }
        PartData::Label(label) => {
            t.set("type", "label")?;
            t.set("xSize", label.size.x)?;
            t.set("ySize", label.size.y)?;
        }
    }
    Ok(t)
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
                    "list_dir",
                    scope.create_function_mut(|_, dir: String| {
                        let t = lua.create_table()?;
                        let entries =
                            std::fs::read_dir(resolve(dir)).map_err(mlua::Error::external)?;

                        for entry in entries {
                            t.set(
                                t.raw_len() + 1,
                                entry
                                    .map_err(mlua::Error::external)?
                                    .file_name()
                                    .to_string_lossy()
                                    .into_owned(),
                            )?;
                        }
                        Ok(t)
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
                            let color = get_color(&opts)?;
                            let important = opts
                                .as_ref()
                                .and_then(|t| t.get::<bool>("important").ok())
                                .unwrap_or(false);
                            let part_type = PartType::gate_from_string(gate_type)?;
                            let label = opts.as_ref().and_then(|t| t.get::<String>("label").ok());
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(part_type, &mut **app, get_position(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                if let Some(l) = label {
                                    part.label = l;
                                }
                                if let PartData::Gate(data) = &mut part.part_data {
                                    data.important = important;
                                }
                            }
                            Ok(id)
                        },
                    )?,
                )?;
                lua.globals().set(
                    "modify_gate",
                    scope.create_function_mut(|_, (id, opts): (u64, Option<mlua::Table>)| {
                        let important_opt =
                            opts.as_ref().and_then(|t| t.get::<bool>("important").ok());
                        let type_opt = opts.as_ref().and_then(|t| t.get::<String>("type").ok());
                        let mut app = app_cell.borrow_mut();
                        if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                            set_opts(&opts, part)?;
                            if let PartData::Gate(data) = &mut part.part_data {
                                if let Some(important) = important_opt {
                                    data.important = important;
                                }
                                if let Some(gate_type) = type_opt {
                                    data.gate_type = GateType::from_part_type(
                                        PartType::gate_from_string(gate_type)?,
                                    );
                                }
                            } else {
                                return Err(mlua::Error::runtime("not a gate!"));
                            }
                        } else {
                            return Err(mlua::Error::runtime("couldnt find part!"));
                        }
                        Ok(())
                    })?,
                )?;
                lua.globals().set(
                    "create_timer",
                    scope.create_function_mut(
                        |_, (secs, ticks, x, y, opts): (u8, u8, f32, f32, Option<mlua::Table>)| {
                            let color = get_color(&opts)?;
                            let label = opts.as_ref().and_then(|t| t.get::<String>("label").ok());
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(PartType::Timer, &mut **app, get_position(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                if let Some(l) = label {
                                    part.label = l;
                                }
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
                    "modify_timer",
                    scope.create_function_mut(|_, (id, opts): (u64, Option<mlua::Table>)| {
                        let secs_opt = opts.as_ref().and_then(|t| t.get::<u8>("seconds").ok());
                        let ticks_opt = opts.as_ref().and_then(|t| t.get::<u8>("ticks").ok());
                        let mut app = app_cell.borrow_mut();
                        if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                            set_opts(&opts, part)?;
                            if let PartData::Timer(data) = &mut part.part_data {
                                if let Some(secs) = secs_opt {
                                    data.secs = secs;
                                }
                                if let Some(ticks) = ticks_opt {
                                    data.ticks = ticks;
                                }
                            } else {
                                return Err(mlua::Error::runtime("not a timer!"));
                            }
                        } else {
                            return Err(mlua::Error::runtime("couldnt find part!"));
                        }
                        Ok(())
                    })?,
                )?;
                lua.globals().set(
                    "create_input",
                    scope.create_function_mut(
                        |_, (x, y, opts): (f32, f32, Option<mlua::Table>)| {
                            let color = get_color(&opts)?;
                            let label = opts.as_ref().and_then(|t| t.get::<String>("label").ok());
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(PartType::Input, &mut **app, get_position(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                if let Some(l) = label {
                                    part.label = l;
                                }
                            }
                            Ok(id)
                        },
                    )?,
                )?;
                lua.globals().set(
                    "create_output",
                    scope.create_function_mut(
                        |_, (x, y, opts): (f32, f32, Option<mlua::Table>)| {
                            let color = get_color(&opts)?;
                            let label = opts.as_ref().and_then(|t| t.get::<String>("label").ok());
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(PartType::Output, &mut **app, get_position(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                if let Some(l) = label {
                                    part.label = l;
                                }
                            }
                            Ok(id)
                        },
                    )?,
                )?;
                lua.globals().set(
                    "create_label",
                    scope.create_function_mut(
                        |_, (label, x, y, opts): (String, f32, f32, Option<mlua::Table>)| {
                            let color = get_color(&opts)?;
                            let mut app = app_cell.borrow_mut();
                            let id = Part::new(PartType::Label, &mut **app, get_position(x, y));
                            if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                                part.color = color;
                                part.label = label
                            }
                            Ok(id)
                        },
                    )?,
                )?;
                lua.globals().set(
                    "modify_other",
                    scope.create_function_mut(|_, (id, opts): (u64, Option<mlua::Table>)| {
                        let mut app = app_cell.borrow_mut();
                        if let Some(part) = app.canvas_snapshot.parts.get_mut(&id) {
                            set_opts(&opts, part)?;
                        } else {
                            return Err(mlua::Error::runtime("couldnt find part!"));
                        }
                        Ok(())
                    })?,
                )?;
                lua.globals().set(
                    "remove_part",
                    scope.create_function_mut(|_, id: u64| {
                        app_cell.borrow_mut().canvas_snapshot.parts.remove(&id);
                        Ok(())
                    })?,
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
                    "remove_connection",
                    scope.create_function_mut(|_, (start, end): (u64, u64)| {
                        app_cell
                            .borrow_mut()
                            .canvas_snapshot
                            .connections
                            .retain(|c| !(c.start.part == start && c.end.part == end));
                        Ok(())
                    })?,
                )?;
                lua.globals().set(
                    "get_part",
                    scope.create_function_mut(|_, id: u64| {
                        if let Some(part) = app_cell.borrow().canvas_snapshot.parts.get(&id) {
                            get_part(part, &lua)
                        } else {
                            Err(mlua::Error::runtime(format!("unable to find part {}!", id)))
                        }
                    })?,
                )?;
                lua.globals().set(
                    "get_canvas",
                    scope.create_function_mut(|_, ()| {
                        let canvas_snapshot = app_cell.borrow().canvas_snapshot.clone();
                        let parts = lua.create_table()?;
                        for (id, part) in canvas_snapshot.parts {
                            parts.set(id, get_part(&part, &lua)?)?;
                        }
                        let connections = lua.create_table()?;
                        for (i, connection) in canvas_snapshot.connections.iter().enumerate() {
                            let t = lua.create_table()?;
                            t.set("start", connection.start.part)?;
                            t.set("end", connection.end.part)?;
                            connections.set(i + 1, t)?;
                        }
                        let t = lua.create_table()?;
                        t.set("parts", parts)?;
                        t.set("connections", connections)?;
                        Ok(t)
                    })?,
                )?;
                lua.globals().set(
                    "get_selection",
                    scope.create_function_mut(|_, ()| {
                        let canvas_snapshot = app_cell.borrow().canvas_snapshot.clone();
                        let selection = app_cell.borrow().selection.clone();
                        let parts = lua.create_table()?;
                        let connections = lua.create_table()?;
                        for select in selection.iter() {
                            if let Selection::Part(id) = select {
                                if let Some(part) = canvas_snapshot.parts.get(id) {
                                    parts.set(parts.raw_len() + 1, get_part(part, &lua)?)?;
                                }
                            } else if let Selection::Connection(id) = select {
                                if let Some(connection) = canvas_snapshot.connections.get(*id) {
                                    let t = lua.create_table()?;
                                    t.set("start", connection.start.part)?;
                                    t.set("end", connection.end.part)?;
                                    connections.set(connections.raw_len() + 1, t)?;
                                }
                            }
                        }
                        let t = lua.create_table()?;
                        t.set("parts", parts)?;
                        t.set("connections", connections)?;
                        Ok(t)
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
                        if let Some(path) = &lua_script.path {
                            let mut pinned = self.config.pinned_scripts.contains(&path);
                            if ui.checkbox(&mut pinned, "Pin script").changed() {
                                if pinned {
                                    if self.config.pinned_scripts.len() < 10 {
                                        self.config.pinned_scripts.push(path.clone());
                                        self.config.save();
                                    } else {
                                        self.toasts.error("Max scripts pinned!");
                                    }
                                } else {
                                    self.config.pinned_scripts.retain(|x| *x != *path);
                                    self.config.save();
                                }
                                self.config.normalize_keybinds();
                            }
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
