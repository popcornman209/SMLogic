import json, datetime

ids = {
    'gate': '9f0f56e8-2c31-4d83-996c-d00a9b296c3f',
    'timer': '8f7fd0e7-c46e-4944-a414-7ce2437bb30f'
}

log = False
def log(string): #basic log function
    time = datetime.now()
    print("{}:{}:{} ; {}".format(time.hour,time.minute,time.second,string))

class base:
    inputCons = [] #list of gates inside contraption to connect to
    outputCons = [] #list of output gates to connect to something elses inputs
    connections = [] #list of objects connected to
    connectionsFrom = [] #list of objects that are connceted to this
    forceKeep = False
    important = False #display in simulator, to see values for debugging or just general testing.
    partType = "container"
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
        if self.partType == "container": l = bp.containerList
        else: l = bp.partList

        if self not in l:
            l.append(self)
            self.identifyRelationships(bp)

class gate(base):
    def __init__(self,type="and",color="222222",pos=None):
        if type not in ["and","or","xor","xnor","nor","nand"]: raise TypeError(f"logic gate type cannot be {type}!")
        
        self.color = color
        self.type = type
        self.pos = pos

        self.inputCons = [self] #list of gates inside contraption to connect to
        self.outputCons = [self] #list of output gates to connect to something elses inputs
        self.partType = "gate"

class bluePrint:
    def __init__(self,mainPart: base, removeDeadEnds=True):
        self.partList = []
        self.containerList = []
        self.mainPart = mainPart

        self.removeDeadEnds = removeDeadEnds

        self.inputs = self.mainPart.inputCons
        self.outputs = self.mainPart.outputCons
        for out in self.outputs:
            if type(out) == list: raise TypeError("there cant be mutlible logic gates per output bit on the main part!")
            out.forceKeep = True
        for inp in self.inputs:
            if type(inp) == list: raise TypeError("there cant be mutlible logic gates per input bit on the main part!")
            inp.forceKeep = True

    def compile(self):
        self.mainPart.identify(self)
        if self.removeDeadEnds:
            for part in self.partList.copy(): self.removeNoConnection(part)

    def removeNoConnection(self,part: base):
        if part.connections == [] and not part.forceKeep:
            for parent in part.connectionsFrom:
                parent.connections.remove(part)
                self.removeNoConnection(parent)
            self.partList.remove(part)