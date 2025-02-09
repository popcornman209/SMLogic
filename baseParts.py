import SMLogic as sml

ids = { #part ids
    'gate': '9f0f56e8-2c31-4d83-996c-d00a9b296c3f',
    'timer': '8f7fd0e7-c46e-4944-a414-7ce2437bb30f'
}

class gate(sml.base): #logic gate
    def __init__(self,mode ,color="222222",pos=None, axis=(1,-2)): #initialise function
        super().__init__()
        self.modes = ["and","or","xor","nand","nor","xnor"]  #all possible modes
        self.partType = "gate"                               #part type to gate, its not a container

        if mode not in self.modes and mode > 5 and mode < 0: raise TypeError(f"logic gate mode cannot be {mode}!") # if its a valid mode for the gate to be in
        
        self.color = color #set the color
        if type(mode) == int: self.mode = mode      #set the mode if number provided
        else: self.mode = self.modes.index(mode)    #otherwise turn the string into a number
        self.pos = pos #set the pos
        self.axis = axis

        self.inputCons = [self]     #list of gates inside contraption to connect to
        self.outputCons = [self]    #list of output gates to connect to something elses inputs
        
    
    def dumpDict(self, bp: sml.bluePrint): #dictionary that it returns
        if len(self.connectionsFrom) > 255: raise RuntimeError("gates can only have up to 255 connections! (sad ik)")
        return {
            "part": ids["gate"],            #sets part id to the gates
            "id": bp.getPartId(self),       #id
            "color": self.color,            #color
            "pos": self.pos,                #color
            "axis": self.axis,              #rotation, formated (xaxis, zaxis)
            "mode": self.mode,              #gate mode, and or etc
            "important": self.important,    #wether its importatnt or not
            "connections": [bp.getPartId(part) for part in self.connections],           #things its connected to
            "connections_from": [bp.getPartId(part) for part in self.connectionsFrom]   #things connected to it
            
        }
    
def gateExport(gateDict):
    return {
        "color": gateDict["color"],
        "controller":{
            "active": False,
            "controllers":[{"id":id} for id in gateDict["connections"]],
            "id": gateDict["id"],
            "joints":None,
            "mode": gateDict["mode"]
        },
        "pos":{"x":gateDict["pos"][0],"y":gateDict["pos"][1],"z":gateDict["pos"][2]},
        "shapeId": gateDict["part"],
        "xaxis":gateDict["axis"][0],
        "zaxis":gateDict["axis"][1]
    }