ids = {
    'gate': '9f0f56e8-2c31-4d83-996c-d00a9b296c3f',
    'timer': '8f7fd0e7-c46e-4944-a414-7ce2437bb30f'
}

class base:
    inputCons = [] #list of gates inside contraption to connect to
    outputCons = [] #list of output gates to connect to something elses inputs
    connections = [] #list of objects connected to
    important = False #display in simulator, to see values for debugging or just general testing.
    partID = None

    def connect(self, reciever: str, ignoreMisMatch=False):
        if len(self.outputCons) == len(reciever.inputCons) or ignoreMisMatch: # TODO work on missmatch, also spell correctly
            conLength = min(len(outputCons),len(reciever.inputCons))
            self.connections = reciever.inputCons
            for out, inp in zip(self.outputCons,self.connections):
                for outCon in (out if type(out) == list else [out]):
                    for inpCon in (inp if type(inp) == list else [inp]):
                        outCon.connect(inpCon)
        else: raise IndexError("outputs dont match inputs! add ignoreMisMatch = True to connect to ignore.")

class gate(base):
    def __init__(self,type="and",color="222222"):
        if type not in ["and","or","xor","xnor","nor","nand"]: raise TypeError(f"logic gate type cannot be {type}!")

        self.color = color
        self.type = type

        self.inputCons = [self] #list of gates inside contraption to connect to
        self.outputCons = [self] #list of output gates to connect to something elses inputs

class bluePrint:
    test = 5