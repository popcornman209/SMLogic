use crate::colors::SM_PALETTE;
use crate::connections::Connection;
use crate::parts::{Part, PartType, Port};
use crate::state::{AppState, Selection};
use eframe::egui::Pos2;
use egui::{Stroke, Ui, Vec2};
use std::time::{Duration, Instant};

const PAINT_CELL_SIZE: Vec2 = egui::vec2(20.0, 20.0);

#[derive(Clone, PartialEq)]
pub enum ConnectorMode {
    AllToAll,
    OneToOneSel,
    OneToOnePos,
}
impl ConnectorMode {
    pub const MODES: &[Self] = &[Self::AllToAll, Self::OneToOneSel, Self::OneToOnePos];

    pub fn to_label(&self) -> &'static str {
        match self {
            ConnectorMode::AllToAll => "All to all",
            ConnectorMode::OneToOneSel => "One to one (selection order)",
            ConnectorMode::OneToOnePos => "One to one (by pos)",
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ConnectorSelection {
    All,
    Inputs,
    Outputs,
}
impl ConnectorSelection {
    pub const MODES: &[Self] = &[Self::All, Self::Inputs, Self::Outputs];
    pub fn to_label(&self) -> &'static str {
        match self {
            ConnectorSelection::All => "All",
            ConnectorSelection::Inputs => "Inputs",
            ConnectorSelection::Outputs => "outputs",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ConnectorData {
    pub selected_ports: Vec<Port>,
    pub mode: ConnectorMode,
    pub selecting: ConnectorSelection,
    pub previewing: bool,
    pub inputs: usize,
    pub outputs: usize,
    pub total: usize,
    pub connection_preview: Vec<Connection>,
    pub status: String,
}
impl ConnectorData {
    pub fn calculate_totals(&mut self) {
        self.inputs = 0;
        self.outputs = 0;
        for port in self.selected_ports.clone() {
            if port.input {
                self.inputs += 1;
            } else {
                self.outputs += 1;
            }
        }
        self.total = self.inputs + self.outputs;
    }

    pub fn toggle_select_port(&mut self, port: Port) {
        if self.selecting == ConnectorSelection::All
            || ((self.selecting == ConnectorSelection::Inputs) == port.input)
        {
            if let Some(pos) = self.selected_ports.iter().position(|x| *x == port) {
                self.selected_ports.remove(pos);
            } else {
                self.selected_ports.push(port);
            }
            self.calculate_totals();
            self.status = String::new();
        }
    }

    pub fn calculate_connections(&mut self, app_state: &AppState) -> Vec<Connection> {
        self.calculate_totals(); // just incase something changed

        if self.inputs == 0 || self.outputs == 0 {
            self.status = "no selection".to_string();
            return Vec::new();
        }

        let (input_ports, output_ports): (Vec<Port>, Vec<Port>) =
            self.selected_ports.iter().partition(|p| p.input); // seperate out the selection

        let mut output: Vec<Connection> = Vec::new();

        match self.mode {
            ConnectorMode::AllToAll => {
                for input_port in &input_ports {
                    for output_port in &output_ports {
                        output.push(Connection {
                            start: *output_port,
                            end: *input_port,
                        })
                    }
                }
            }
            ConnectorMode::OneToOneSel => {
                if self.inputs == self.outputs {
                    for (input_port, output_port) in input_ports.iter().zip(output_ports.iter()) {
                        output.push(Connection {
                            start: *output_port,
                            end: *input_port,
                        })
                    }
                } else {
                    self.status = "not an equal amount of inputs and outputs!".to_string()
                }
            }
            ConnectorMode::OneToOnePos => {
                if self.inputs == self.outputs {
                    let mut inputs = input_ports.clone();
                    let mut outputs = output_ports.clone();
                    sort_ports_by_position(&mut inputs, app_state);
                    sort_ports_by_position(&mut outputs, app_state);
                    for (input_port, output_port) in inputs.iter().zip(outputs.iter()) {
                        output.push(Connection {
                            start: *output_port,
                            end: *input_port,
                        })
                    }
                } else {
                    self.status = "not an equal amount of inputs and outputs!".to_string()
                }
            }
        }
        output
    }
}

//current tool being used
#[derive(Clone, PartialEq)]
pub enum Tool {
    PlacePart(PartType),
    Paint,
    Connector(ConnectorData),
    Simulator,
}

impl Tool {
    pub const TOOLS: &[Option<Tool>] = &[
        None,
        Some(Tool::Paint),
        Some(Tool::Connector(ConnectorData {
            selected_ports: Vec::new(),
            mode: ConnectorMode::AllToAll,
            selecting: ConnectorSelection::All,
            previewing: false,
            inputs: 0,
            outputs: 0,
            total: 0,
            connection_preview: Vec::new(),
            status: String::new(),
        })),
        Some(Tool::Simulator),
    ];
}

impl AppState {
    pub fn draw_sidebar_tool_properties(&mut self, ui: &mut Ui) {
        let mut connect = false;
        match &mut self.active_tool {
            Some(Tool::Paint) => {
                ui.separator();
                ui.heading("Paint Tool");
                egui::Grid::new("palette_grid")
                    .spacing(egui::vec2(4.0, 4.0))
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        for row in SM_PALETTE.iter() {
                            for color in row.iter().rev() {
                                let (rect, response) =
                                    ui.allocate_exact_size(PAINT_CELL_SIZE, egui::Sense::click());

                                ui.painter().rect_filled(rect, 2.0, *color);

                                if self.current_paint_color == *color {
                                    ui.painter().rect_stroke(
                                        rect,
                                        2.0,
                                        Stroke::new(2.0, egui::Color32::WHITE),
                                        egui::StrokeKind::Outside,
                                    );
                                }

                                if response.clicked() {
                                    self.current_paint_color = *color;
                                }
                            }
                            ui.end_row();
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Custom: ");
                    let rgb = self.current_paint_color.to_srgba_unmultiplied();
                    let mut rgb3 = [rgb[0], rgb[1], rgb[2]];
                    if ui.color_edit_button_srgb(&mut rgb3).changed() {
                        self.current_paint_color =
                            egui::Color32::from_rgb(rgb3[0], rgb3[1], rgb3[2]);
                    }
                });
            }
            Some(Tool::Connector(connector_data)) => {
                ui.separator();
                ui.heading("Connector Tool");
                ui.horizontal(|ui| {
                    // mode selector combo box
                    ui.label("Mode: ");
                    egui::ComboBox::from_id_salt("connector_mode_combo")
                        .width(10.0)
                        .selected_text(
                            connector_data
                                .mode
                                .to_label()
                                .chars()
                                .take(10)
                                .collect::<String>(),
                        )
                        .show_ui(ui, |ui| {
                            for mode in ConnectorMode::MODES {
                                if ui
                                    .selectable_label(&connector_data.mode == mode, mode.to_label())
                                    .clicked()
                                {
                                    connector_data.mode = mode.clone();
                                    connector_data.status = String::new();
                                }
                            }
                        })
                });
                ui.horizontal(|ui| {
                    // selecting combo box
                    ui.label("Select: ");
                    egui::ComboBox::from_id_salt("connector_selecting_combo")
                        .width(10.0)
                        .selected_text(
                            connector_data
                                .selecting
                                .to_label()
                                .chars()
                                .take(10)
                                .collect::<String>(),
                        )
                        .show_ui(ui, |ui| {
                            for selection in ConnectorSelection::MODES {
                                if ui
                                    .selectable_label(
                                        &connector_data.selecting == selection,
                                        selection.to_label(),
                                    )
                                    .clicked()
                                {
                                    connector_data.selecting = selection.clone()
                                }
                            }
                        })
                });

                ui.label(format!("inputs: {}", connector_data.inputs));
                ui.label(format!("outputs: {}", connector_data.outputs));
                ui.label(format!("total: {}", connector_data.total));
                ui.horizontal(|ui| {
                    ui.label("Preview: ");
                    ui.checkbox(&mut connector_data.previewing, "");
                });
                ui.label(format!("status: {}", connector_data.status));
                if ui
                    .add_enabled(
                        connector_data.status == "ok".to_string(),
                        egui::Button::new("Connect!"),
                    )
                    .clicked()
                {
                    connect = true;
                }
            }
            Some(Tool::Simulator) => {
                ui.separator();
                ui.heading("Connector Tool");
                if let Some(sim_snapshot) = &self.sim_snapshot {
                    let (running, tick, target_spt, part_count) = {
                        let snapshot = sim_snapshot.lock();
                        self.sim_state_outputs_snapshot = Some(snapshot.outputs.clone());
                        (
                            snapshot.running,
                            snapshot.tick,
                            snapshot.target_spt,
                            snapshot.outputs.len(),
                        )
                    };

                    let mut new_running = running;
                    let mut new_step = false;
                    let mut new_target_spt = target_spt;
                    let mut mutations = false;

                    if ui.button(if running { "Stop" } else { "Start" }).clicked() {
                        new_running = !running;
                        mutations = true;
                    }
                    if !running {
                        if ui.button("Tick Step").clicked() {
                            new_step = true;
                            mutations = true;
                        }
                    }
                    let mut limit_tps = target_spt.is_some();
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut limit_tps, "Limit TPS").changed() {
                            new_target_spt = if limit_tps {
                                Some(Duration::from_secs_f32(1.0 / 40.0))
                            } else {
                                None
                            };
                            mutations = true;
                        }
                        if let Some(spt) = target_spt {
                            let mut tps = 1.0 / spt.as_secs_f32();
                            if ui
                                .add(egui::DragValue::new(&mut tps).range(1.0..=100000.0))
                                .changed()
                            {
                                new_target_spt = Some(Duration::from_secs_f32(1.0 / tps));
                                mutations = true;
                            }
                        }
                    });

                    ui.label(format!("Tick: {}", format_with_commas(tick)));
                    if self.last_tps_check.elapsed().as_secs_f64() >= 0.5 {
                        self.current_tps = (tick - self.last_tick_count) as f64
                            / self.last_tps_check.elapsed().as_secs_f64();
                        self.last_tick_count = tick;
                        self.last_tps_check = Instant::now();
                    }
                    ui.label(format!(
                        "TPS: {}",
                        format_with_commas(self.current_tps as u64)
                    ));
                    ui.label(format!("Parts: {}", part_count));

                    // lock variables again if something has to be changed
                    if mutations {
                        if let Some(sim_state) = &self.sim_state {
                            let mut state = sim_state.lock();
                            state.running = new_running;
                            if new_step {
                                state.step = true;
                            }
                            state.target_spt = new_target_spt;
                        }
                    }
                }
            }
            _ => {}
        }
        if connect {
            if let Some(Tool::Connector(connector_data)) = &mut self.active_tool {
                let new_connections: Vec<Connection> = connector_data
                    .connection_preview
                    .iter()
                    .filter(|x| !self.canvas_snapshot.connections.contains(x))
                    .cloned()
                    .collect();
                connector_data.selected_ports.clear();
                connector_data.status = String::new();
                self.push_undo();
                for connection in new_connections {
                    self.add_connection(connection);
                }
            }
        }
    }
}

