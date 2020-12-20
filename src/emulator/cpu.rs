mod registers;

pub struct CPU
{
    pub regs : registers::Registers
}

impl CPU {
    pub fn new() -> CPU
    {
        CPU { regs : registers::Registers::new()}
    }
    
    // Execute an instruction/opcode
    // Good opcode table: https://meganesulli.com/generate-gb-opcodes/
    pub fn execute(&mut self, opcode : u8)
    {

        match opcode {
            0x0 => {  } // NOP (No op)

            // Load into B
            0x40 => { } // LD B B (NOP)
            0x41 => { self.regs.b = self.regs.c} // LD B C
            0x42 => { self.regs.b = self.regs.d} // LD B D 
            0x43 => { self.regs.b = self.regs.e} // LD B E
            0x44 => { self.regs.b = self.regs.h} // LD B H
            0x45 => { self.regs.b = self.regs.l} // LD B L
            0x46 => { } // TODO 
            0x47 => { self.regs.b = self.regs.a} // LD B L

            // Load into C
            0x48 => { self.regs.c = self.regs.b} // LD C B
            0x49 => { } // LD C C (NOP)
            0x4A => { self.regs.c = self.regs.d} // LD C D
            0x4B => { self.regs.c = self.regs.e} // LD C E
            0x4C => { self.regs.c = self.regs.h} // LD C H
            0x4D => { self.regs.c = self.regs.l} // LD C L
            0x4E => { } // TODO
            0x4F => { self.regs.c = self.regs.a} // LD C A

            // Load into D
            0x50 => { self.regs.d = self.regs.b} // LD D B
            0x51 => { self.regs.d = self.regs.c} // LD D C
            0x52 => { } // LD D D (NOOP)
            0x53 => { self.regs.d = self.regs.e} // LD D E
            0x54 => { self.regs.d = self.regs.h} // LD D H
            0x55 => { self.regs.d = self.regs.l} // LD D L
            0x56 => { } // TODO
            0x57 => { self.regs.d = self.regs.a} // LD D A

            // Load into E
            0x58 => { self.regs.e = self.regs.b} // LD E B
            0x59 => { self.regs.e = self.regs.c} // LD E C 
            0x5A => { self.regs.e = self.regs.d} // LD E D
            0x5B => { } // LD E E (NOP)
            0x5C => { self.regs.e = self.regs.h} // LD E H
            0x5D => { self.regs.e = self.regs.l} // LD E L
            0x5E => { } // TODO, LD HL
            0x5F => { self.regs.e = self.regs.a} // LD E A

            // Load into H
            0x60 => { self.regs.h = self.regs.b} // LD H B
            0x61 => { self.regs.h = self.regs.c} // LD H C
            0x62 => { self.regs.h = self.regs.d} // LD H D 
            0x63 => { self.regs.h = self.regs.e} // LD H E
            0x64 => { } // LD H H (NOP)
            0x65 => { self.regs.h = self.regs.l} // LD H L
            0x66 => { } // TODO, LD HL
            0x67 => { self.regs.h = self.regs.a} // LD H A

            // Load into L
            0x68 => { self.regs.l = self.regs.b} // LD L B
            0x69 => { self.regs.l = self.regs.c} // LD L C 
            0x6A => { self.regs.l = self.regs.d} // LD L D
            0x6B => { self.regs.l = self.regs.e} // LD L E
            0x6C => { self.regs.l = self.regs.h} // LD L H
            0x6D => { } // LD L L (NOP)
            0x6E => { } // TODO, LD HL
            0x6F => { self.regs.l = self.regs.a} // LD L A

            // Load into A
            0x78 => { self.regs.a = self.regs.b} // LD A B
            0x79 => { self.regs.a = self.regs.c} // LD A C 
            0x7A => { self.regs.a = self.regs.d} // LD A D
            0x7B => { self.regs.a = self.regs.e} // LD A E
            0x7C => { self.regs.a = self.regs.h} // LD A H
            0x7D => { self.regs.a = self.regs.l} // LD A L
            0x7E => { } // TODO, LD HL
            0x7F => { self.regs.a = self.regs.a} // LD A A (NOOP)

            // Add instruction
            0x80 => { self.regs.a = self.add(self.regs.b); } // ADD B
            0x81 => { self.regs.a = self.add(self.regs.c); } // ADD C
            0x82 => { self.regs.a = self.add(self.regs.d); } // ADD D
            0x83 => { self.regs.a = self.add(self.regs.e); } // ADD E
            0x84 => { self.regs.a = self.add(self.regs.h); } // ADD H
            0x85 => { self.regs.a = self.add(self.regs.l); } // ADD L
            0x87 => { self.regs.a = self.add(self.regs.a); } // ADD A

            _ => { /* TODO: support more instructions */ }
          }
    }

    // ADD Instruction
    fn add(&mut self, value: u8) -> u8
    {
        let (new_value, did_overflow) = self.regs.a.overflowing_add(value);
        self.set_flags(new_value, did_overflow, false, CPU::calculate_half_carry(self.regs.a, value));
        new_value
    }

    // Various helpers
    fn calculate_half_carry(register : u8 , result : u8) -> bool
    {
        (register & 0xF) + (result & 0xF) > 0xF
    }

    fn set_flags(&mut self, new_value : u8, carry : bool, subtract: bool, half_carry : bool)
    {
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(subtract);
        self.regs.set_carry_flag(carry);
        self.regs.set_halfcarry_flag(half_carry);

    }
}