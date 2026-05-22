use egui::Color32;

use crate::parts::{GateType, PartData, Port};
use crate::state::{AppState, CanvasSnapshot};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub enum PartType {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    Timer(VecDeque<bool>),
    Switch,
}

pub struct SimState {
    pub running: bool,
    pub step: bool,
    pub kill_thread: bool,
    pub target_spt: Option<Duration>,
    pub tick: u64,
    pub part_types: Vec<PartType>,
    pub part_outputs: Vec<bool>,
    pub part_inputs: Vec<Vec<usize>>,
}

impl SimState {
    pub fn tick(&mut self) {
        let prev_outputs = self.part_outputs.clone();
        for i in 0..self.part_types.len() {
            if self.part_inputs[i].len() != 0 {
                let inputs: Vec<bool> = self.part_inputs[i]
                    .iter()
                    .map(|&idx| prev_outputs[idx])
                    .collect();
                match &mut self.part_types[i] {
                    PartType::And => {
                        self.part_outputs[i] = inputs.iter().all(|&x| x);
                    }
                    PartType::Or => {
                        self.part_outputs[i] = inputs.iter().any(|&x| x);
                    }
                    PartType::Xor => {
                        self.part_outputs[i] = inputs.iter().filter(|&&x| x).count() % 2 == 1;
                    }
                    PartType::Nand => {
                        self.part_outputs[i] = !inputs.iter().all(|&x| x);
                    }
                    PartType::Nor => {
                        self.part_outputs[i] = !inputs.iter().any(|&x| x);
                    }
                    PartType::Xnor => {
                        self.part_outputs[i] = !inputs.iter().filter(|&&x| x).count() % 2 == 1;
                    }
                    PartType::Timer(buffer) => {
                        let input = inputs.first().copied().unwrap_or(false);
                        self.part_outputs[i] = buffer.pop_back().unwrap_or(false);
                        buffer.push_front(input);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn from_canvas_snapshot(canvas: &mut CanvasSnapshot) -> Self {
        let (part_types, _, connections, id_remap, _) = get_canvas_raw_data(canvas.clone());
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
            part_inputs: part_inputs,
        }
    }
}

// this was fucking torture to make istg lost my mind
pub fn get_canvas_raw_data(
    canvas: CanvasSnapshot,
) -> (
    Vec<PartType>,
    Vec<Color32>,
    Vec<(usize, usize)>,
    HashMap<u64, usize>,
    HashMap<u64, usize>,
) {
    let mut id_remap: HashMap<u64, usize> = HashMap::new();
    let mut part_output: Vec<PartType> = Vec::new();
    let mut color_output: Vec<Color32> = Vec::new();
    let mut connection_output: Vec<(usize, usize)> = Vec::new();
    let mut tunnel_connections: HashMap<u64, usize> = HashMap::new();
    let mut sub_tunnel_connections: HashMap<u64, HashMap<u64, usize>> = HashMap::new();

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
                id_remap.insert(*part_id, new_i);
            }
            PartData::Timer(timer) => {
                let new_i = part_output.len();
                let ticks = timer.get_ticks();
                part_output.push(PartType::Timer(VecDeque::from(vec![false; ticks])));
                color_output.push(part.color);
                id_remap.insert(*part_id, new_i);
            }
            PartData::Switch(_switch) => {
                let new_i = part_output.len();
                part_output.push(PartType::Switch);
                color_output.push(part.color);
                id_remap.insert(*part_id, new_i);
            }
            PartData::Module(module) => {
                let (
                    module_parts,
                    colors,
                    module_connections,
                    _module_id_remap,
                    module_tunnel_connections,
                ) = get_canvas_raw_data(module.canvas_snapshot);
                let offset = part_output.len();
                part_output.extend(module_parts);
                color_output.extend(colors);

                connection_output.extend(
                    module_connections
                        .iter()
                        .map(|(a, b)| (a + offset, b + offset)),
                );

                let remapped_tunnel_connections: HashMap<_, _> = module_tunnel_connections
                    .iter()
                    .map(|(&k, &v)| (k, v + offset))
                    .collect(); // new indexes

                sub_tunnel_connections.insert(*part_id, remapped_tunnel_connections);
            }
            _ => {}
        }
    }

    let resolve = |port: &Port| -> Option<usize> {
        if let Some(&new_id) = id_remap.get(&port.part) {
            Some(new_id)
        } else if let Some(module) = sub_tunnel_connections.get(&port.part) {
            port.port_id.and_then(|pid| module.get(&pid).copied())
        } else {
            None
        }
    };

    for connection in canvas.connections {
        if canvas
            .parts
            .get(&connection.start.part)
            .map_or(false, |p| matches!(&p.part_data, PartData::IO(_)))
        {
            if let Some(new_id) = resolve(&connection.end) {
                tunnel_connections.insert(connection.start.part, new_id);
            }
        } else if canvas
            .parts
            .get(&connection.end.part)
            .map_or(false, |p| matches!(&p.part_data, PartData::IO(_)))
        {
            if let Some(new_id) = resolve(&connection.start) {
                tunnel_connections.insert(connection.end.part, new_id);
            }
        } else {
            let start = resolve(&connection.start);
            let end = resolve(&connection.end);
            if let (Some(start_port), Some(end_port)) = (start, end) {
                connection_output.push((start_port, end_port));
            }
        }
    }

    (
        part_output,
        color_output,
        connection_output,
        id_remap,
        tunnel_connections,
    )
}

pub fn main_loop(sim_state: Arc<Mutex<SimState>>) {
    let mut last_tick = Instant::now();
    loop {
        let mut state = sim_state.lock().unwrap();
        if state.running {
            state.tick();
            if let Some(spt) = state.target_spt {
                let elapsed = last_tick.elapsed();
                if elapsed < spt {
                    std::thread::sleep(spt - elapsed);
                }
                last_tick = Instant::now();
            }
        } else if state.step {
            state.tick();
            state.step = false;
        }
        if state.kill_thread {
            break;
        }
    }
}

pub fn start_thread(canvas: &mut CanvasSnapshot) -> Arc<Mutex<SimState>> {
    let sim_state = Arc::new(Mutex::new(SimState::from_canvas_snapshot(canvas)));
    let sim_state_thread = Arc::clone(&sim_state);
    std::thread::spawn(move || main_loop(sim_state_thread));
    sim_state
}

impl AppState {
    pub fn start_simulation(&mut self) {
        self.sim_state = Some(start_thread(&mut self.canvas_snapshot));
    }
}