pub fn tool_label(tool: &Option<Tool>) -> &'static str {
    match tool {
        None => "Select",
        Some(Tool::Paint) => "Paint Tool",
        Some(Tool::Connector(_)) => "Connnector",
        Some(Tool::Simulator) => "Simulator",
        _ => "???",
    }
}

impl AppState {
    pub fn handle_tool(&mut self, world_pos: Pos2, shift_held: bool) {
        match self.active_tool.clone() {
            None | Some(Tool::Connector(_)) => {}
            Some(Tool::PlacePart(part_type)) => {
                self.push_undo();
                let part_id = Part::new(part_type, self, world_pos);
                self.reload_connection_counts();
                self.select_part(part_id, shift_held);
            }
            Some(Tool::Paint) => {
                for selection in self.selection.clone() {
                    if let Selection::Part(part_id) = selection {
                        if let Some(part) = self.canvas_snapshot.parts.get_mut(&part_id) {
                            part.color = self.current_paint_color.clone();
                        }
                    }
                }
                self.selection.clear();
            }
            Some(Tool::Simulator) => {
                if let Some(part) = self.part_at_pos(world_pos) {
                    if let Some(new_i) = part.simulation_index {
                        if let Some(sim_state) = &self.sim_state {
                            let mut state = sim_state.lock();
                            let new_val = !state.part_outputs[new_i];
                            state.part_outputs[new_i] = new_val;
                            state.prev_outputs[new_i] = new_val;
                        }
                    }
                }
            }
        }
    }

