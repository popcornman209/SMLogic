use egui::{Color32, Pos2};

use crate::parts::{GateType, PartData, Port};
use crate::state::{AppState, CanvasSnapshot};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const BATCH_SIZE: usize = 64; // how many operations are done per loop, so i dont have to lock the
// variable as much. might change this to be dynamic later tho.

#[derive(Clone)]
pub enum PartType {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    Timer(VecDeque<bool>),
}

pub struct SimSnapshot {
    pub outputs: Vec<bool>,
    pub tick: u64,
    pub running: bool,
    pub target_spt: Option<Duration>,
}

pub struct SimState {
    pub running: bool,
    pub step: bool,
    pub kill_thread: bool,
    pub target_spt: Option<Duration>, // 1 / tps
    pub tick: u64,
    pub part_types: Vec<PartType>,
    pub part_outputs: Vec<bool>,
    pub prev_outputs: Vec<bool>,
    pub part_inputs: Vec<Vec<usize>>,
}

impl SimState {
    pub fn tick(&mut self) {
        self.tick += 1;
        std::mem::swap(&mut self.part_outputs, &mut self.prev_outputs);
        for i in 0..self.part_types.len() {
            if self.part_inputs[i].len() != 0 {
                let input_idxs = &self.part_inputs[i];
                match &mut self.part_types[i] {
                    PartType::And => {
                        self.part_outputs[i] = input_idxs.iter().all(|&idx| self.prev_outputs[idx]);
                    }
                    PartType::Or => {
                        self.part_outputs[i] = input_idxs.iter().any(|&idx| self.prev_outputs[idx]);
                    }
                    PartType::Xor => {
                        self.part_outputs[i] = input_idxs
                            .iter()
                            .filter(|&&idx| self.prev_outputs[idx])
                            .count()
                            % 2
                            == 1;
                    }
                    PartType::Nand => {
                        self.part_outputs[i] =
                            !input_idxs.iter().all(|&idx| self.prev_outputs[idx]);
                    }
                    PartType::Nor => {
                        self.part_outputs[i] =
                            !input_idxs.iter().any(|&idx| self.prev_outputs[idx]);
                    }
                    PartType::Xnor => {
                        self.part_outputs[i] = !input_idxs
                            .iter()
                            .filter(|&&idx| self.prev_outputs[idx])
                            .count()
                            % 2
                            == 1;
                    }
                    PartType::Timer(buffer) => {
                        let input = input_idxs
                            .first()
                            .map(|&idx| self.prev_outputs[idx])
                            .unwrap_or(false);
                        if buffer.is_empty() {
                            // 0 tick timer doesnt need any buffer stuff, just pass it through
                            self.part_outputs[i] = input;
                        } else {
                            let out = buffer.pop_back().unwrap_or(false);
                            buffer.push_front(input);
                            self.part_outputs[i] = out;
                        }
                    }
                }
            }
        }
    }

    pub fn from_canvas_snapshot(canvas: &mut CanvasSnapshot) -> Self {
        let (
            part_types,
            _colors,
            _positions,
            connections,
            id_remap,
            _tunnel_connections,
            _io_parts,
        ) = get_canvas_raw_data(canvas.clone(), true);
        for (original_id, new_i) in id_remap {
            if let Some(part) = canvas.parts.get_mut(&original_id) {
                part.simulation_index = Some(new_i);
            }
        }

        let mut part_inputs: Vec<Vec<usize>> = vec![Vec::new(); part_types.len()];
        for connection in connections {
            part_inputs[connection.1].push(connection.0);
        }

        Self {
            running: false,
            step: false,
            kill_thread: false,
            target_spt: Some(Duration::from_secs_f32(0.025)), // 40 tps
            tick: 0,
            part_types: part_types.clone(),
            part_outputs: vec![false; part_types.len()],
            prev_outputs: vec![false; part_types.len()],
            part_inputs: part_inputs,
        }
    }
}

