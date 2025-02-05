import json

gate = "9f0f56e8-2c31-4d83-996c-d00a9b296c3f"
fp = "/home/leo/.local/share/Steam/steamapps/compatdata/387990/pfx/drive_c/users/steamuser/AppData/Roaming/Axolot Games/Scrap Mechanic/User/User_76561198280907739/Blueprints/"
cpu2a = "6022bb43-d11e-48ec-8674-66460691ebb6"

with open(f"{fp}/{cpu2a}/blueprint.json","r") as f:
    data = json.load(f)

gates = 0
cons = 0
for part in data["bodies"][0]["childs"]:
    if part["shapeId"] == gate:
        gates += 1
        if part["controller"]["controllers"]: cons += len(part["controller"]["controllers"])

print(gates)#1527
print(cons)#2490