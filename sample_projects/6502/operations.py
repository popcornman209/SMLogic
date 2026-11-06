from numpy import uint8, uint16, seterr
from p6502 import addressModes as am

seterr(over="ignore")

class ADC:
    opcodes = {
        0x69: lambda p: ADC.base(p, am.imm(p)),     # ADC Immediate
        0x65: lambda p: ADC.base(p, am.zpg(p)),     # ADC Zero Page
        0x75: lambda p: ADC.base(p, am.zpgx(p)),    # ADC Zero Page,X
        0x6D: lambda p: ADC.base(p, am.abs(p)),     # ADC Absolute
        0x7D: lambda p: ADC.base(p, am.absx(p)),    # ADC Absolute,X
        0x79: lambda p: ADC.base(p, am.absy(p)),    # ADC Absolute,Y
        0x61: lambda p: ADC.base(p, am.indx(p)),    # ADC (Indirect,X)
        0x71: lambda p: ADC.base(p, am.indy(p)),    # ADC (Indirect),Y
    }
    
    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = uint16(p.ac) + uint16(v) + (1 if p.flags.carry else 0)
        result8 = uint8(result)

        # Set flags
        p.flags.carry = result > 0xFF
        p.flags.zero = (result8 == 0)
        p.flags.negative = (result8 & 0x80) != 0
        p.flags.overflow = ((~(p.ac ^ v)) & (p.ac ^ result8) & 0x80) != 0

        p.ac = uint8(result8)

