import json, datetime, time, threading, math

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
    active = False

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
        

    def identify(self,bp):
        bp.recursion -= 1
        if self.partType == "container": l = bp.containerList
        else: l = bp.partList

        if self not in l and bp.recursion > 0:
            l.append(self)
            for part in self.inputCons: part.identify(bp)
            for part in self.outputCons: part.identify(bp)
            for part in self.connections: part.identify(bp)
            for part in self.connectionsFrom: part.identify(bp)
        bp.recursion += 1
    
    def dumpDict(self):
        raise RuntimeError("cant dump container dictoinary!")

    def simulate(self,midFrame): raise RuntimeError("cant simulate an entire container, if your making a part add a simulate method!")

class gate(base):
    modes = ["and","or","xor","nand","nor","xnor"]
    partType = "gate"

    def __init__(self,mode=0,color="222222",pos=None):
        if mode not in self.modes and mode > 5 and mode < 0: raise TypeError(f"logic gate mode cannot be {mode}!")
        
        self.color = color
        if type(mode) == int: self.mode = mode
        else: self.mode = self.modes.index(mode)
        self.pos = pos

        self.inputCons = [self] #list of gates inside contraption to connect to
        self.outputCons = [self] #list of output gates to connect to something elses inputs
    
    # def dumpDict(self, bp):
    #     return {
    #         "part": ids["gate"],
    #         "color": self.color,
    #         "pos": self.pos,
    #         "mode": self.mode,
    #         "connections": [bp.getPartId(part) for part in self.connections],
    #         "connectionsFrom": [bp.getPartId(part) for part in self.connectionsFrom],
    #         "important": self.important
    #     }
    
    def simulate(self,midFrmae):
        if midFrmae:
            self.sInputs = [part.active for part in self.connectionsFrom]
        else:
            if self.mode == 0 or self.mode == 3: self.active = all(self.sInputs)
            elif self.mode == 0 or self.mode == 3: self.active = any(self.sInputs)
            elif self.mode == 0 or self.mode == 3: self.active = sum(self.sInputs) % 2 == 1
            if self.mode >= 3: self.active = not self.active

class simThread: # TODO
    def __init__(partList):
        self.t = threading.Thread()
        self.partList = partList
    
    def simulateMidFrame(self):
        for part in parts: part.simulate(True)
    def simulateFullFrame(self):
        for part in parts: part.simulate(True)

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

    # def getNetwork(self): return [part.dumpDict(self) for part in self.partList]

    # def getPartId(self,part): return self.partList.index(part)

    def compile(self):
        self.recursion = 900
        self.mainPart.identify(self)
        self.threadPartLists = self.partList
        if self.removeDeadEnds:
            for part in self.partList.copy(): self.removeNoConnection(part)

    def removeNoConnection(self,part: base):
        if part.connections == [] and not part.forceKeep:
            for parent in part.connectionsFrom:
                parent.connections.remove(part)
                self.removeNoConnection(parent)
            self.partList.remove(part)
    
    def splitThreads(self,threads):
        alreadyAssigned = 0
        self.threadPartLists = []
        for i in range(threads):
            partsInThread = math.ceil(len(self.partList[alreadyAssigned:])/(threads-i))
            self.threadPartLists.append(self.partList[alreadyAssigned:partsInThread+alreadyAssigned])
            alreadyAssigned += partsInThread
        
    def simThread(self,parts,midFrame):
        for part in parts: part.simulate(midFrame)
    
    def simulateFrame(self):
        for thread in self.threadPartLists:
            t = threading.Thread(target=lambda: self.simThread(thread,True))
            t.start()