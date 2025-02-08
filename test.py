import SMLogic as sml
import exporter as exp
import baseParts as bParts

sml.logInfo = True

class test1(sml.base):
    def __init__(self):
        super().__init__()
        self.g1 = bParts.gate(0)
        self.g2 = bParts.gate("or")
        self.g1.connect(self.g2)
        self.inputCons = [self.g1,self.g2]
        self.outputCons = [self.g2]

class test2(sml.base):
    def __init__(self):
        super().__init__()
        self.g1 = bParts.gate("xnor")
        self.g2 = bParts.gate("xor")
        self.inputCons = [self.g1,self.g2]

t1 = test1()
t2 = test2()

t1.connect(t2,ignoreMisMatch=True)

bp = sml.bluePrint(t1,removeDeadEnds=False)
bp.compile()

expBP = exp.overWriteBluePrint("output",byName=True)
expBP.loadNetwork(bp.getNetwork())
expBP.export()