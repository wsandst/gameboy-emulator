mod registers;
use crate::emulator::memory;

pub struct CPU
{
    pub regs : registers::Registers,
}

impl CPU {
    pub fn new() -> CPU
    {
        CPU { regs : registers::Registers::new()}
    }
    
    // Execute an instruction/opcode
    // Good opcode table: https://meganesulli.com/generate-gb-opcodes/
    pub fn execute(&mut self, opcode : u8, memory: &mut memory::Memory)
    {
        match opcode {
            0x0 => {  } // NOP (No op)

            // LD d16 BC,DE,HL
            0x01 => { let v = self.fetchword(memory); self.regs.set_bc(v)} // LD BC d16
            0x11 => { let v = self.fetchword(memory); self.regs.set_de(v)} // LD DE d16
            0x21 => { let v = self.fetchword(memory); self.regs.set_hl(v)} // LD HL d16
            0x31 => { self.regs.sp = self.fetchword(memory)} // LD SP d16

            // LD (Wide) A
            0x02 => { memory.write_byte(self.regs.get_bc(), self.regs.a)} // LD (BC) A
            0x12 => { memory.write_byte(self.regs.get_de(), self.regs.a)} // LD (DE) A
            0x22 => { memory.write_byte(self.regs.get_hl(), self.regs.a); 
                let v = self.regs.get_hl().wrapping_add(1); self.regs.set_hl(v)} // LD (HL+) A
            0x32 => { memory.write_byte(self.regs.get_hl(), self.regs.a);
                let v = self.regs.get_hl().wrapping_sub(1); self.regs.set_hl(v)} // LD (HL-) A

            // LD A (Wide)
            0x0A => { self.regs.a = memory.read_byte(self.regs.get_bc())} // LD A (BC)
            0x1A => { self.regs.a = memory.read_byte(self.regs.get_de())} // LD A (DE)
            0x2A => { self.regs.a = memory.read_byte(self.regs.get_hl());
                let v = self.regs.get_hl().wrapping_add(1); self.regs.set_hl(v)} // LD A (HL+)
            0x3A => { self.regs.a = memory.read_byte(self.regs.get_hl());
                let v = self.regs.get_hl().wrapping_sub(1); self.regs.set_hl(v)} // LD A (HL-)

            // LD B,D,H,(HL) d8
            0x06 => { self.regs.b = self.fetchbyte(memory)} // LD d8 B
            0x16 => { self.regs.d = self.fetchbyte(memory)} // LD d8 D
            0x26 => { self.regs.h = self.fetchbyte(memory)} // LD d8 H
            0x36 => { memory.write_byte(self.regs.get_hl(), self.fetchbyte(memory))} // LD d8 (HL)

            // LD C, E, L, A  d8
            0x0E => { self.regs.c = self.fetchbyte(memory)} // LD d8 C
            0x1E => { self.regs.e = self.fetchbyte(memory)} // LD d8 E
            0x2E => { self.regs.l = self.fetchbyte(memory)} // LD d8 L
            0x3E => { self.regs.a = self.fetchbyte(memory)} // LD d8 A

            // Increment (Wide)
            0x03 => { let v = self.regs.get_bc().wrapping_add(1); self.regs.set_bc(v)} // INC BC
            0x13 => { let v = self.regs.get_de().wrapping_add(1); self.regs.set_de(v)} // INC DE
            0x23 => { let v = self.regs.get_hl().wrapping_add(1); self.regs.set_hl(v)} // INC HL
            0x33 => { self.regs.sp = self.regs.sp.wrapping_add(1);} // INC SP

            // Decrement (Wide)
            0x0B => { let v = self.regs.get_bc().wrapping_sub(1); self.regs.set_bc(v)} // DEC BC
            0x1B => { let v = self.regs.get_de().wrapping_sub(1); self.regs.set_de(v)} // DEC DE
            0x2B => { let v = self.regs.get_hl().wrapping_sub(1); self.regs.set_hl(v)} // DEC HL
            0x3B => { self.regs.sp = self.regs.sp.wrapping_sub(1);} // DEC SP

            // Add (Wide)
            0x09 => { let v = self.regs.get_hl().wrapping_add(self.regs.get_bc()); self.regs.set_hl(v)} // ADD HL BC
            0x19 => { let v = self.regs.get_hl().wrapping_add(self.regs.get_de()); self.regs.set_hl(v)} // ADD HL DE
            0x29 => { let v = self.regs.get_hl().wrapping_add(self.regs.get_hl()); self.regs.set_hl(v)} // ADD HL HL
            0x39 => { let v = self.regs.get_hl().wrapping_add(self.regs.sp); self.regs.set_hl(v)} // ADD HL SP

            // Increment B, D, H
            0x04 => {self.regs.b = self.inc(self.regs.b)} // INC B
            0x14 => {self.regs.d = self.inc(self.regs.d)} // INC D
            0x24 => {self.regs.h = self.inc(self.regs.h)} // INC H
            0x34 => { let addr = self.fetchword(memory); 
                memory.write_byte(addr, memory.read_byte(addr).wrapping_add(1))} // INC (HL)

            // Decrement B, D, H
            0x05 => { self.regs.b = self.dec(self.regs.b)} // DEC B
            0x15 => { self.regs.d = self.dec(self.regs.d)} // DEC D
            0x25 => { self.regs.h = self.dec(self.regs.h)} // DEC H
            0x35 => { let addr = self.fetchword(memory); 
                memory.write_byte(addr, memory.read_byte(addr).wrapping_sub(1))} // DEC (HL)

            // Set carry flag CF
            0x37 => {self.regs.set_carry_flag(true)} // SCF

            // Store stack pointer at address
            0x08 => {memory.write_word(self.fetchword(memory), self.regs.sp)} // LD (a16), SP

            // Increment C, E, L, A
            0x0C => { self.regs.c = self.dec(self.regs.c)} // DEC C
            0x1C => { self.regs.e = self.dec(self.regs.e)} // DEC E
            0x2C => { self.regs.l = self.dec(self.regs.l)} // DEC L
            0x3C => { self.regs.a = self.dec(self.regs.a)} // DEC A

            // Decrement C, E, L, A
            0x0D => { self.regs.c = self.inc(self.regs.c)} // INC C
            0x1D => { self.regs.e = self.inc(self.regs.e)} // INC E
            0x2D => { self.regs.l = self.inc(self.regs.l)} // INC L
            0x3D => { self.regs.a = self.inc(self.regs.a)} // INC A

            // Complement A
            0x3E => { self.regs.a = !self.regs.a} // CPL

            // Complement carry flag
            0x3F => { let c = self.regs.get_carry_flag(); self.regs.set_carry_flag(!c)} // CCF

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
            0x7E => { } // TODO LD HL
            0x7F => { self.regs.a = self.regs.a} // LD A A (NOOP)

            // Add instruction
            0x80 => { self.regs.a = self.add(self.regs.b, false); } // ADD B
            0x81 => { self.regs.a = self.add(self.regs.c, false); } // ADD C
            0x82 => { self.regs.a = self.add(self.regs.d, false); } // ADD D
            0x83 => { self.regs.a = self.add(self.regs.e, false); } // ADD E
            0x84 => { self.regs.a = self.add(self.regs.h, false); } // ADD H
            0x85 => { self.regs.a = self.add(self.regs.l, false); } // ADD L
            // TODO ADD HL
            0x87 => { self.regs.a = self.add(self.regs.a, false); } // ADD A

            // Add with carry instruction
            0x88 => { self.regs.a = self.add(self.regs.b, true); } // ADC B
            0x89 => { self.regs.a = self.add(self.regs.c, true); } // ADC C
            0x8A => { self.regs.a = self.add(self.regs.d, true); } // ADC D
            0x8B => { self.regs.a = self.add(self.regs.e, true); } // ADC E
            0x8C => { self.regs.a = self.add(self.regs.h, true); } // ADC H
            0x8D => { self.regs.a = self.add(self.regs.l, true); } // ADC L
            // TODO ADC HL
            0x8F => { self.regs.a = self.add(self.regs.a, true); } // ADC A

            // Sub instruction
            0x90 => { self.regs.a = self.sub(self.regs.b, false); } // SUB B
            0x91 => { self.regs.a = self.sub(self.regs.c, false); } // SUB C
            0x92 => { self.regs.a = self.sub(self.regs.d, false); } // SUB D
            0x93 => { self.regs.a = self.sub(self.regs.e, false); } // SUB E
            0x94 => { self.regs.a = self.sub(self.regs.h, false); } // SUB H
            0x95 => { self.regs.a = self.sub(self.regs.l, false); } // SUB L
            // TODO SUB HL
            0x97 => { self.regs.a = self.sub(self.regs.a, false); } // SUB A

            // Sub with carry instruction
            0x98 => { self.regs.a = self.sub(self.regs.b, true); } // SBC B
            0x99 => { self.regs.a = self.sub(self.regs.c, true); } // SBC C
            0x9A => { self.regs.a = self.sub(self.regs.d, true); } // SBC D
            0x9B => { self.regs.a = self.sub(self.regs.e, true); } // SBC E
            0x9C => { self.regs.a = self.sub(self.regs.h, true); } // SBC H
            0x9D => { self.regs.a = self.sub(self.regs.l, true); } // SBC L
            // TODO SBC HL
            0x9F => { self.regs.a = self.sub(self.regs.a, true); } // SBC A

            // And instruction
            0xA0 => { self.regs.a = self.and(self.regs.b); } // AND B
            0xA1 => { self.regs.a = self.and(self.regs.c); } // AND C
            0xA2 => { self.regs.a = self.and(self.regs.d); } // AND D
            0xA3 => { self.regs.a = self.and(self.regs.e); } // AND E
            0xA4 => { self.regs.a = self.and(self.regs.h); } // AND H
            0xA5 => { self.regs.a = self.and(self.regs.l); } // AND L
            // TODO AND HL
            0xA7 => { self.regs.a = self.and(self.regs.a); } // AND A

            // Xor  instruction
            0xA8 => { self.regs.a = self.xor(self.regs.b); } // XOR B
            0xA9 => { self.regs.a = self.xor(self.regs.c); } // XOR C
            0xAA => { self.regs.a = self.xor(self.regs.d); } // XOR D
            0xAB => { self.regs.a = self.xor(self.regs.e); } // XOR E
            0xAC => { self.regs.a = self.xor(self.regs.h); } // XOR H
            0xAD => { self.regs.a = self.xor(self.regs.l); } // XOR L
            // TODO XOR HL
            0xAF => { self.regs.a = self.xor(self.regs.a); } // XOR A

            // Or instruction
            0xB0 => { self.regs.a = self.or(self.regs.b); } // OR B
            0xB1 => { self.regs.a = self.or(self.regs.c); } // OR C
            0xB2 => { self.regs.a = self.or(self.regs.d); } // OR D
            0xB3 => { self.regs.a = self.or(self.regs.e); } // OR E
            0xB4 => { self.regs.a = self.or(self.regs.h); } // OR H
            0xB5 => { self.regs.a = self.or(self.regs.l); } // OR L
            // TODO OR HL
            0xB7 => { self.regs.a = self.or(self.regs.a); } // OR A

            // CP instruction (set zero flag if the registers are equal)
            0xB8 => { self.regs.set_zero_flag(self.regs.a == self.regs.b) } // CP B
            0xB9 => { self.regs.set_zero_flag(self.regs.a == self.regs.c) } // CP C
            0xBA => { self.regs.set_zero_flag(self.regs.a == self.regs.d) } // CP D
            0xBB => { self.regs.set_zero_flag(self.regs.a == self.regs.e) } // CP E
            0xBC => { self.regs.set_zero_flag(self.regs.a == self.regs.h) } // CP H
            0xBD => { self.regs.set_zero_flag(self.regs.a == self.regs.l) } // CP L
            // TODO CP HL
            0xBF => { self.regs.set_zero_flag(true); } // CP A, A == A

            // LD (8 bit, high ram)
            0xE0 => { memory.write_byte(0xFF00 + self.fetchbyte(memory) as u16, self.regs.a) } // LD (a8) A
            0xF0 => { self.regs.a = memory.read_byte(0xFF00 + self.fetchbyte(memory) as u16) } // LD A (a8)

            // LD (C, high ram)
            0xE3 => { memory.write_byte(0xFF00 + self.regs.c as u16, self.regs.a) } // LD (C) A
            0xF3 => { self.regs.a = memory.read_byte(0xFF00 + self.regs.c as u16) } // LD A (C)            

            // Relative jumps
            0x20 => { if !self.regs.get_zero_flag() { // JR NZ s8 
                self.jump_relative(memory); 
            } } 

            0x30 => { if !self.regs.get_carry_flag() { // JR NC s8
                self.jump_relative(memory); 
            } } 

            0x18 => { self.jump_relative(memory)} // JR s8

            0x28 => { if self.regs.get_zero_flag() { // JR Z s8
                self.jump_relative(memory); } } 

            0x38 => { if self.regs.get_carry_flag() { // JR C s8
                self.jump_relative(memory); } }

            // Absolute jumps
            0xC2 => {if !self.regs.get_zero_flag() { // JP NZ a16
                self.jump(memory);
            }}

            0xD2 => {if !self.regs.get_carry_flag() { // JP NC a16
                self.jump(memory);
            }}

            0xC3 => { self.jump(memory); } // JP a16

            0xCA => {if self.regs.get_zero_flag() { // JP Z a16
                self.jump(memory);
            }}

            0xDA => {if self.regs.get_carry_flag() { // JP C a16
                self.jump(memory);
            }}

            0xE9 => {self.regs.pc = self.regs.get_hl()}

            other => panic!("Instruction {:2X} is not implemented", other)
          }
    }

    fn fetchbyte(&mut self, memory: &memory::Memory) -> u8 
    {
        let byte = memory.read_byte(self.regs.pc);
        self.regs.pc += 1;
        return byte;
    }

    fn fetchword(&mut self, memory: &memory::Memory) -> u16
    {
        let byte = memory.read_word(self.regs.pc);
        self.regs.pc += 2;
        return byte;
    }

    // JR Instruction
    fn jump_relative(&mut self, memory: &memory::Memory) {
        let offset = (self.fetchbyte(memory) as i8) as i32;
        self.regs.pc = (self.regs.pc as i32 + offset) as u16;
    }

    // JP Instruction
    fn jump(&mut self, memory: &memory::Memory) {
        self.regs.pc = self.fetchword(memory);
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

    // AND Instruction
    fn and(&mut self, value: u8) -> u8
    {
        let new_value = self.regs.a & value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(true);
        new_value
    }

    // XOR Instruction
    fn xor(&mut self, value: u8) -> u8
    {
        let new_value = self.regs.a ^ value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(false);
        new_value
    }

    // OR Instruction
    fn or(&mut self, value: u8) -> u8
    {
        let new_value = self.regs.a | value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(false);
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
        self.regs.set_subtract_flag(true);
        new_value
    }
}