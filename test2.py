import SMLogic as sml

gates = 2000
conPerGate = 2

class testPart(sml.base):
    def __init__(self):
        for i in range(gates):
            g = sml.gate()
            if i > 3:
                g.connect(self.inputCons[i-1])
                g.connect(self.inputCons[i-2])
            self.inputCons.append(g)

p = testPart()
print("a")
bp = sml.bluePrint(p,removeDeadEnds=False)
bp.compile()
#bp.simulate(1)