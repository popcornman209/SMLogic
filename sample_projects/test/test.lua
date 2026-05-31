print(read_file("test.lua"))

-- main clock
timer = create_timer(0,18,-140,40, {color="#ff0000"})
nand_gate = create_gate("and", -140, 140)
and_gate = create_gate("and", -260, 60)

add_connection(timer, nand_gate)
add_connection(nand_gate, and_gate)
add_connection(and_gate, timer)

modify_gate(nand_gate, {type = "nand", color="#0000ff", label="NAND"})

-- should be #ff0000
print(get_part(timer)["color"])

data = read_bytes("lua_test.bin")

white_gates = {}

-- create an 8x8 grid of gates, white = bit 1, black = bit 0
for row = 0, 7 do
	for col = 0, 7 do
		local bit_index = row * 8 + col
		local byte_val = data[bit_index // 8 + 1]
		local is_set = (byte_val >> (bit_index % 8)) & 1 == 1
		local color = is_set and "#ffffff" or "#000000"
		id = create_output(col * 80, row * 60, {color = color})
		if is_set then
			table.insert(white_gates, id)
		end
	end
end

-- connect clock to white gates to make them blink
for _, white_gate in ipairs(white_gates) do
    add_connection(timer, white_gate)
end

selection = get_selection()
print("parts selected: "..#selection["parts"])
print("connections selected: "..#selection["connections"])