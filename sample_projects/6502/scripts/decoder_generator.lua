bits = 4

local inputs = {}
for i = 0, bits-1 do
    inputs[i] = create_input(-120, i * -80, {label= "input "..i})
end
for index = 0, 2^bits-1 do
    local y = index * -80
    local nand_gate = create_gate("nor", 0, y)
    local and_gate  = create_gate("and", 120, y)
    local output    = create_output(240, y, {label = "output 0x"..string.format("%02X", index)})
    add_connection(nand_gate, and_gate)
    add_connection(and_gate, output)
    for bit = 0, bits-1 do
        local is_one = (index >> bit) & 1 == 1
        if is_one then
            add_connection(inputs[bit], and_gate)
        else
            add_connection(inputs[bit], nand_gate)
        end
    end
end