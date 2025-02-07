import os, json, uuid, shutil, baseParts

if os.name == "nt":
    userFolder = os.environ['APPDATA']+"/Axolot Games/Scrap Mechanic/User/"
else:
    userFolder = os.path.expanduser("~")+"/.steam/steam/steamapps/compatdata/387990/pfx/drive_c/users/steamuser/Application Data/Axolot Games/Scrap Mechanic/User/"
    if not os.path.isdir(userFolder): userFolder = None

bpFolder = None
if userFolder:
    users = os.listdir(userFolder)
    if len(users) == 1: bpFolder = userFolder+users[0]+"/Blueprints/"

class positioning:
    def zeroZero(self, partDict, bp, important, init=False):
        if init:
            bp.importantCounter = 0
            return

        if important:
            partDict["pos"] = (bp.importantCounter,-1,0)
            bp.importantCounter += 1
        else:
            partDict["pos"] = (0,0,0)
    def line(self, partDict, bp, important, init=False):
        if init:
            bp.importantCounter = 0
            bp.unImportantCounter = 0
            return

        if important:
            partDict["pos"] = (bp.importantCounter,-1,0)
            bp.importantCounter += 1
        else:
            partDict["pos"] = (bp.unImportantCounter,0,0)
            bp.unImportantCounter += 1



def getIdFromName(name, folder=bpFolder):
    if folder:
        blueprints = os.listdir(folder)
        for blueprint in blueprints:
            if os.path.isdir(bpFolder+blueprint):
                with open(bpFolder+blueprint+"/description.json","r") as f:
                    if json.load(f)["name"] == name: return blueprint, bpFolder+blueprint
        return False
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.")

def createBluePrint(name,description='#{STEAM_WORKSHOP_NO_DESCRIPTION}',localId=None,icon="defaultIcon.png",bpFolder=bpFolder):
    if bpFolder:
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

        return bluePrint(path)
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.")

def overWriteBluePrint(id,bpFolder=bpFolder,byName=True):
    if bpFolder:
        if byName: id = getIdFromName(id,folder=bpFolder)
        if id:
            if os.path.isdir(bpFolder+id):
                return bluePrint(bpFolder+id)
        return False
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.")


class bluePrint:
    partList = []
    partExportFunctions = {
        baseParts.ids["gate"]: baseParts.gateExport
    }

    positioningMethod = positioning.zeroZero
    overwritePositioning = False
    ignoreUnknownParts = False
    seperateImportant = True

    def __init__(self, path):
        self.path = path
    
    def loadPartList(self,partList): self.partList = partList
    def jsonLoadsPartList(self,string): self.partList = json.loads(string)
    def jsonLoadPartList(self,file): self.partList = json.load(file)

    def genOutputDict(self):
        outputChildsList = []
        self.positioningMethod(None,None,None,init=True)
        for part in partList:
            if part["pos"] == None or self.overwritePositioning:
                self.positioningMethod(part, self, part["important"] and self.seperateImportant)
                # TODO
        return {
            "bodies":[
                {"childs":outputChildsList}
            ],
            "version":4
        }