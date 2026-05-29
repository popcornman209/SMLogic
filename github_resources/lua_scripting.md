# Lua Scripting
This is a basic Lua55 intregration, with features to read files and create/modify parts, primarily for anything very tedious and easily automated.

# custom functions

### create_gate
you can create a logic gate with the `create_gate` function, which takes the arguments `(type, x_position, y_position)` and optional arguments {color = hex} or {important = bool} and returns the parts ID.
```lua
-- basic xor gate at 0,0
create_gate("xor", 0, 0)
-- green nor gate at 0,0
id = create_gate("nor", 0, 0 {color = "#00ff00"})
-- red important and gate at 100,20
id = create_gate("and", 100, 20, {color = "#ff0000", important = true})
```

### create_timer
timers are created with the `create_timer` function. it takes the inputs of ()
```lua
-- timer 10 seconds long at 0, 0
create_timer(10, 0, 0, 0)
-- blue timer at -120,30 being 0 seconds and 18 ticks long
id = create_timer(0, 18, -120, 30, {color = "#0000ff"})
```

### add_connection
you can connect parts together with `add_connection`, inputs being `(from_part_id, to_part_id)`, where this will connect the `from_part` to the `to_part`. this currently does not support modules as they have multiple inputs. the id being used is the same as the part id returned from create_timer or create_gate and the one shown in properties on the left side bar.
```lua
-- connects gate1 to gate2
add_connection(gate1_id, gate2_id)
```

### print
a generic print function, prints any string to the output.
```lua
print("hello world")
```

### read_file
takes in a local path to a file within the project folder or a global one on your filesystem and returns a string of the data inside.
```lua
file_data = read_file("test.txt")
```

### read_bytes
takes the same inputs as read_file, but returns the raw bytes of the file as opposed to a string.
```lua
data = read_bytes("test.bin")
```

## examples

this example prints everything in test.lua, and creates a 8x8 grid of gates based on the bits in lua_test.bin. all bits that are 1 flash on and off twice per second and are colored white, while all other bits are black
```lua
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
```
