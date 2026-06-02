local width = 48
local height = 36
local fps_multiplier = 2 -- 40/2 = 20 fps

local buffer_count = 8
local max_frames = 30 * 20

-- create start input (when activated generates <buffer> tick long pulse)
local input_id = create_input(0, -140, { color = "#00ff00" })
local nand_id = create_gate("nand", 120, -100)
local and_id = create_gate("and", 120, -180)
local xor_id = create_gate("xor", 260, -180)
local timer_id = create_timer(0, (buffer_count * fps_multiplier) - 1, 260, -100)
add_connection(input_id, nand_id)
add_connection(input_id, and_id)
add_connection(nand_id, and_id)
add_connection(xor_id, xor_id)
add_connection(and_id, xor_id)
add_connection(and_id, timer_id)
add_connection(timer_id, xor_id)

local buffer_activator_clock = {}
local last_activator = -1
for i = 1, buffer_count * fps_multiplier do
	local id = create_gate("or", 260, -260 - i * 60)
	if last_activator ~= -1 then
		add_connection(id, last_activator)
	end
	last_activator = id
	table.insert(buffer_activator_clock, id)
end
add_connection(buffer_activator_clock[1], last_activator)
add_connection(timer_id, buffer_activator_clock[buffer_count * fps_multiplier])

local buffer_activators = {}
for i = 1, buffer_count do
	local id = create_gate("or", 380, -320 - ((i - 1) * fps_multiplier) * 60)
	for x = 1, fps_multiplier do
		add_connection(buffer_activator_clock[((i - 1) * fps_multiplier) + 1 + (x - 1)], id)
	end
	table.insert(buffer_activators, id)
end

-- main output gates
local output_gates = {}
for y = 1, height do
	for x = 1, width do
		table.insert(output_gates, create_gate("or", x * 80, y * 60, { color = "#000000", important = true }))
	end
end

-- buffers, go directly into the output gates
local buffer_main_activators = {}
for i = 1, buffer_count do
	local current_buffer = {}
	for y = 1, height do
		for x = 1, width do
			local id = create_gate("and", (x + width) * 80, (y + (height + 1) * (i - 1)) * 60, { color = "#ffff00" })
			add_connection(id, output_gates[x + (y - 1) * width])
			add_connection(buffer_activators[buffer_count - (i - 1)], id)
			table.insert(current_buffer, id)
		end
	end
	table.insert(buffer_main_activators, current_buffer)
end

local buffers = {}
for i = 1, buffer_count do
	local current_buffer = {}
	for y = 1, height do
		for x = 1, width do
			local id = create_gate("or", (x + width * 2) * 80, (y + (height + 1) * (i - 1)) * 60, { color = "#0000ff" })
			add_connection(id, buffer_main_activators[i][x + (y - 1) * width])
			table.insert(current_buffer, id)
		end
	end
	table.insert(buffers, current_buffer)
end

local data = read_bytes("./bin_generator/bad_apple.bin")

local bytes_per_frame = (width * height) / 8 -- 216

local function get_bit(frame_index, pixel_index)
	local byte_val = data[frame_index * bytes_per_frame + (pixel_index // 8) + 1]
	return (byte_val >> (7 - (pixel_index % 8))) & 1
end

local frames = math.min(max_frames, #data / bytes_per_frame)

local last_timer = xor_id
for i = 1, frames, buffer_count do
	id = create_timer(0, (buffer_count * fps_multiplier) - 1, 480 + (i / 8) * 120, -180)
	add_connection(last_timer, id)
	last_timer = id
	for x = 1, buffer_count do
		local frame_index = i + x - 2
		for p = 0, width * height - 1 do
			if get_bit(frame_index, p) == 1 then
				add_connection(id, buffers[x][p + 1])
			end
		end
	end
end
