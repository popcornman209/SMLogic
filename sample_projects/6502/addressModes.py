from enum import IntEnum

class modeID(IntEnum):
    AC = 0
    ABS = 1
    ABSX = 2
    ABSY = 3
    IMM = 4
    IMPLIED = 5
    IND = 6
    INDX = 7
    INDY = 8
    REL = 9
    ZPG = 10
    ZPGX = 11
    ZPGY = 12

def absx(p): #abs,X – Absolute address + X
    addr = (p.fetch_word() + p.x) & 0xFFFF
    return modeID.ABSX, addr, p.memory[addr]

def absy(p): #abs,Y – Absolute address + Y
    addr = (p.fetch_word() + p.y) & 0xFFFF
    return modeID.ABSY, addr, p.memory[addr]

def imm(p): # – Immediate: operand is next byte
    return modeID.IMM, None, p.fetch_byte()

def ind(p):
    ptr = p.fetch_word()
    target = p.memory.get16(ptr)  # Use get16 to read 16-bit address
    return modeID.IND, target, None

def indx(p): #(X,ind) – Indexed Indirect: fetch pointer from zp + X
    zp = p.fetch_byte() + p.x
    addr = p.memory.get16(zp)
    return modeID.INDX, addr, p.memory[addr]

def indy(p): #(ind),Y – Indirect Indexed: fetch pointer, add Y
    zp = p.fetch_byte()
    base_addr = p.memory.get16(zp)
    addr = base_addr + p.y
    return modeID.INDY, addr, p.memory[addr]

def rel(p):
    offset = p.fetch_byte().astype(int)  # or int(offset)
    if offset >= 0x80:
        offset -= 0x100
    addr = (int(p.pc) + offset) & 0xFFFF
    return modeID.REL, addr, None

def zpg(p): #zpg – Zero page address
    addr = p.fetch_byte()
    return modeID.ZPG, addr, p.memory[addr]

def zpgx(p): #zpg,X – Zero page address + X (no carry)
    addr = (p.fetch_byte() + p.x) & 0xFF
    return modeID.ZPGX, addr, p.memory[addr]

def zpgy(p): #zpg,Y – Zero page address + Y (no carry)
    addr = (p.fetch_byte() + p.y) & 0xFF
    return modeID.ZPGY, addr, p.memory[addr]
