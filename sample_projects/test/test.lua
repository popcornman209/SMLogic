print(read_file("test.lua"))

data = read_bytes("lua_test.bin")

white_gates = {}

-- create an 8x8 grid of gates, white = bit 1, black = bit 0
for row = 0, 7 do
	for col = 0, 7 do
		local bit_index = row * 8 + col
		local byte_val = data[bit_index // 8 + 1]
		local is_set = (byte_val >> (bit_index % 8)) & 1 == 1
		local color = is_set and "#ffffff" or "#000000"
		id = create_gate("nand", col * 80, row * 60 + 30, {color = color, important = true})
		if is_set then
			table.insert(white_gates, id)
		end
	end
end

-- main clock
timer = create_timer(0,18,-120,30)
nand_gate = create_gate("nand", -120, 110)
and_gate = create_gate("and", -240, 30)

add_connection(timer, nand_gate)
add_connection(nand_gate, and_gate)
add_connection(and_gate, timer)

-- connect clock to white gates to make them blink
for _, white_gate in ipairs(white_gates) do
    add_connection(timer, white_gate)
end