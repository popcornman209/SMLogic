import SMLogic as sml

class gate:
    def __init__(self,dict,bp):
        self.type = dict["type"]
        self.important = dict["important"]
        self.consFrom = dict["consFrom"]
        self.bp = bp
    
    def sMidFrame(self):
        self.inputs = []

class bp:
    def __init__(self): pass