    pub fn select_part(&mut self, part_id: u64, shift_held: bool) {
        if !shift_held {
            self.selection.clear();
        }
        if !self.selection.contains(&Selection::Part(part_id)) {
            self.selection.push(Selection::Part(part_id));
        }
    }
    pub fn select_connection(&mut self, connection_id: usize, shift_held: bool) {
        if !shift_held {
            self.selection.clear();
        }
        if !self
            .selection
            .contains(&Selection::Connection(connection_id))
        {
            self.selection.push(Selection::Connection(connection_id));
        }
    }
}

fn sort_ports_by_position(ports: &mut Vec<Port>, app: &AppState) {
    // this function was made by ai
    // im too stupid to figure ts out

    // figure out the spread on each axis
    let positions: Vec<Pos2> = ports.iter().filter_map(|p| p.pos(app)).collect();

    let x_spread = positions.iter().map(|p| p.x).fold(f32::MIN, f32::max)
        - positions.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let y_spread = positions.iter().map(|p| p.y).fold(f32::MIN, f32::max)
        - positions.iter().map(|p| p.y).fold(f32::MAX, f32::min);

    ports.sort_by(|a, b| {
        let a_pos = a.pos(app).unwrap_or_default();
        let b_pos = b.pos(app).unwrap_or_default();
        if y_spread >= x_spread {
            // primary: top to bottom, secondary: left to right
            a_pos
                .y
                .partial_cmp(&b_pos.y)
                .unwrap()
                .then(a_pos.x.partial_cmp(&b_pos.x).unwrap())
        } else {
            // primary: left to right, secondary: top to bottom
            a_pos
                .x
                .partial_cmp(&b_pos.x)
                .unwrap()
                .then(a_pos.y.partial_cmp(&b_pos.y).unwrap())
        }
    });
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    s.chars()
        .rev()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && i % 3 == 0 {
                vec![',', c]
            } else {
                vec![c]
            }
        })
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}
