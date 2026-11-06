-- finds rows of parts, then detects any part in that row with non
-- default colors, that parts label and color is then copied to
-- the rest of the parts in the row.

parts = get_selection()["parts"]

yGroups = {}
colors = {}
labels = {}
for i, part in ipairs(parts) do
	foundGroup = false
	for y, group in pairs(yGroups) do
		if part["y"] == y then
			table.insert(yGroups[y], part)
			foundGroup = true
			break
		end
	end
	if foundGroup == false then
		yGroups[part["y"]] = {part}
	end
	if part["color"] ~= "#df7f01" and part["color"] ~= "#df7f00" then
		colors[part["y"]] = part["color"]
		labels[part["y"]] = part["label"]
	end
end
for y, group in pairs(yGroups) do
	color = colors[y]
	label = labels[y]
	for i, part in ipairs(group) do
		modify_other(part["id"], {color = color, label = label})
	end
end