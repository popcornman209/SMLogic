import os, json

if os.name == "nt":
    userFolder = os.environ['APPDATA']+"/Axolot Games/Scrap Mechanic/User/"
else:
    userFolder = "~/.steam/steam/steamapps/compatdata/387990/pfx/drive_c/users/steamuser/Application Data/Axolot Games/Scrap Mechanic/User/"
    if not os.path.isdir(userFolder): userFolder = None

bpFolder = None
if userFolder:
    users = os.listdir(userFolder)
    if len(users) == 1: bpFolder = userFolder+users[0]+"/Blueprints/"

def getIdFromName(name, folder=bpFolder):
    if folder:
        blueprints = os.listdir(folder)
        for blueprint in blueprints:
            with open(bpFolder+blueprint+"/description.json","r") as f:
                if json.load(f)["name"] == name: return blueprint, bpFolder+blueprint
    else: raise FileNotFoundError("bp folder not provided or found automatically! set SMExporter.bpFolder to your blueprints folder.")