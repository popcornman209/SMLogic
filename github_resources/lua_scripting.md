# Lua Scripting
This is a basic Lua55 intregration, with features to read files and create/modify parts, primarily for anything very tedious and easily automated. currently modules are not very well implemented, and watch out to make sure your program doesnt end up in an infinite loop, the whole app will crash lol.

# custom functions

### create_gate
you can create a logic gate with the `create_gate` function, which takes the arguments `(type, x_position, y_position, {opts})` and optional arguments {color = hex}, {important = bool}, or {label = name} and returns the parts ID.
```lua
-- basic xor gate at 0,0
create_gate("xor", 0, 0)
-- green nor gate at 0,0
id = create_gate("nor", 0, 0 {color = "#00ff00"})
-- red important and gate at 100,20
id = create_gate("and", 100, 20, {color = "#ff0000", important = true, label = "test"})
```

### create_timer
timers are created with the `create_timer` function. it takes the inputs of `(seconds, ticks, x, y, {opts})` and optional arguements of `{color = hex}` and `{label = name}`
```lua
-- timer 10 seconds long at 0, 0
create_timer(10, 0, 0, 0)
-- blue timer at -120,30 being 0 seconds and 18 ticks long
id = create_timer(0, 18, -120, 30, {color = "#0000ff", label = "test"})
```

### create_input/create_output
both of these functions take the standard inputs of `(x, y, {opts})` and optional arguments of `{color = hex}` and `{label = name}`

### create_label
takes the inputs `(label, x, y, {opts})` and the same optional color input.

### modify_gate
takes the inputs `(id, {opts})`, with the optional arugments being `x`, `y`, `color`, `label`, `important`, or `type`. the data inputted is the same as outputted by `get_part`

### modify_timer
similar to modify_gate but instead includes optional arguments of `x`, `y`, `color`, `label`, `seconds`, or `ticks`.

### modify_other
same as the two before it, but only includes the optional arguments `x`, `y`, `color`, or `label`. you can modify any part with this function, you only need to use part specific modification functions if you are modifying part specific variables.

### add_connection
you can connect parts together with `add_connection`, inputs being `(from_part_id, to_part_id)`, where this will connect the `from_part` to the `to_part`. this currently does not support modules as they have multiple inputs. the id being used is the same as the part id returned from create_timer or create_gate and the one shown in properties on the left side bar.
```lua
-- connects gate1 to gate2
add_connection(gate1_id, gate2_id)
```

### get_part
takes an input of the part id, and returns any data about the part as well as extra data depending on part type.
```lua
-- base info
{
	"id" = id, 			 -- int
	"x" = xpos, 		 -- float
	"y" = ypos, 		 -- float
	"label" = label, 	 -- string
	"color" = hex color, -- string
}
-- gates
{
	"type": gate type ("and", "xnor", etc), -- string
	"important": gate importance,			-- bool
}
-- timers
{
	"type" = "timer",		  -- string
	"seconds" = # of seconds, -- int
	"ticks" = # of ticks,	  -- int
}
-- io
{
	"type" = "io",						 -- string
	"input" = wether is an input or not, -- bool
}
-- module
{
	"type" = "module", -- string
	-- no other data included atm
}
-- label
{
	"type" = "label",  -- string
	"xSize" = x scale, -- float
	"ySize" = y scale, -- float
}
```

### get_canvas
returns a list of parts and connections, formatted as below and takes no arguments.
```lua
{
	"parts" = {
		-- list of parts with same formatting as get_part()
	},
	"connections" = {
		{
			"from": part id -- int
			"to": part id 	-- int
		}
	}
}
```

### get_selection
returns all parts and connections selected, formatted the same as `get_canvas`, also takes no arguments.

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

### list_dir
takes an input of a directory and returns a table of all files and directories in that folder.

## examples

this example creates a 8x8 grid of gates based on the bits in lua_test.bin. all bits that are 1 flash on and off twice per second and are colored white, while all other bits are black. It then prints the number of parts and connecions selected.
```lua
-- main clock
timer = create_timer(0,18,-140,40, {color="#ff0000"})
nand_gate = create_gate("and", -140, 140) -- wrong type so we can fix it later to test modify_gate
and_gate = create_gate("and", -260, 60)

add_connection(timer, nand_gate)
add_connection(nand_gate, and_gate)
add_connection(and_gate, timer)

-- fix the nand gate
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
```