class AND:
    opcodes = {
        0x29: lambda p: AND.base(p, am.imm(p)),     # AND Immediate
        0x25: lambda p: AND.base(p, am.zpg(p)),     # AND Zero Page
        0x35: lambda p: AND.base(p, am.zpgx(p)),    # AND Zero Page,X
        0x2D: lambda p: AND.base(p, am.abs(p)),     # AND Absolute
        0x3D: lambda p: AND.base(p, am.absx(p)),    # AND Absolute,X
        0x39: lambda p: AND.base(p, am.absy(p)),    # AND Absolute,Y
        0x21: lambda p: AND.base(p, am.indx(p)),    # AND (Indirect,X)
        0x31: lambda p: AND.base(p, am.indy(p)),    # AND (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.ac & v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        p.ac = result

class ASL:
    opcodes = {
        0x0A: lambda p: ASL.base(p, am.ac(p)),     # ASL Accumulator
        0x06: lambda p: ASL.base(p, am.zpg(p)),      # ASL Zero Page
        0x16: lambda p: ASL.base(p, am.zpgx(p)),     # ASL Zero Page,X
        0x0E: lambda p: ASL.base(p, am.abs(p)),      # ASL Absolute
        0x1E: lambda p: ASL.base(p, am.absx(p)),     # ASL Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = (v << 1) & 0xFF  # 8-bit wraparound

        # Set flags
        p.flags.carry = (v & 0x80) != 0
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        # Write result
        if mode == am.modeID.AC:
            p.ac = result
        else:
            p.memory[addr] = result

class Branches:
    opcodes = {
        0x90: lambda p: Branches.base(p, am.rel(p), p.flags.carry, False),      # BCC Relative
        0xB0: lambda p: Branches.base(p, am.rel(p), p.flags.carry, True),       # BCS Relative
        0xF0: lambda p: Branches.base(p, am.rel(p), p.flags.zero, True),        # BEQ Relative
        0x30: lambda p: Branches.base(p, am.rel(p), p.flags.negative, True),    # BMI Relative
        0xD0: lambda p: Branches.base(p, am.rel(p), p.flags.zero, False),       # BNE Relative
        0x10: lambda p: Branches.base(p, am.rel(p), p.flags.negative, False),   # BPL Relative
        0x50: lambda p: Branches.base(p, am.rel(p), p.flags.overflow, False),   # BVC Relative
        0x70: lambda p: Branches.base(p, am.rel(p), p.flags.overflow, True),    # BVS Relative
    }

    @staticmethod
    def base(p, mode_data, flag, condition):
        mode, addr, v = mode_data
        if flag == condition:
            p.pc = addr  # Update PC with relative address

class BIT:
    opcodes = {
        0x24: lambda p: BIT.base(p, am.zpg(p)),      # BIT Zero Page
        0x2C: lambda p: BIT.base(p, am.abs(p)),      # BIT Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.ac & v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (v & 0x80) != 0
        p.flags.overflow = (v & 0x40) != 0
        
class CMP:
    opcodes = {
        0xC9: lambda p: CMP.base(p, am.imm(p)),     # CMP Immediate
        0xC5: lambda p: CMP.base(p, am.zpg(p)),     # CMP Zero Page
        0xD5: lambda p: CMP.base(p, am.zpgx(p)),    # CMP Zero Page,X
        0xCD: lambda p: CMP.base(p, am.abs(p)),     # CMP Absolute
        0xDD: lambda p: CMP.base(p, am.absx(p)),    # CMP Absolute,X
        0xD9: lambda p: CMP.base(p, am.absy(p)),    # CMP Absolute,Y
        0xC1: lambda p: CMP.base(p, am.indx(p)),    # CMP (Indirect,X)
        0xD1: lambda p: CMP.base(p, am.indy(p)),    # CMP (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.ac - v

        # Set flags
        p.flags.carry = (result >= 0)
        p.flags.zero = (result & 0xFF == 0)
        p.flags.negative = (result & 0x80) != 0

class CPX:
    opcodes = {
        0xE0: lambda p: CPX.base(p, am.imm(p)),     # CPX Immediate
        0xE4: lambda p: CPX.base(p, am.zpg(p)),     # CPX Zero Page
        0xEC: lambda p: CPX.base(p, am.abs(p)),     # CPX Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.x - v

        # Set flags
        p.flags.carry = (result >= 0)
        p.flags.zero = (result & 0xFF == 0)
        p.flags.negative = (result & 0x80) != 0

class CPY:
    opcodes = {
        0xC0: lambda p: CPY.base(p, am.imm(p)),     # CPY Immediate
        0xC4: lambda p: CPY.base(p, am.zpg(p)),     # CPY Zero Page
        0xCC: lambda p: CPY.base(p, am.abs(p)),     # CPY Absolute
    }
    
    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.y - v

        # Set flags
        p.flags.carry = (result >= 0)
        p.flags.zero = (result & 0xFF == 0)
        p.flags.negative = (result & 0x80) != 0

class CMP:
    opcodes = {
        0xC9: lambda p: CMP.base(p, am.imm(p)),     # CMP Immediate
        0xC5: lambda p: CMP.base(p, am.zpg(p)),     # CMP Zero Page
        0xD5: lambda p: CMP.base(p, am.zpgx(p)),    # CMP Zero Page,X
        0xCD: lambda p: CMP.base(p, am.abs(p)),     # CMP Absolute
        0xDD: lambda p: CMP.base(p, am.absx(p)),    # CMP Absolute,X
        0xD9: lambda p: CMP.base(p, am.absy(p)),    # CMP Absolute,Y
        0xC1: lambda p: CMP.base(p, am.indx(p)),    # CMP (Indirect,X)
        0xD1: lambda p: CMP.base(p, am.indy(p)),    # CMP (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        diff = (int(p.ac) - int(v)) & 0xFF  # 8-bit unsigned result

        p.flags.carry = p.ac >= v           # Unsigned comparison — valid in NumPy
        p.flags.zero = diff == 0
        p.flags.negative = (diff & 0x80) != 0

class CPX:
    opcodes = {
        0xE0: lambda p: CPX.base(p, am.imm(p)),     # CPX Immediate
        0xE4: lambda p: CPX.base(p, am.zpg(p)),     # CPX Zero Page
        0xEC: lambda p: CPX.base(p, am.abs(p)),     # CPX Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        diff = (int(p.x) - int(v)) & 0xFF

        p.flags.carry = p.x >= v
        p.flags.zero = diff == 0
        p.flags.negative = (diff & 0x80) != 0
        
class CPY:
    opcodes = {
        0xC0: lambda p: CPY.base(p, am.imm(p)),     # CPY Immediate
        0xC4: lambda p: CPY.base(p, am.zpg(p)),     # CPY Zero Page
        0xCC: lambda p: CPY.base(p, am.abs(p)),     # CPY Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        diff = (int(p.y) - int(v)) & 0xFF

        p.flags.carry = p.y >= v
        p.flags.zero = diff == 0
        p.flags.negative = (diff & 0x80) != 0

class DEC:
    opcodes = {
        0xC6: lambda p: DEC.base(p, am.zpg(p)),      # DEC Zero Page
        0xD6: lambda p: DEC.base(p, am.zpgx(p)),     # DEC Zero Page,X
        0xCE: lambda p: DEC.base(p, am.abs(p)),      # DEC Absolute
        0xDE: lambda p: DEC.base(p, am.absx(p)),     # DEC Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = v - 1

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0
        p.memory[addr] = result

class DEX:
    opcodes = {
        0xCA: lambda p: DEX.base(p, am.implied(p)),  # DEX Implied
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.x = p.x - 1

        # Set flags
        p.flags.zero = (p.x == 0)
        p.flags.negative = (p.x & 0x80) != 0

class DEY:
    opcodes = {
        0x88: lambda p: DEY.base(p, am.implied(p)),  # DEY Implied
    }
    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.y = p.y - 1

        # Set flags
        p.flags.zero = (p.y == 0)
        p.flags.negative = (p.y & 0x80) != 0

class EOR:
    opcodes = {
        0x49: lambda p: EOR.base(p, am.imm(p)),     # EOR Immediate
        0x45: lambda p: EOR.base(p, am.zpg(p)),     # EOR Zero Page
        0x55: lambda p: EOR.base(p, am.zpgx(p)),    # EOR Zero Page,X
        0x4D: lambda p: EOR.base(p, am.abs(p)),     # EOR Absolute
        0x5D: lambda p: EOR.base(p, am.absx(p)),    # EOR Absolute,X
        0x59: lambda p: EOR.base(p, am.absy(p)),    # EOR Absolute,Y
        0x41: lambda p: EOR.base(p, am.indx(p)),    # EOR (Indirect,X)
        0x51: lambda p: EOR.base(p, am.indy(p)),    # EOR (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.ac ^ v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        p.ac = result

class INC:
    opcodes = {
        0xE6: lambda p: INC.base(p, am.zpg(p)),      # INC Zero Page
        0xF6: lambda p: INC.base(p, am.zpgx(p)),     # INC Zero Page,X
        0xEE: lambda p: INC.base(p, am.abs(p)),      # INC Absolute
        0xFE: lambda p: INC.base(p, am.absx(p)),     # INC Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = v + 1

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0
        p.memory[addr] = result

class INX:
    opcodes = {
        0xE8: lambda p: INX.base(p, am.implied(p)),  # INX Implied
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.x = p.x + 1

        # Set flags
        p.flags.zero = (p.x == 0)
        p.flags.negative = (p.x & 0x80) != 0
        
class INY:
    opcodes = {
        0xC8: lambda p: INY.base(p, am.implied(p)),  # INY Implied
    }
    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.y = p.y + 1

        # Set flags
        p.flags.zero = (p.y == 0)
        p.flags.negative = (p.y & 0x80) != 0

class JMP:
    opcodes = {
        0x4C: lambda p: JMP.base(p, am.abs(p)),      # JMP Absolute
        0x6C: lambda p: JMP.base(p, am.ind(p)),      # JMP Indirect
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.pc = uint16(addr)  # Set PC to the address from the mode data

class JSR:
    opcodes = {
        0x20: lambda p: JSR.base(p, am.abs(p)),      # JSR Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.stack.push16(p.pc-1)  # Push current PC onto stack
        p.pc = uint16(addr)   # Set PC to the address from the mode data

class LDA:
    opcodes = {
        0xA9: lambda p: LDA.base(p, am.imm(p)),     # LDA Immediate
        0xA5: lambda p: LDA.base(p, am.zpg(p)),     # LDA Zero Page
        0xB5: lambda p: LDA.base(p, am.zpgx(p)),    # LDA Zero Page,X
        0xAD: lambda p: LDA.base(p, am.abs(p)),     # LDA Absolute
        0xBD: lambda p: LDA.base(p, am.absx(p)),    # LDA Absolute,X
        0xB9: lambda p: LDA.base(p, am.absy(p)),    # LDA Absolute,Y
        0xA1: lambda p: LDA.base(p, am.indx(p)),    # LDA (Indirect,X)
        0xB1: lambda p: LDA.base(p, am.indy(p)),    # LDA (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        p.ac = result

class LDX:
    opcodes = {
        0xA2: lambda p: LDX.base(p, am.imm(p)),     # LDX Immediate
        0xA6: lambda p: LDX.base(p, am.zpg(p)),     # LDX Zero Page
        0xB6: lambda p: LDX.base(p, am.zpgy(p)),    # LDX Zero Page,Y
        0xAE: lambda p: LDX.base(p, am.abs(p)),     # LDX Absolute
        0xBE: lambda p: LDX.base(p, am.absy(p)),    # LDX Absolute,Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        p.x = result

class LDY:
    opcodes = {
        0xA0: lambda p: LDY.base(p, am.imm(p)),     # LDY Immediate
        0xA4: lambda p: LDY.base(p, am.zpg(p)),     # LDY Zero Page
        0xB4: lambda p: LDY.base(p, am.zpgx(p)),    # LDY Zero Page,X
        0xAC: lambda p: LDY.base(p, am.abs(p)),     # LDY Absolute
        0xBC: lambda p: LDY.base(p, am.absx(p)),    # LDY Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        p.y = result

class LSR:
    opcodes = {
        0x4A: lambda p: LSR.base(p, am.ac(p)),     # LSR Accumulator
        0x46: lambda p: LSR.base(p, am.zpg(p)),      # LSR Zero Page
        0x56: lambda p: LSR.base(p, am.zpgx(p)),     # LSR Zero Page,X
        0x4E: lambda p: LSR.base(p, am.abs(p)),      # LSR Absolute
        0x5E: lambda p: LSR.base(p, am.absx(p)),     # LSR Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = v >> 1

        # Set flags
        p.flags.carry = (v & 0x01) != 0
        p.flags.zero = (result == 0)
        p.flags.negative = False

        if mode == am.modeID.AC:
            p.ac = result
        else:
            p.memory[addr] = result         

class NOP:
    opcodes = {
        0xEA: lambda p: NOP.base(p, am.implied(p)),  # NOP Implied
    }

    @staticmethod
    def base(p, mode_data):
        # NOP does nothing, just return
        pass

class ORA:
    opcodes = {
        0x09: lambda p: ORA.base(p, am.imm(p)),     # ORA Immediate
        0x05: lambda p: ORA.base(p, am.zpg(p)),     # ORA Zero Page
        0x15: lambda p: ORA.base(p, am.zpgx(p)),    # ORA Zero Page,X
        0x0D: lambda p: ORA.base(p, am.abs(p)),     # ORA Absolute
        0x1D: lambda p: ORA.base(p, am.absx(p)),    # ORA Absolute,X
        0x19: lambda p: ORA.base(p, am.absy(p)),    # ORA Absolute,Y
        0x01: lambda p: ORA.base(p, am.indx(p)),    # ORA (Indirect,X)
        0x11: lambda p: ORA.base(p, am.indy(p)),    # ORA (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = p.ac | v

        # Set flags
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        p.ac = result

class PHA:
    opcodes = {
        0x48: lambda p: PHA.base(p, am.implied(p)),  # PHA Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.stack.push(p.ac)  # Push accumulator onto stack

class PHP:
    opcodes = {
        0x08: lambda p: PHP.base(p, am.implied(p)),  # PHP Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.stack.push(p.sr | 0b00110000)  # Push status register onto stack with unused bits set

class PLA:
    opcodes = {
        0x68: lambda p: PLA.base(p, am.implied(p)),  # PLA Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.ac = p.stack.pull()  # Pop value from stack into accumulator

        # Set flags
        p.flags.zero = (p.ac == 0)
        p.flags.negative = (p.ac & 0x80) != 0

class PLP:
    opcodes = {
        0x28: lambda p: PLP.base(p, am.implied(p)),  # PLP Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.sr = p.stack.pull()  # Pop value from stack into status register

class ROL:
    opcodes = {
        0x2A: lambda p: ROL.base(p, am.ac(p)),     # ROL Accumulator
        0x26: lambda p: ROL.base(p, am.zpg(p)),      # ROL Zero Page
        0x36: lambda p: ROL.base(p, am.zpgx(p)),     # ROL Zero Page,X
        0x2E: lambda p: ROL.base(p, am.abs(p)),      # ROL Absolute
        0x3E: lambda p: ROL.base(p, am.absx(p)),     # ROL Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = uint8(v << 1) | (1 if p.flags.carry else 0)

        # Set flags
        p.flags.carry = (v & 0x80) != 0
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        if mode == am.modeID.AC:
            p.ac = result
        else:
            p.memory[addr] = result   

class ROR:
    opcodes = {
        0x6A: lambda p: ROR.base(p, am.ac(p)),     # ROR Accumulator
        0x66: lambda p: ROR.base(p, am.zpg(p)),      # ROR Zero Page
        0x76: lambda p: ROR.base(p, am.zpgx(p)),     # ROR Zero Page,X
        0x6E: lambda p: ROR.base(p, am.abs(p)),      # ROR Absolute
        0x7E: lambda p: ROR.base(p, am.absx(p)),     # ROR Absolute,X
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        result = uint8(v >> 1) | (0x80 if p.flags.carry else 0)

        # Set flags
        p.flags.carry = (v & 0x01) != 0
        p.flags.zero = (result == 0)
        p.flags.negative = (result & 0x80) != 0

        if mode == am.modeID.AC:
            p.ac = result
        else:
            p.memory[addr] = result   

class RTI:
    opcodes = {
        0x40: lambda p: RTI.base(p, am.implied(p)),  # RTI Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.sr = p.stack.pull()  # Pop status register from stack
        p.pc = p.stack.pull16()  # Pop program counter from stack

class RTS:
    opcodes = {
        0x60: lambda p: RTS.base(p, am.implied(p)),  # RTS Implied
    }
    
    @staticmethod
    def base(p, mode_data):
        p.pc = p.stack.pull16() + 1  # Pop program counter from stack

class SBC:
    opcodes = {
        0xE9: lambda p: SBC.base(p, am.imm(p)),     # SBC Immediate
        0xE5: lambda p: SBC.base(p, am.zpg(p)),     # SBC Zero Page
        0xF5: lambda p: SBC.base(p, am.zpgx(p)),    # SBC Zero Page,X
        0xED: lambda p: SBC.base(p, am.abs(p)),     # SBC Absolute
        0xFD: lambda p: SBC.base(p, am.absx(p)),    # SBC Absolute,X
        0xF9: lambda p: SBC.base(p, am.absy(p)),    # SBC Absolute,Y
        0xE1: lambda p: SBC.base(p, am.indx(p)),    # SBC (Indirect,X)
        0xF1: lambda p: SBC.base(p, am.indy(p)),    # SBC (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        carry_in = 0 if p.flags.carry else 1

        a = int(p.ac)
        b = int(v)

        result = a - b - carry_in
        result8 = result & 0xFF  # 8-bit wraparound

        # Flags
        p.flags.carry = result >= 0
        p.flags.zero = (result8 == 0)
        p.flags.negative = (result8 & 0x80) != 0
        p.flags.overflow = ((a ^ b) & (a ^ result8) & 0x80) != 0

        p.ac = uint8(result8)

class setFlags:
    opcodes = {
        0x38: lambda p: setFlags.base(p,0),     # SEC Implied
        0xF8: lambda p: setFlags.base(p,1),   # SED Implied
        0x78: lambda p: setFlags.base(p,2), # SEI Implied
    }

    @staticmethod
    def base(p, flag):
        if flag == 0: p.flags.carry = True
        elif flag == 1: p.flags.decimal = True
        elif flag == 2: p.flags.interrupt = True

class STA:
    opcodes = {
        0x85: lambda p: STA.base(p, am.zpg(p)),     # STA Zero Page
        0x95: lambda p: STA.base(p, am.zpgx(p)),    # STA Zero Page,X
        0x8D: lambda p: STA.base(p, am.abs(p)),     # STA Absolute
        0x9D: lambda p: STA.base(p, am.absx(p)),    # STA Absolute,X
        0x99: lambda p: STA.base(p, am.absy(p)),    # STA Absolute,Y
        0x81: lambda p: STA.base(p, am.indx(p)),    # STA (Indirect,X)
        0x91: lambda p: STA.base(p, am.indy(p)),    # STA (Indirect),Y
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.memory[addr] = p.ac

class STX:
    opcodes = {
        0x86: lambda p: STX.base(p, am.zpg(p)),     # STX Zero Page
        0x96: lambda p: STX.base(p, am.zpgy(p)),    # STX Zero Page,Y
        0x8E: lambda p: STX.base(p, am.abs(p)),     # STX Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.memory[addr] = p.x

class STY:
    opcodes = {
        0x84: lambda p: STY.base(p, am.zpg(p)),     # STY Zero Page
        0x94: lambda p: STY.base(p, am.zpgx(p)),    # STY Zero Page,X
        0x8C: lambda p: STY.base(p, am.abs(p)),     # STY Absolute
    }

    @staticmethod
    def base(p, mode_data):
        mode, addr, v = mode_data
        p.memory[addr] = p.y

class TAX:
    opcodes = {
        0xAA: lambda p: TAX.base(p, am.implied(p)),  # TAX Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.x = p.ac  # Transfer accumulator to X register

        # Set flags
        p.flags.zero = (p.x == 0)
        p.flags.negative = (p.x & 0x80) != 0

class TAY:
    opcodes = {
        0xA8: lambda p: TAY.base(p, am.implied(p)),  # TAY Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.y = p.ac  # Transfer accumulator to Y register

        # Set flags
        p.flags.zero = (p.y == 0)
        p.flags.negative = (p.y & 0x80) != 0

class TSX:
    opcodes = {
        0xBA: lambda p: TSX.base(p, am.implied(p)),  # TSX Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.x = p.sp  # Transfer stack pointer to X register

        # Set flags
        p.flags.zero = (p.x == 0)
        p.flags.negative = (p.x & 0x80) != 0

class TXA:
    opcodes = {
        0x8A: lambda p: TXA.base(p, am.implied(p)),  # TXA Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.ac = p.x  # Transfer X register to accumulator

        # Set flags
        p.flags.zero = (p.ac == 0)
        p.flags.negative = (p.ac & 0x80) != 0

class TXS:
    opcodes = {
        0x9A: lambda p: TXS.base(p, am.implied(p)),  # TXS Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.sp = p.x  # Transfer X register to stack pointer

class TYA:
    opcodes = {
        0x98: lambda p: TYA.base(p, am.implied(p)),  # TYA Implied
    }

    @staticmethod
    def base(p, mode_data):
        p.ac = p.y  # Transfer Y register to accumulator

        # Set flags
        p.flags.zero = (p.ac == 0)
        p.flags.negative = (p.ac & 0x80) != 0

#base 6502 operations
baseOps = [ADC, AND, ASL, Branches, BIT, BRK, clearFlags, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI, RTS, SBC, setFlags, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA]
# Additional operations for 65C02
cOps = []
