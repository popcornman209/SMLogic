import json

ids = {
    'gate': '9f0f56e8-2c31-4d83-996c-d00a9b296c3f',
    'timer': '8f7fd0e7-c46e-4944-a414-7ce2437bb30f'
}

class base:
    inputCons = [] #list of gates inside contraption to connect to
    outputCons = [] #list of output gates to connect to something elses inputs
    connections = [] #list of objects connected to
    connectionsFrom = [] #list of objects that are connceted to this
    important = False #display in simulator, to see values for debugging or just general testing.
    partID = None

    def connect(self, reciever, ignoreMisMatch=False):
        if len(self.outputCons) == len(reciever.inputCons) or ignoreMisMatch: # TODO work on missmatch, also spell correctly
            conLength = min(len(self.outputCons),len(reciever.inputCons))
            self.connections = reciever.inputCons[:conLength]
            reciever.connectionsFrom = self.outputCons[:conLength]
            for out, inp in zip(self.outputCons[:conLength],self.connections[:conLength]):
                for outCon in (out if type(out) == list else [out]):
                    for inpCon in (inp if type(inp) == list else [inp]):
                        if outCon != self: outCon.connect(inpCon)
        else: raise IndexError("outputs dont match inputs! add ignoreMisMatch = True to connect to ignore.")

    def identifyRelationships(self, bp):
        for part in self.inputCons: part.identify(bp)
        for part in self.outputCons: part.identify(bp)
        for part in self.connections: part.identify(bp)
        for part in self.connectionsFrom: part.identify(bp)

    def identify(self,bp):
        if self.partID == None:
            self.identifyRelationships(bp)
            self.partID = -1
    
    def genNetworkDict(self): raise RuntimeError("part has no genNetworkDict method!, only gates and timers can compile")

class gate(base):
    def __init__(self,type="and",color="222222",pos=None):
        if type not in ["and","or","xor","xnor","nor","nand"]: raise TypeError(f"logic gate type cannot be {type}!")

        self.color = color
        self.type = type
        self.pos = pos

        self.inputCons = [self] #list of gates inside contraption to connect to
        self.outputCons = [self] #list of output gates to connect to something elses inputs

    def identify(self,bp):
        if self.partID == None:
            for part in self.connections: part.identify(bp)
            self.partID = bp.currentId
            bp.currentId += 1
            bp.partList.append(self)
    
    def genNetworkDict(self):
        return {
            "part": ids['gate'],
            "type": self.type,
            "color": self.color,
            "connections": [part.partID for part in self.connections],
            "connectionsFrom": [part.partID for part in self.connectionsFrom],
            "pos": self.pos
        }

class bluePrint:
    def __init__(self,mainPart: base, removeNoConnections=False):
        self.currentId = 1
        self.partList = []
        self.mainPart = mainPart
        self.network = {}
        self.removeNoConnections = removeNoConnections
    
    def compile(self):
        self.mainPart.identify(self)
        self.generateNetwork()
    
    def exportJson(self,file):
        with open(file,"w") as f: json.dump(self.network,f)
    
    def removeNoConnection(self,part):
        if len(self.network[part]["connections"]) == 0:
            for parent in self.network[part]["connectionsFrom"]:
                self.removeNoConnection(parent)
            self.network.pop(part)

    def generateNetwork(self):
        for part in self.partList:
            self.network[part.partID] = part.genNetworkDict()
        if self.removeNoConnections:
            partsToRemove = [(part if val["connections"] == 0 else None) for part,val in self.network.items()]
            #self.removeNoConnection(part)