import json, datetime

logInfo = False     #should log information to console or not
def log(string):    #basic log function
    if logInfo:
        time = datetime.datetime.now()
        print("{}:{}:{} ; {}".format(time.hour,time.minute,time.second,string))

class base:
    def __init__(self):
        self.inputCons = []          #list of gates inside contraption to connect to
        self.outputCons = []         #list of output gates to connect to something elses inputs
        self.connections = []        #list of objects connected to
        self.connectionsFrom = []    #list of objects that are connceted to this
        self.forceKeep = False       #wether to forcefully keep part, bypasses remove dead ends
        self.important = False       #display in simulator or seperate in exporter, to see values for debugging or just general testing, or as inputs or outputs
        self.partType = "container"  #type of object, if isnt "container" it will be added to part list and exported. dont change unless you have a dump dict method

    def connect(self, reciever, ignoreMisMatch=False):
        if len(self.outputCons) == len(reciever.inputCons) or ignoreMisMatch: #if connections match, or it doesnt matter
            conLength = min(len(self.outputCons),len(reciever.inputCons)) #length of connection, if one has 2 ouputs and other has 3 inputs only 2 will be used

            self.connections += reciever.inputCons[:conLength]      #adds connections to list of things connected to
            reciever.connectionsFrom += self.outputCons[:conLength] #adds connections from a part, aka the parent connected to it. used for identifying all parts and simulating faster

            if self.outputCons != [self]:
                for out, inp in zip(self.outputCons[:conLength],self.connections[:conLength]):      #for each output and input
                    for outCon in (out if type(out) == list else [out]):                            #go through each gate of one parts outputs
                        for inpCon in (inp if type(inp) == list else [inp]):                        #go through each gate of the others inputs
                            if outCon != self: outCon.connect(inpCon)                               #connect them to each other
        else: raise IndexError("outputs dont match inputs! add ignoreMisMatch = True to connect to ignore.") #if there are more inputs than outputs and vice versa
        
    def identify(self,bp): #for indentifying the part and saving it to the blueprint
        bp.recursion -= 1 #lowers the current recursion level
        if self.partType == "container": l = bp.containerList #if this is a container, add it to that
        else: l = bp.partList #otherwise go in the parts list

        if self not in l and bp.recursion > 0: #if it hasnt already been indentified and recursion hasnt hit its limit yet
            l.append(self) #add to the list
            for part in self.inputCons: part.identify(bp)       #identify all inputs of this object
            for part in self.outputCons: part.identify(bp)      #identify all the ouputs of this object
            for part in self.connections: part.identify(bp)     #identify everything this object is connected to
            for part in self.connectionsFrom: part.identify(bp) #idenfity everything connected to this object
        elif bp.recursion <= 0:
            log("reached max recursion depth, skipping.") #reached max depth
        bp.recursion += 1
    
    def dumpDict(self): #used for parts, containers cannot do this as its not a part.
        raise RuntimeError("cant dump container dictionary! if your making a part, add a dumpDict method.")


class bluePrint:
    def __init__(self,mainPart: base, removeDeadEnds=True):
        self.mainPart = mainPart                #main part, this will be used for the inputs and outputs of a object
        self.removeDeadEnds = removeDeadEnds    #wether to remove dead ends, paths that lead nowhere and dont effect the output

        self.maxRecursion = 900      #max depth for recursion,
        self.partList = []           #list of parts, empty until compiled
        self.containerList = []      #list of containers, not used for much except knowing which containers have been checked

        for out in self.mainPart.outputCons:
            if type(out) == list: raise TypeError("there cant be mutlible logic gates per output bit on the main part!") #if one output bit has multible gates assigned to it
            out.forceKeep = True #force keep the output
            out.important = True #make it important
        for inp in self.mainPart.inputCons:
            if type(inp) == list: raise TypeError("there cant be mutlible logic gates per input bit on the main part!") #if one input bit has multible gates assigned to it
            inp.forceKeep = True #force keep the input
            inp.important = True #make it important

    def getNetwork(self): return [part.dumpDict(self) for part in self.partList]    #get network dictionary, for simulating and exporting to sm
    def jsonDumps(self): return json.dumps(self.getNetwork())                       #dump dictionary to a json string
    def jsonDump(self,f): json.dump(self.getNetwork(),f)                            #dump dictionary to a file

    def getPartId(self,part): return self.partList.index(part)                      #get a parts id, just where it is in a list. is subject to change if partlist changes.

    def compile(self): #identifies all parts, should be run after the entire part is made and assembled
        self.recursion = self.maxRecursion      #sets current recursion
        self.mainPart.identify(self)            #starts identifying all parts (starts at the main one and travels through all connections)
        if self.removeDeadEnds:                 #if removing all deadends
            for part in self.partList.copy(): self.removeNoConnection(part) #go through all objects and remove dead ends

    def removeNoConnection(self,part: base): #for removing dead ends
        if part.connections == [] and not part.forceKeep:   #if the part isnt connected to anything (doesnt effect output) and it isnt forcefully kept (main part outputs)
            for parent in part.connectionsFrom:             #go through all parents
                parent.connections.remove(part)             #remove dead end part from its connections
                self.removeNoConnection(parent)             #and check if it is now a dead end too
            self.partList.remove(part)                      #remove the part
            log("removed dead end")