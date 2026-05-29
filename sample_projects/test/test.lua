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
		id = create_gate("nand", col * 80, row * 60 + 30, {color = color})
		if is_set then
			table.insert(white_gates, id)
		end
	end
end

other_gate = create_gate("and",-120,30)

for _, white_gate in ipairs(white_gates) do
    add_connection(other_gate, white_gate)
end