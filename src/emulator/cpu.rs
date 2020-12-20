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

            // Increment B, D, H
            0x04 => {self.regs.b = self.inc(self.regs.b)} // INC B
            0x14 => {self.regs.d = self.inc(self.regs.d)} // INC D
            0x24 => {self.regs.h = self.inc(self.regs.h)} // INC H

            // Decrement B, D, H
            0x05 => {self.regs.b = self.dec(self.regs.b)} // DEC B
            0x15 => {self.regs.d = self.dec(self.regs.d)} // DEC D
            0x25 => {self.regs.h = self.dec(self.regs.h)} // DEC H

            // Decrement C, E, L, A
            0x0B => {self.regs.c = self.inc(self.regs.c)} // INC C
            0x1B => {self.regs.e = self.inc(self.regs.e)} // INC E
            0x2B => {self.regs.l = self.inc(self.regs.l)} // INC L
            0x3B => {self.regs.a = self.inc(self.regs.a)} // INC A

            // Increment C, E, L, A
            0x0C => {self.regs.c = self.dec(self.regs.c)} // DEC C
            0x1C => {self.regs.e = self.dec(self.regs.e)} // DEC E
            0x2C => {self.regs.l = self.dec(self.regs.l)} // DEC L
            0x3C => {self.regs.a = self.dec(self.regs.a)} // DEC A

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
            0x80 => { self.regs.a = self.add(self.regs.b, false); } // ADD B
            0x81 => { self.regs.a = self.add(self.regs.c, false); } // ADD C
            0x82 => { self.regs.a = self.add(self.regs.d, false); } // ADD D
            0x83 => { self.regs.a = self.add(self.regs.e, false); } // ADD E
            0x84 => { self.regs.a = self.add(self.regs.h, false); } // ADD H
            0x85 => { self.regs.a = self.add(self.regs.l, false); } // ADD L
            // TODO HL
            0x87 => { self.regs.a = self.add(self.regs.a, false); } // ADD A

            // Add with carry instruction
            0x88 => { self.regs.a = self.add(self.regs.b, true); } // ADC B
            0x89 => { self.regs.a = self.add(self.regs.c, true); } // ADC C
            0x8A => { self.regs.a = self.add(self.regs.d, true); } // ADC D
            0x8B => { self.regs.a = self.add(self.regs.e, true); } // ADC E
            0x8C => { self.regs.a = self.add(self.regs.h, true); } // ADC H
            0x8D => { self.regs.a = self.add(self.regs.l, true); } // ADC L
            // TODO HL
            0x8F => { self.regs.a = self.add(self.regs.a, false); } // ADC A

            // Sub instruction
            0x90 => { self.regs.a = self.sub(self.regs.b, false); } // SUB B
            0x91 => { self.regs.a = self.sub(self.regs.c, false); } // SUB C
            0x92 => { self.regs.a = self.sub(self.regs.d, false); } // SUB D
            0x93 => { self.regs.a = self.sub(self.regs.e, false); } // SUB E
            0x94 => { self.regs.a = self.sub(self.regs.h, false); } // SUB H
            0x95 => { self.regs.a = self.sub(self.regs.l, false); } // SUB L
            // TODO HL
            0x97 => { self.regs.a = self.sub(self.regs.a, false); } // SUB A

            // Sub with carry instruction
            0x98 => { self.regs.a = self.sub(self.regs.b, true); } // SBC B
            0x99 => { self.regs.a = self.sub(self.regs.c, true); } // SBC C
            0x9A => { self.regs.a = self.sub(self.regs.d, true); } // SBC D
            0x9B => { self.regs.a = self.sub(self.regs.e, true); } // SBC E
            0x9C => { self.regs.a = self.sub(self.regs.h, true); } // SBC H
            0x9D => { self.regs.a = self.sub(self.regs.l, true); } // SBC L
            // TODO HL
            0x9F => { self.regs.a = self.sub(self.regs.a, true); } // SUB A

            _ => { /* TODO: support more instructions */ }
          }
    }

    // ADD Instruction
    fn add(&mut self, value: u8, use_carry: bool) -> u8
    {
        let carry = if use_carry && self.regs.get_carry_flag() { 1 } else { 0 };
        let new_value = self.regs.a.wrapping_add(value).wrapping_add(carry);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag((self.regs.a as u16) + (value as u16) + (carry as u16) > 0xFF);
        self.regs.set_halfcarry_flag((self.regs.a & 0xF) + (value & 0xF) + carry > 0xF);
        new_value
    }

    // SUB Instruction
    fn sub(&mut self, value: u8, use_carry: bool) -> u8
    {
        let carry = if use_carry && self.regs.get_carry_flag() { 1 } else { 0 };
        let new_value = self.regs.a.wrapping_sub(value).wrapping_sub(carry);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(true);
        self.regs.set_carry_flag((self.regs.a as u16) < (value as u16) + (carry as u16));
        self.regs.set_halfcarry_flag((self.regs.a & 0x0F) < (value & 0x0F) + carry);
        new_value
    }

    // INC Instruction
    fn inc(&mut self, value: u8) -> u8
    {
        let new_value = value.wrapping_add(1);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_halfcarry_flag((value & 0xF) + 1 > 0xF);
        self.regs.set_subtract_flag(false);
        new_value
    }

    // DEC Instruction
    fn dec(&mut self, value: u8) -> u8
    {
        let new_value = value.wrapping_sub(1);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_halfcarry_flag((value & 0x0F) == 0);
        self.regs.set_subtract_flag(false);
        new_value
    }
}