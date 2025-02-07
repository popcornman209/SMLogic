import os, json, uuid, shutil

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
        return False
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.")

def createBluePrint(name,description='#{STEAM_WORKSHOP_NO_DESCRIPTION}',localId=None,icon="defaultIcon.png",bpFolder=bpFolder):
    if not localId: localId = uuid.uuid4()
    path = bpFolder+localId
    if os.path.isdir(path): raise FileExistsError("blueprint already exists!")

    os.mkdir(path)
    shutil.copy2(icon,path+"/icon.png")
    with open(path+"/description.json","w") as f:
        json.dump({
            "description" : description,
            "localId" : localId,
            "name" : name,
            "type" : "Blueprint",
            "version" : 0
        },f)

class bluePrint:
    def __init__(self,path):
        self.path = path