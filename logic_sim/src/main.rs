// At the top of your file (or in a separate module)
use serde::{Deserialize, Serialize};
use std::fs;
use ocl::{ProQue, Buffer};

#[derive(Deserialize, Serialize, Debug)]
struct Gate {
    mode: u8,                // Gate type (0-5)
    connections: Vec<u32>,   // (Optional: if you need output connections)
    connections_from: Vec<u32>, // The input connections (indices of gates feeding into this gate)
}

// Sets up the OpenCL context using a kernel source file and a number of gates.
fn setup_opencl(num_gates: usize) -> ProQue {
    // Read the kernel source (and use it in the builder)
    let kernel_src = fs::read_to_string("src/kernel.cl")
        .expect("Failed to read kernel file");
    ProQue::builder()
        .src(&kernel_src)
        .dims(num_gates)
        .build()
        .expect("Failed to create OpenCL context")
}

// Creates and returns the buffers for types, connections, offsets, current state, and next state.
fn setup_opencl_buffers(
    pro_que: &ProQue,
    num_gates: usize,
    num_connections: usize,
) -> (Buffer<u8>, Buffer<u32>, Buffer<u32>, Buffer<u8>, Buffer<u8>) {
    let types = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(num_gates)
        .build()
        .unwrap();

    let connections_from = Buffer::<u32>::builder()
        .queue(pro_que.queue().clone())
        .len(num_connections)
        .build()
        .unwrap();

    let offsets = Buffer::<u32>::builder()
        .queue(pro_que.queue().clone())
        .len(num_gates + 1)
        .build()
        .unwrap();

    let current = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(num_gates)
        .build()
        .unwrap();

    let next = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(num_gates)
        .build()
        .unwrap();

    (types, connections_from, offsets, current, next)
}

// Runs one simulation (logic) step.
fn logic_step(
    types: &Buffer<u8>,
    connections_from: &Buffer<u32>,
    offsets: &Buffer<u32>,
    current: &Buffer<u8>,
    next: &Buffer<u8>,
    pro_que: &ProQue,
) {
    let kernel = pro_que
        .kernel_builder("logic_step")
        .arg(types)
        .arg(connections_from)
        .arg(offsets)
        .arg(current)
        .arg(next)
        .build()
        .unwrap();

    unsafe {
        kernel.enq().unwrap();
    }

    // Example: Read back the result if needed
    let mut next_results = vec![0u8; pro_que.dims().to_len()];
    next.read(&mut next_results).enq().unwrap();
    println!("Simulation step completed!");
}

fn main() {
    let file_content = std::fs::read_to_string("gates.json")
        .expect("Failed to read file");
    let gates: Vec<Gate> = serde_json::from_str(&file_content)
        .expect("Failed to parse JSON");

    let num_gates = gates.len();

    // Build a flattened vector of all input connections from "connectionsFrom"
    // and an offsets vector indicating where each gate's connection list begins.
    let mut flattened_connections: Vec<u32> = Vec::new();
    let mut offsets_vec: Vec<u32> = Vec::with_capacity(num_gates + 1);
    offsets_vec.push(0); // First offset is 0.
    for gate in &gates {
        flattened_connections.extend(&gate.connections_from);
        offsets_vec.push(flattened_connections.len() as u32);
    }
    let num_connections = flattened_connections.len();

    // Build a vector for gate modes (using the "mode" field).
    let gate_modes: Vec<u8> = gates.iter().map(|gate| gate.mode).collect();

    // Set up the OpenCL context and buffers.
    let pro_que = setup_opencl(num_gates);
    // Note: current and next need to be mutable so that we can swap them.
    let (types, connections_from, offsets, mut current, mut next) =
        setup_opencl_buffers(&pro_que, num_gates, num_connections);

    // Write data to the GPU buffers.
    types.write(&gate_modes).enq().unwrap();
    connections_from.write(&flattened_connections).enq().unwrap();
    offsets.write(&offsets_vec).enq().unwrap();

    // Initialize the current state (for example, all zeros).
    let initial_state = vec![0u8; num_gates];
    current.write(&initial_state).enq().unwrap();

    // Run 5 simulation frames.
    for frame in 0..5000 {
        // Enqueue the kernel to compute the next state.
        logic_step(&types, &connections_from, &offsets, &current, &next, &pro_que);
        // Ensure all kernel work is done.
        pro_que.queue().finish().unwrap();

        // Read back the computed state from the 'next' buffer.
        let mut output = vec![0u8; num_gates];
        next.read(&mut output).enq().unwrap();
        println!("Frame {}: {:?}", frame, output);

        // Swap the buffers so that 'next' becomes the current state for the next frame.
        std::mem::swap(&mut current, &mut next);
    }
}