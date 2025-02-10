import os, json, uuid, shutil, baseParts

if os.name == "nt": #if on windows (untested)
    userFolder = os.environ['APPDATA']+"/Axolot Games/Scrap Mechanic/User/" #that is the sm folder
else: #otherwise (on linux)
    userFolder = os.path.expanduser("~")+"/.steam/steam/steamapps/compatdata/387990/pfx/drive_c/users/steamuser/Application Data/Axolot Games/Scrap Mechanic/User/"

if not os.path.isdir(userFolder): userFolder = None #if that folder doesnt exist, on another drive or something

bpFolder = None
if userFolder:
    users = os.listdir(userFolder) #get all users in the folder
    if len(users) == 1: bpFolder = userFolder+users[0]+"/Blueprints/" #if theres only one then find the blueprint folder of that user

class positioning: #class that stores different positioning methods
    def zeroZero(partDict, bp, important, init=False): #everything at zero zero
        if init: #if initializing
            bp.importantCounter = 0 #x value for important gates
            return

        if important: #if the gate is important
            partDict["pos"] = (bp.importantCounter,-1,0) #put it on the "important line"
            bp.importantCounter += 1 #move the next important gate over
        else: #if its not important
            partDict["pos"] = (0,0,0) #put it at zero zero
    def line(partDict, bp, important, init=False): #line of gates
        if init:
            bp.importantCounter = 0 #line of important gates
            bp.unImportantCounter = 0 #line of unimportant gates
            return

        if important:
            partDict["pos"] = (bp.importantCounter,-1,0) #position important gates
            bp.importantCounter += 1
        else:
            partDict["pos"] = (bp.unImportantCounter,0,0) #position unimportant
            bp.unImportantCounter += 1


def getIdFromName(name, folder=bpFolder): #getting part id from the name
    if folder: #if the folder exists
        blueprints = os.listdir(folder) #list of blueprints
        for blueprint in blueprints: #go through each blueprint
            if os.path.isdir(folder+blueprint): #if its a folder, no files
                with open(folder+blueprint+"/description.json","r") as f: #open the description file
                    if json.load(f)["name"] == name: return blueprint, folder+blueprint #if the name checks then return the blueprint id, and path
        return False #not found
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.") #blueprint folder not specified

def createBluePrint(name,description='#{STEAM_WORKSHOP_NO_DESCRIPTION}',localId=None,icon="defaultIcon.png",bpFolder=bpFolder): #create new blueprint
    if bpFolder: #if folder specified
        if not localId: localId = uuid.uuid4() #generate new id
        path = bpFolder+localId #make the path of the bp
        if os.path.isdir(path): raise FileExistsError("blueprint already exists!") #if the blueprint is already there

        os.mkdir(path) #make new folder for blueprint
        shutil.copy2(icon,path+"/icon.png") #copy icon over to it
        with open(path+"/description.json","w") as f: #open description file
            json.dump({ #dump information
                "description" : description,
                "localId" : localId,
                "name" : name,
                "type" : "Blueprint",
                "version" : 0
            },f)

        return bluePrint(path) #return blueprint
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.")

def overWriteBluePrint(id,bpFolder=bpFolder,byName=True): #overwrite existing blueprint
    if bpFolder: #if folder specified
        if byName: id, path = getIdFromName(id,folder=bpFolder) #if finding the blueprint by name do so
        if id: #if id found, getIdFromName can return false if not found
            if os.path.isdir(path): #if that blueprint exists
                return bluePrint(path) #return the blueprint object
        return False
    else: raise FileNotFoundError("bp folder not provided or found automatically! set exporter.bpFolder to your blueprints folder.")


class bluePrint:
    def __init__(self, path):
        self.path = path #path of blueprint folder

        self.partList = [] #list of parts
        self.partExportFunctions = { #functions used to generate part dictionaries, different parts can be different in scrap mechanic
            baseParts.ids["gate"]: baseParts.gateExport, #gate function
            baseParts.ids["timer"]: baseParts.timerExport #timer function
        }

        self.positioningMethod = positioning.zeroZero #default positioning method
        self.overwritePositioning = False #overwrite position if manually specified on each part
        self.ignoreUnknownParts = False #ignore parts not known, otherwise will throw error if theres a unknown part
        self.seperateImportant = True #seperate the important gates, for inputs and ouputs and things
    
    def loadNetwork(self,partList): self.partList = partList #load network dictionary
    def jsonLoadsNetwork(self,string): self.partList = json.loads(string) #load network from json string
    def jsonLoadNetwork(self,file): self.partList = json.load(file) #load network from json file

    def genOutputDict(self): #generate output dictionary
        outputChildsList = [] #list of children, or parts/blocks. no support for joints atm.
        self.positioningMethod(None,self,None,init=True) #initialize positioning method
        for part in self.partList: #loop through each part
            if part["pos"] == None or self.overwritePositioning: self.positioningMethod(part, self, part["important"] and self.seperateImportant) #if part should be positioned, do so.
            if part["part"] in self.partExportFunctions: #if the part has a export function
                outputChildsList.append(self.partExportFunctions[part["part"]](part)) #run it and add it to the children list
            elif self.ignoreUnknownParts == False: raise RuntimeError(f"part {part["part"]} not in partExportFunctions, cant export") #throw an error if part unknown and not ignored
        return { #return dictionary
            "bodies":[
                {"childs":outputChildsList}
            ],
            "version":4
        }
    
    def export(self): #export to file
        with open(self.path+"/blueprint.json","w") as f: #open blueprint.json file
            json.dump(self.genOutputDict(),f) #dump the information to it