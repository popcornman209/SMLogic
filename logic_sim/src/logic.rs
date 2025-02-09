use serde::{Deserialize, Serialize};
use std::fs;
use ocl::{ProQue, Buffer, Kernel};

#[derive(Deserialize, Serialize, Debug)]
pub struct Gate {
    pub mode: u8,                   // Gate type (0-5) for and or xor nand nor xnor
    pub connections_from: Vec<u32>, // input connections 
}

// Sets up the OpenCL context using a kernel source file and a number of gates.
pub fn setup_opencl(num_gates: usize) -> ProQue {
    let kernel_src = fs::read_to_string("src/kernel.cl") //kernel.cl file. for computing the gates on the gpu itself
        .expect("kernel.cl file couldnt load, missing?");
    ProQue::builder()
        .src(&kernel_src)
        .dims(num_gates)
        .build()
        .expect("Failed to create OpenCL context")
}

pub fn setup_opencl_buffers(pro_que: &ProQue, num_gates: usize, num_connections: usize) -> (Buffer<u8>, Buffer<u32>, Buffer<u32>, Buffer<u8>, Buffer<u8>) {
    let types = Buffer::<u8>::builder() //types of gates, 
        .queue(pro_que.queue().clone())
        .len(num_gates)
        .build()
        .unwrap();

    let connections_from = Buffer::<u32>::builder() //parts connected to this part
        .queue(pro_que.queue().clone())
        .len(num_connections)
        .build()
        .unwrap();

    let offsets = Buffer::<u32>::builder() //start locations of connections_from per gate. [0,1,3,7] would mean the amount of connections is [1,2,4,restOfBuffer]
        .queue(pro_que.queue().clone())
        .len(num_gates + 1)
        .build()
        .unwrap();

    let current = Buffer::<u8>::builder() // current gate values, used for calculating next values for all gates connected to
        .queue(pro_que.queue().clone())
        .len(num_gates)
        .build()
        .unwrap();

    let next = Buffer::<u8>::builder() // next gate values, all are applied after the midFrame is complete
        .queue(pro_que.queue().clone())
        .len(num_gates)
        .build()
        .unwrap();

    (types, connections_from, offsets, current, next)
}

//setup for the logic step, makes the kernel thing
pub fn build_logic_kernel(
    pro_que: &ProQue,
    types: &Buffer<u8>,
    connections_from: &Buffer<u32>,
    offsets: &Buffer<u32>,
    current: &Buffer<u8>,
    next: &Buffer<u8>,
) -> Kernel {
    pro_que
        .kernel_builder("logic_step")
        .arg(types)
        .arg(connections_from)
        .arg(offsets)
        .arg(current)
        .arg(next)
        .build()
        .unwrap()
}

pub fn logic_step(kernel: &Kernel, current: &Buffer<u8>, next: &Buffer<u8>) {
    // Update the kernel arguments if needed (not necessary unless buffer changes)
    kernel.set_arg(3, current).unwrap(); // current buffer
    kernel.set_arg(4, next).unwrap();    // next buffer

    unsafe {
        kernel.enq().unwrap();
    }
}