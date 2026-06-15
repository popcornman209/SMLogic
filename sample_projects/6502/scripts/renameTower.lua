-- variables
name_format = "ac %s"





-- script
parts = get_selection()["parts"]

table.sort(parts, function(a,b)
	return a["y"] > b["y"]
end)

for i, part in ipairs(parts) do
	modify_other(part["id"], {label = string.format(name_format,i-1)})
end