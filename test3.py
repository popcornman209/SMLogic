import SMLogic as sml
import baseParts as bsp

class testPart(sml.base):
    def __init__(self):
        super().__init__()
        g1 = bsp.gate(0)
        g2 = bsp.gate(3)
        g3 = bsp.gate(0)
        g4 = bsp.gate(0)

        g1.connect(g2)
        g2.connect(g3)
        g3.connect(g4)

        self.outputCons = [g3]

p = testPart()
bp = sml.bluePrint(p,removeDeadEnds=False)
bp.compile()
with open('logic_sim/gates.json',"w") as f:
    bp.jsonDump(f)