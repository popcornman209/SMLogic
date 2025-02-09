mod logic;
use std::time::Instant;

fn main() {
    let file_content = std::fs::read_to_string("gates.json")
        .expect("Failed to read file");
    let gates: Vec<logic::Gate> = serde_json::from_str(&file_content)
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
    let pro_que = logic::setup_opencl(num_gates);
    // Note: current and next need to be mutable so that we can swap them.
    let (types, connections_from, offsets, mut current, mut next) =
        logic::setup_opencl_buffers(&pro_que, num_gates, num_connections);

    // Write data to the GPU buffers.
    types.write(&gate_modes).enq().unwrap();
    connections_from.write(&flattened_connections).enq().unwrap();
    offsets.write(&offsets_vec).enq().unwrap();

    // Initialize the current state (for example, all zeros).
    let initial_state = vec![0u8; num_gates];
    current.write(&initial_state).enq().unwrap();

    let kernel = logic::build_logic_kernel(&pro_que, &types, &connections_from, &offsets, &current, &next);

    let start_time = Instant::now();

    for _frame in 0..40000 {
        // Enqueue the kernel to compute the next state.
        logic::logic_step(&kernel, &current, &next);
        // Ensure all kernel work is done.
        pro_que.queue().finish().unwrap();

        // Read back the computed state from the 'next' buffer.
        let mut output = vec![0u8; num_gates];
        next.read(&mut output).enq().unwrap();
        //println!("Frame {}: {:?}", frame, output);

        // Swap the buffers so that 'next' becomes the current state for the next frame.
        std::mem::swap(&mut current, &mut next);
    }
    let duration = start_time.elapsed();
    println!("Logic step took: {:?}", duration);
}