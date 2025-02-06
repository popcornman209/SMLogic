import SMLogic as sml
from baseParts import *

sml.logInfo = True

class test1(sml.base):
    def __init__(self):
        self.g1 = gate(0)
        self.g2 = gate("or")
        self.g1.connect(self.g2)
        self.inputCons = [self.g1,self.g2]
        self.outputCons = [self.g2]

class test2(sml.base):
    def __init__(self):
        self.g1 = gate("xnor")
        self.g2 = gate("xor")
        self.inputCons = [self.g1,self.g2]

t1 = test1()
t2 = test2()

t1.connect(t2,ignoreMisMatch=True)

bp = sml.bluePrint(t1,removeDeadEnds=False)
bp.compile()
print(bp.jsonDumps())
input()