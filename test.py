import SMLogic as sml

class test1(sml.base):
    def __init__(self):
        self.g1 = sml.gate(type="and")
        self.g2 = sml.gate(type="or")
        self.g1.connect(self.g2)
        self.inputCons = [self.g1,self.g2]
        self.outputCons = [self.g2]

class test2(sml.base):
    def __init__(self):
        self.g1 = sml.gate(type="xnor")
        self.g2 = sml.gate(type="xor")
        self.inputCons = [self.g1,self.g2]

t1 = test1()
t2 = test2()

t1.connect(t2,ignoreMisMatch=True)

bp = sml.bluePrint(t1,removeNoConnections=True)
bp.compile()
input()