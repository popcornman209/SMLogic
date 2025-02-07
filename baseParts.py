import SMLogic as sml

ids = { #part ids
    'gate': '9f0f56e8-2c31-4d83-996c-d00a9b296c3f',
    'timer': '8f7fd0e7-c46e-4944-a414-7ce2437bb30f'
}

class gate(sml.base): #logic gate
    modes = ["and","or","xor","nand","nor","xnor"]  #all possible modes
    partType = "gate"                               #part type to gate, its not a container

    def __init__(self,mode ,color="222222",pos=None): #initialise function
        if mode not in self.modes and mode > 5 and mode < 0: raise TypeError(f"logic gate mode cannot be {mode}!") # if its a valid mode for the gate to be in
        
        self.color = color #set the color
        if type(mode) == int: self.mode = mode      #set the mode if number provided
        else: self.mode = self.modes.index(mode)    #otherwise turn the string into a number
        self.pos = pos #set the pos

        self.inputCons = [self]     #list of gates inside contraption to connect to
        self.outputCons = [self]    #list of output gates to connect to something elses inputs
    
    def dumpDict(self, bp: sml.bluePrint): #dictionary that it returns
        return {
            "part": ids["gate"],            #sets part id to the gates
            "id": bp.getPartId(self),       #id
            "color": self.color,            #color
            "pos": self.pos,                #color
            "mode": self.mode,              #gate mode, and or etc
            "important": self.important,    #wether its importatnt or not
            "connections": [bp.getPartId(part) for part in self.connections],           #things its connected to
            "connectionsFrom": [bp.getPartId(part) for part in self.connectionsFrom]   #things connected to it
            
        }
    
def genSMOutput(self,gateDict):
    return {
        "color": gateDict["color"],
        "controller":{
            "active": False,
            "controllers":gateDict["connections"],
            "id": gateDict["id"],
            "joints":None,
            "mode":type                                 #TODO
        },
        "pos":{"x":pos[0],"y":pos[1],"z":pos[2]},
        "shapeId": gateDict["part"],
        "xaxis":1,
        "zaxis":-2
    }