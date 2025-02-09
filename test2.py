import SMLogic as sml
import baseParts as bsp

gates = 4
conPerGate = 1

class testPart(sml.base):
    def __init__(self):
        super().__init__()
        for i in range(gates):
            g = bsp.gate(0)
            if i > conPerGate:
                for j in range(conPerGate):
                    g.connect(self.inputCons[i-j-1])
            self.inputCons.append(g)

p = testPart()
bp = sml.bluePrint(p,removeDeadEnds=False)
bp.compile()
with open('logic_sim/gates.json',"w") as f:
    bp.jsonDump(f)