// this was fucking torture to make istg lost my mind
// also had alot of issues so i js made claude fix them (this function was too much for me lmao)
pub fn get_canvas_raw_data(
    canvas: CanvasSnapshot,
    top_level: bool, // wether it is the main canvas or not (not sub modules)
) -> (
    Vec<PartType>,            // part_output
    Vec<Color32>,             // color_output
    Vec<Pos2>,                // pos_output
    Vec<(usize, usize)>,      // connection_output
    HashMap<u64, usize>,      // id remap
    HashMap<u64, Vec<usize>>, // tunnel connections
    Vec<usize>,               // io parts (only should have stuff in it if top level)
) {
    let mut id_remap: HashMap<u64, usize> = HashMap::new();
    let mut part_output: Vec<PartType> = Vec::new();
    let mut color_output: Vec<Color32> = Vec::new();
    let mut pos_output: Vec<Pos2> = Vec::new();
    let mut connection_output: Vec<(usize, usize)> = Vec::new();

    // top level only
    let mut io_parts: Vec<usize> = Vec::new();

    let mut tunnel_connections: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut sub_tunnel_connections: HashMap<u64, HashMap<u64, Vec<usize>>> = HashMap::new();

    for (part_id, part) in &canvas.parts {
        match part.part_data.clone() {
            PartData::Gate(gate) => {
                let new_i = part_output.len();
                part_output.push(match gate.gate_type {
                    GateType::And => PartType::And,
                    GateType::Or => PartType::Or,
                    GateType::Xor => PartType::Xor,
                    GateType::Nand => PartType::Nand,
                    GateType::Nor => PartType::Nor,
                    GateType::Xnor => PartType::Xnor,
                });
                color_output.push(part.color);
                pos_output.push(part.pos);
                id_remap.insert(*part_id, new_i);
            }
            PartData::Timer(timer) => {
                let new_i = part_output.len();
                let ticks = timer.get_ticks();
                part_output.push(PartType::Timer(VecDeque::from(vec![false; ticks])));
                color_output.push(part.color);
                pos_output.push(part.pos);
                id_remap.insert(*part_id, new_i);
            }
            PartData::Module(module) => {
                let (
                    module_parts,
                    colors,
                    positions,
                    module_connections,
                    _module_id_remap,
                    module_tunnel_connections,
                    _io_parts, // should be empty anyway
                ) = get_canvas_raw_data(module.canvas_snapshot, false);
                let offset = part_output.len();
                part_output.extend(module_parts);
                color_output.extend(colors);
                pos_output.extend(positions);

                connection_output.extend(
                    module_connections
                        .iter()
                        .map(|(a, b)| (a + offset, b + offset)),
                );

                let remapped_tunnel_connections: HashMap<_, _> = module_tunnel_connections
                    .iter()
                    .map(|(&k, v)| (k, v.iter().map(|&x| x + offset).collect::<Vec<_>>()))
                    .collect();

                sub_tunnel_connections.insert(*part_id, remapped_tunnel_connections);
            }
            PartData::IO(_io) => {
                if top_level {
                    let new_i = part_output.len();
                    part_output.push(PartType::And);
                    color_output.push(part.color);
                    pos_output.push(part.pos);
                    id_remap.insert(*part_id, new_i);
                    io_parts.push(new_i);
                }
            }
            _ => {}
        }
    }

    let resolve = |port: &Port| -> Vec<usize> {
        if let Some(&new_id) = id_remap.get(&port.part) {
            vec![new_id]
        } else if let Some(module) = sub_tunnel_connections.get(&port.part) {
            port.port_id
                .and_then(|pid| module.get(&pid))
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    for connection in canvas.connections {
        let start_is_io = canvas
            .parts
            .get(&connection.start.part)
            .map_or(false, |p| matches!(&p.part_data, PartData::IO(_)));
        let end_is_io = canvas
            .parts
            .get(&connection.end.part)
            .map_or(false, |p| matches!(&p.part_data, PartData::IO(_)));

        if !top_level && start_is_io {
            // sub-module IO input fans out to multiple internal gates — collect all of them
            for new_id in resolve(&connection.end) {
                tunnel_connections
                    .entry(connection.start.part)
                    .or_default()
                    .push(new_id);
            }
        } else if !top_level && end_is_io {
            for new_id in resolve(&connection.start) {
                tunnel_connections
                    .entry(connection.end.part)
                    .or_default()
                    .push(new_id);
            }
        } else {
            // top-level IO parts are real AND gates in id_remap, so resolve() finds them normally
            let starts = resolve(&connection.start);
            let ends = resolve(&connection.end);
            for &start_port in &starts {
                for &end_port in &ends {
                    connection_output.push((start_port, end_port));
                }
            }
        }
    }

    (
        part_output,
        color_output,
        pos_output,
        connection_output,
        id_remap,
        tunnel_connections,
        io_parts,
    )
}

pub fn main_loop(sim_state: Arc<Mutex<SimState>>, sim_snapshot: Arc<Mutex<SimSnapshot>>) {
    let mut last_tick = Instant::now();
    loop {
        let (running, step, spt, kill) = {
            let mut state = sim_state.lock();
            let batch = if state.target_spt.is_none() {
                BATCH_SIZE
            } else {
                1
            };
            if state.running {
                for _ in 0..batch {
                    state.tick();
                }
            } else if state.step {
                state.tick();
                state.step = false;
            }
            let mut snap = sim_snapshot.lock();
            snap.outputs.clone_from(&state.part_outputs);
            snap.tick = state.tick;
            snap.running = state.running;
            snap.target_spt = state.target_spt;
            drop(snap);
            (
                state.running,
                state.step,
                state.target_spt,
                state.kill_thread,
            )
        };

        if kill {
            break;
        }

        if running {
            if let Some(spt) = spt {
                let elapsed = last_tick.elapsed();
                if elapsed < spt {
                    std::thread::sleep(spt - elapsed);
                }
                last_tick = Instant::now();
            }
        } else if !running && !step {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

pub fn start_thread(
    canvas: &mut CanvasSnapshot,
) -> (Arc<Mutex<SimState>>, Arc<Mutex<SimSnapshot>>) {
    let sim_state = Arc::new(Mutex::new(SimState::from_canvas_snapshot(canvas)));
    let sim_state_thread = Arc::clone(&sim_state);
    let sim_snapshot = Arc::new(Mutex::new(SimSnapshot {
        outputs: Vec::new(),
        tick: 0,
        running: false,
        target_spt: None,
    }));
    let sim_snapshot_thread = Arc::clone(&sim_snapshot);
    std::thread::spawn(move || main_loop(sim_state_thread, sim_snapshot_thread));
    (sim_state, sim_snapshot)
}

impl AppState {
    pub fn start_simulation(&mut self) {
        let (sim_state, sim_snapshot) = start_thread(&mut self.canvas_snapshot);
        self.sim_state = Some(sim_state);
        self.sim_snapshot = Some(sim_snapshot);
        self.last_tick_count = 0;
        self.last_tps_check = Instant::now();
    }
    pub fn end_simulation(&mut self) {
        if let Some(sim_state) = &self.sim_state {
            let mut state = sim_state.lock();
            state.kill_thread = true;
            state.running = false;
        }
        self.sim_state = None;
        self.sim_snapshot = None;
        for part in self.canvas_snapshot.parts.values_mut() {
            part.simulation_index = None;
        }
    }
}
