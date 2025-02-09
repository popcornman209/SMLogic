__kernel void logic_step(__global const uchar *types, 
                         __global const uint *connectionsFrom, 
                         __global const uint *offsets, 
                         __global const uchar *current, 
                         __global uchar *next) {
    int idx = get_global_id(0);
    
    // Read gate type
    uchar type = types[idx];
    uchar result = 0;
    uchar xor_count = 0;

    uint start = offsets[idx];
    uint end = offsets[idx + 1];

    for (uint i = start; i < end; i++) {
        uint conn_idx = connectionsFrom[i];
        uchar input_val = current[conn_idx];

        if (type == 2 || type == 5) {
            xor_count += input_val;  // Count active inputs for XOR/XNOR
        } else {
            if (i == start) result = input_val;
            else {
                if (type == 0 || type == 3) result &= input_val;  // AND/NAND
                else if (type == 1 || type == 4) result |= input_val;  // OR/NOR
            }
        }
    }

    // XOR/XNOR logic (separate from the loop)
    if (type == 2) result = (xor_count % 2 == 1) ? 1 : 0;  // XOR
    if (type == 5) result = (xor_count % 2 == 0) ? 1 : 0;  // XNOR

    if (type == 3 || type == 4) result = !result;  // NAND/NOR inversion

    next[idx] = result;
}
