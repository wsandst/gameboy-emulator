mod registers;
use super::memory;

/// Represents the 8-bit CPU of a Gameboy/Gameboy Color.
/// 
/// The Gameboy CPU is a SharpLR35902, which is a 8080/Z80 derivative.  
/// Todo:
/// Interupts
/// Track CPU cycles 
pub struct CPU
{
    pub regs : registers::Registers,
}

impl CPU {
    pub fn new() -> CPU
    {
        CPU { regs : registers::Registers::new()}
    }
    
    /// Execute a CPU instruction/opcode.
    /// Currently implements most of the ~512 instructions. 
    /// Good opcode table can be found at: https://meganesulli.com/generate-gb-opcodes/
    /// After every case in the match, the instruction mnemonic is stated.
    /// (X) refers to the value at the memory address X
    /// d8/d16 means the next immediate byte/word from the pc
    /// s8 means next immediate signed byte from the pc
    
    pub fn cycle(&mut self, memory: &mut memory::Memory) {
        let opcode = self.fetchbyte(memory);
        self.execute(opcode, memory);
    }

    pub fn execute(&mut self, opcode : u8, memory: &mut memory::Memory)
    {
        // There are ~256 regular instructions, which are covered by a large match statement.
        match opcode {
            0x0 => {  } // NOP (No op)
            0x10 => {  } // HALT
            0xCB => { let wide_op = self.fetchbyte(memory); self.execute_cb(wide_op, memory);} // Wide instructions prefix

            // Interrupt
            0xFB => { memory.interrupt_flag = false as u8 } // DE, enable interrupts
            0xF3 => { memory.interrupt_flag = false as u8; } // DI, prohibit interrupts 

            // LD d16 BC,DE,HL
            0x01 => { let v = self.fetchword(memory); self.regs.set_bc(v)} // LD BC d16
            0x11 => { let v = self.fetchword(memory); self.regs.set_de(v)} // LD DE d16
            0x21 => { let v = self.fetchword(memory); self.regs.set_hl(v)} // LD HL d16
            0x31 => { self.regs.sp = self.fetchword(memory)} // LD SP d16

            // LD (Wide) A
            0x02 => { memory.write_byte(self.regs.get_bc(), self.regs.a)} // LD (BC) A
            0x12 => { memory.write_byte(self.regs.get_de(), self.regs.a);} // LD (DE) A
            0x22 => { memory.write_byte(self.regs.get_hl(), self.regs.a); 
                let v = self.regs.get_hl(); memory.write_byte(v, self.regs.a); self.regs.set_hl(v.wrapping_add(1));} // LD (HL+) A
            0x32 => { memory.write_byte(self.regs.get_hl(), self.regs.a);
                let v = self.regs.get_hl(); memory.write_byte(v, self.regs.a); self.regs.set_hl(v.wrapping_sub(1));} // LD (HL-) A

            // LD A (Wide)
            0x0A => { self.regs.a = memory.read_byte(self.regs.get_bc())} // LD A (BC)
            0x1A => { self.regs.a = memory.read_byte(self.regs.get_de())} // LD A (DE)
            0x2A => { self.regs.a = memory.read_byte(self.regs.get_hl());
                let v = self.regs.get_hl(); self.regs.a = memory.read_byte(v); self.regs.set_hl(v.wrapping_add(1))} // LD A (HL+)
            0x3A => { self.regs.a = memory.read_byte(self.regs.get_hl());
                let v = self.regs.get_hl(); self.regs.a = memory.read_byte(v); self.regs.set_hl(v.wrapping_sub(1))} // LD A (HL-)

            // LD (HL) B,C,D,E,H,L
            0x70 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.b)} // LD (HL) B
            0x71 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.c)} // LD (HL) C
            0x72 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.d)} // LD (HL) D
            0x73 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.e)} // LD (HL) E
            0x74 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.h)} // LD (HL) H
            0x75 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.l)} // LD (HL) L
            0x77 => {let addr = self.regs.get_hl(); memory.write_byte(addr, self.regs.a)} // LD (HL) A

            // LD B,D,H,(HL) d8
            0x06 => { self.regs.b = self.fetchbyte(memory)} // LD B d8
            0x16 => { self.regs.d = self.fetchbyte(memory)} // LD D d8
            0x26 => { self.regs.h = self.fetchbyte(memory)} // LD H d8
            0x36 => { memory.write_byte(self.regs.get_hl(), self.fetchbyte(memory))} // LD (HL) d8

            // LD C, E, L, A  d8
            0x0E => { self.regs.c = self.fetchbyte(memory)} // LD C d8
            0x1E => { self.regs.e = self.fetchbyte(memory)} // LD E d8
            0x2E => { self.regs.l = self.fetchbyte(memory)} // LD L d8
            0x3E => { self.regs.a = self.fetchbyte(memory)} // LD A d8 

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
            0x09 => { let v = self.regs.get_bc(); self.op_addword(v);} // ADD HL BC
            0x19 => { let v = self.regs.get_de(); self.op_addword(v);} // ADD HL DE
            0x29 => { let v = self.regs.get_hl(); self.op_addword(v);} // ADD HL HL
            0x39 => { let v = self.regs.sp; self.op_addword(v);} // ADD HL SP

            // Increment B, D, H
            0x04 => {self.regs.b = self.op_inc(self.regs.b)} // INC B
            0x14 => {self.regs.d = self.op_inc(self.regs.d)} // INC D
            0x24 => {self.regs.h = self.op_inc(self.regs.h)} // INC H
            0x34 => { let addr = self.regs.get_hl();
                memory.write_byte(addr, self.op_inc(memory.read_byte(addr)))} // INC (HL)

            // Decrement B, D, H
            0x05 => { self.regs.b = self.op_dec(self.regs.b)} // DEC B
            0x15 => { self.regs.d = self.op_dec(self.regs.d)} // DEC D
            0x25 => { self.regs.h = self.op_dec(self.regs.h)} // DEC H
            0x35 => { let addr = self.regs.get_hl(); 
                memory.write_byte(addr, self.op_dec(memory.read_byte(addr)))} // DEC (HL)

            // Set carry flag CF
            0x37 => {self.regs.set_carry_flag(true); self.regs.set_halfcarry_flag(false); self.regs.set_subtract_flag(false);} // SCF

            // Rotate
            0x07 => { self.regs.a = self.op_rlc(self.regs.a); self.regs.set_zero_flag(false);} // RLCA
            0x0F => { self.regs.a = self.op_rrc(self.regs.a); self.regs.set_zero_flag(false); } // RRCA
            0x17 => { self.regs.a = self.op_rl(self.regs.a); self.regs.set_zero_flag(false);} // RLA
            0x1F => { self.regs.a = self.op_rr(self.regs.a); self.regs.set_zero_flag(false); } // RRA

            // Store stack pointer at address
            0x08 => {memory.write_word(self.fetchword(memory), self.regs.sp)} // LD (a16), SP

            // Increment C, E, L, A
            0x0C => { self.regs.c = self.op_inc(self.regs.c)} // INC C
            0x1C => { self.regs.e = self.op_inc(self.regs.e)} // INC E
            0x2C => { self.regs.l = self.op_inc(self.regs.l)} // INC L
            0x3C => { self.regs.a = self.op_inc(self.regs.a)} // INC A

            // Decrement C, E, L, A
            0x0D => { self.regs.c = self.op_dec(self.regs.c)} // DEC C
            0x1D => { self.regs.e = self.op_dec(self.regs.e)} // DEC E
            0x2D => { self.regs.l = self.op_dec(self.regs.l)} // DEC L
            0x3D => { self.regs.a = self.op_dec(self.regs.a)} // DEC A

            // Complement A
            0x2F => { self.regs.a = !self.regs.a; self.regs.set_subtract_flag(true); self.regs.set_halfcarry_flag(true);} // CPL

            // Complement carry flag
            0x3F => {  // CCF
                let c = self.regs.get_carry_flag(); self.regs.set_carry_flag(!c);
                self.regs.set_halfcarry_flag(false); self.regs.set_subtract_flag(false);
            } 

            // Load into B
            0x40 => { } // LD B B (NOP)
            0x41 => { self.regs.b = self.regs.c} // LD B C
            0x42 => { self.regs.b = self.regs.d} // LD B D 
            0x43 => { self.regs.b = self.regs.e} // LD B E
            0x44 => { self.regs.b = self.regs.h} // LD B H
            0x45 => { self.regs.b = self.regs.l} // LD B L
            0x46 => {self.regs.b = memory.read_byte(self.regs.get_hl())} // LD B (HL)
            0x47 => { self.regs.b = self.regs.a} // LD B A

            // Load into C
            0x48 => { self.regs.c = self.regs.b} // LD C B
            0x49 => { } // LD C C (NOP)
            0x4A => { self.regs.c = self.regs.d} // LD C D
            0x4B => { self.regs.c = self.regs.e} // LD C E
            0x4C => { self.regs.c = self.regs.h} // LD C H
            0x4D => { self.regs.c = self.regs.l} // LD C L
            0x4E => {self.regs.c = memory.read_byte(self.regs.get_hl())} // LD C (HL)
            0x4F => { self.regs.c = self.regs.a} // LD C A

            // Load into D
            0x50 => { self.regs.d = self.regs.b} // LD D B
            0x51 => { self.regs.d = self.regs.c} // LD D C
            0x52 => { } // LD D D (NOOP)
            0x53 => { self.regs.d = self.regs.e} // LD D E
            0x54 => { self.regs.d = self.regs.h} // LD D H
            0x55 => { self.regs.d = self.regs.l} // LD D L
            0x56 => {self.regs.d = memory.read_byte(self.regs.get_hl())} // LD D (HL)
            0x57 => { self.regs.d = self.regs.a} // LD D A

            // Load into E
            0x58 => { self.regs.e = self.regs.b} // LD E B
            0x59 => { self.regs.e = self.regs.c} // LD E C 
            0x5A => { self.regs.e = self.regs.d} // LD E D
            0x5B => { } // LD E E (NOP)
            0x5C => { self.regs.e = self.regs.h} // LD E H
            0x5D => { self.regs.e = self.regs.l} // LD E L
            0x5E => {self.regs.e = memory.read_byte(self.regs.get_hl())} // LD E (HL)
            0x5F => { self.regs.e = self.regs.a} // LD E A

            // Load into H
            0x60 => { self.regs.h = self.regs.b} // LD H B
            0x61 => { self.regs.h = self.regs.c} // LD H C
            0x62 => { self.regs.h = self.regs.d} // LD H D 
            0x63 => { self.regs.h = self.regs.e} // LD H E
            0x64 => { } // LD H H (NOP)
            0x65 => { self.regs.h = self.regs.l} // LD H L
            0x66 => {self.regs.h = memory.read_byte(self.regs.get_hl())} // LD H (HL)
            0x67 => { self.regs.h = self.regs.a} // LD H A

            // Load into L
            0x68 => { self.regs.l = self.regs.b} // LD L B
            0x69 => { self.regs.l = self.regs.c} // LD L C 
            0x6A => { self.regs.l = self.regs.d} // LD L D
            0x6B => { self.regs.l = self.regs.e} // LD L E
            0x6C => { self.regs.l = self.regs.h} // LD L H
            0x6D => { } // LD L L (NOP)
            0x6E => {self.regs.l = memory.read_byte(self.regs.get_hl())} // LD L (HL)
            0x6F => { self.regs.l = self.regs.a} // LD L A

            // Load into A
            0x78 => { self.regs.a = self.regs.b} // LD A B
            0x79 => { self.regs.a = self.regs.c} // LD A C 
            0x7A => { self.regs.a = self.regs.d} // LD A D
            0x7B => { self.regs.a = self.regs.e} // LD A E
            0x7C => { self.regs.a = self.regs.h} // LD A H
            0x7D => { self.regs.a = self.regs.l} // LD A L
            0x7E => {self.regs.a = memory.read_byte(self.regs.get_hl())} // LD A (HL)
            0x7F => { } // LD A A (NOOP)

            // Add instruction
            0x80 => { self.op_add(self.regs.b, false); } // ADD B
            0x81 => { self.op_add(self.regs.c, false); } // ADD C
            0x82 => { self.op_add(self.regs.d, false); } // ADD D
            0x83 => { self.op_add(self.regs.e, false); } // ADD E
            0x84 => { self.op_add(self.regs.h, false); } // ADD H
            0x85 => { self.op_add(self.regs.l, false); } // ADD L
            0x86 => { let v = memory.read_byte(self.regs.get_hl()); self.op_add(v, false); } // ADD (HL)
            0x87 => { self.op_add(self.regs.a, false); } // ADD A
            0xC6 => { let v = self.fetchbyte(memory);  self.op_add(v, false)} // ADD A d8

            // Add with carry instruction
            0x88 => { self.op_add(self.regs.b, true); } // ADC B
            0x89 => { self.op_add(self.regs.c, true); } // ADC C
            0x8A => { self.op_add(self.regs.d, true); } // ADC D
            0x8B => { self.op_add(self.regs.e, true); } // ADC E
            0x8C => { self.op_add(self.regs.h, true); } // ADC H
            0x8D => { self.op_add(self.regs.l, true); } // ADC L
            0x8E => { let v = memory.read_byte(self.regs.get_hl()); self.op_add(v, true); } // ADC (HL)
            0x8F => { self.op_add(self.regs.a, true); } // ADC A
            0xCE => { let v = self.fetchbyte(memory); self.op_add(v, true)} // ADC A d8

            // Sub instruction
            0x90 => { self.op_sub(self.regs.b, false); } // SUB B
            0x91 => { self.op_sub(self.regs.c, false); } // SUB C
            0x92 => { self.op_sub(self.regs.d, false); } // SUB D
            0x93 => { self.op_sub(self.regs.e, false); } // SUB E
            0x94 => { self.op_sub(self.regs.h, false); } // SUB H
            0x95 => { self.op_sub(self.regs.l, false); } // SUB L
            0x96 => { let v = memory.read_byte(self.regs.get_hl()); self.op_sub(v, false); } // SUB (HL)
            0x97 => { self.op_sub(self.regs.a, false); } // SUB A
            0xD6 => { let v = self.fetchbyte(memory); self.op_sub(v, false) } // SUB d8

            // Sub with carry instruction
            0x98 => { self.op_sub(self.regs.b, true); } // SBC B
            0x99 => { self.op_sub(self.regs.c, true); } // SBC C
            0x9A => { self.op_sub(self.regs.d, true); } // SBC D
            0x9B => { self.op_sub(self.regs.e, true); } // SBC E
            0x9C => { self.op_sub(self.regs.h, true); } // SBC H
            0x9D => { self.op_sub(self.regs.l, true); } // SBC L
            0x9E => { let v = memory.read_byte(self.regs.get_hl()); self.op_sub(v, true); } // SUB (HL)
            0x9F => { self.op_sub(self.regs.a, true); } // SBC A
            0xDE => { let v = self.fetchbyte(memory); self.op_sub(v, true) } // SBC d8

            // AND instruction
            0xA0 => { self.op_and(self.regs.b); } // AND B
            0xA1 => { self.op_and(self.regs.c); } // AND C
            0xA2 => { self.op_and(self.regs.d); } // AND D
            0xA3 => { self.op_and(self.regs.e); } // AND E
            0xA4 => { self.op_and(self.regs.h); } // AND H
            0xA5 => { self.op_and(self.regs.l); } // AND L
            0xA6 => { let v = memory.read_byte(self.regs.get_hl()); self.op_and(v); } // AND (HL)
            0xA7 => { self.op_and(self.regs.a);} // AND A
            0xE6 => { let v = self.fetchbyte(memory); self.op_and(v)} // AND d8

            // XOR  instruction
            0xA8 => { self.op_xor(self.regs.b); } // XOR B
            0xA9 => { self.op_xor(self.regs.c); } // XOR C
            0xAA => { self.op_xor(self.regs.d); } // XOR D
            0xAB => { self.op_xor(self.regs.e); } // XOR E
            0xAC => { self.op_xor(self.regs.h); } // XOR H
            0xAD => { self.op_xor(self.regs.l); } // XOR L
            0xAE => { let v = memory.read_byte(self.regs.get_hl()); self.op_xor(v); } // XOR (HL)
            0xAF => { self.op_xor(self.regs.a) } // XOR A
            0xEE => { let v = self.fetchbyte(memory); self.op_xor(v)} // XOR d8

            // OR instruction
            0xB0 => { self.op_or(self.regs.b); } // OR B
            0xB1 => { self.op_or(self.regs.c); } // OR C
            0xB2 => { self.op_or(self.regs.d); } // OR D
            0xB3 => { self.op_or(self.regs.e); } // OR E
            0xB4 => { self.op_or(self.regs.h); } // OR H
            0xB5 => { self.op_or(self.regs.l); } // OR L
            0xB6 => { let v = memory.read_byte(self.regs.get_hl()); self.op_or(v); } // OR (HL)
            0xB7 => { self.op_or(self.regs.a) } // OR A
            0xF6 => { let v = self.fetchbyte(memory); self.op_or(v)} // OR d8

            // CP instruction (set zero flag if the registers are equal)
            0xB8 => { self.op_cp(self.regs.b); } // CP B
            0xB9 => { self.op_cp(self.regs.c); } // CP C
            0xBA => { self.op_cp(self.regs.d); } // CP D
            0xBB => { self.op_cp(self.regs.e); } // CP E
            0xBC => { self.op_cp(self.regs.h); } // CP H
            0xBD => { self.op_cp(self.regs.l); } // CP L
            0xBE => { let v = memory.read_byte(self.regs.get_hl()); self.op_cp(v); } // CP (HL)
            0xBF => { self.op_cp(self.regs.a); } // CP A, (A == A)
            0xFE => { let v = self.fetchbyte(memory); self.op_cp(v); } // CP A d8

            // LD (8 bit, high ram)
            0xE0 => { memory.write_byte(0xFF00 | self.fetchbyte(memory) as u16, self.regs.a) } // LD (a8) A
            0xF0 => { 
                let addr = 0xFF00 | self.fetchbyte(memory) as u16;
                self.regs.a = memory.read_byte(addr) } // LD A (a8)

            // LD (C, high ram)
            0xE2 => { memory.write_byte(0xFF00 | self.regs.c as u16, self.regs.a) } // LD (C) A
            0xF2 => { self.regs.a = memory.read_byte(0xFF00 | self.regs.c as u16) } // LD A (C)            

            // LD A 
            0xEA => { 
                let addr = self.fetchword(memory); 
                memory.write_byte(addr, self.regs.a)} // LD (a16) A
            0xFA => { self.regs.a = memory.read_byte(self.fetchword(memory))} // LD A (a16)

            // Other instructions
            0xF9 => { self.regs.sp = self.regs.get_hl()} // LD HL SP
            0xE8 => { self.regs.sp = self.op_addwordimm(memory, self.regs.sp)} // ADD SP s8
            0xF8 => {
                let r = self.op_addwordimm(memory, self.regs.sp); 
                self.regs.set_hl(r);
            } //LD HL SP+s8

            0x27 => { self.op_daa()} // DAA

            // Stack
            // Push
            0xC5 => { self.push_stack(memory, self.regs.get_bc())} // PUSH BC
            0xD5 => { self.push_stack(memory, self.regs.get_de())} // PUSH DE
            0xE5 => { self.push_stack(memory, self.regs.get_hl())} // PUSH HL
            0xF5 => { self.push_stack(memory, self.regs.get_af())} // PUSH AF

            // Pop
            0xC1 => { let v = self.pop_stack(memory); self.regs.set_bc(v)} // POP BC
            0xD1 => { let v = self.pop_stack(memory); self.regs.set_de(v)} // POP DE
            0xE1 => { let v = self.pop_stack(memory); self.regs.set_hl(v)} // POP HL
            0xF1 => { let v = self.pop_stack(memory); self.regs.set_af(v)} // POP AF
        
            // Call
            0xC4 => { if !self.regs.get_zero_flag() { self.op_call(memory); } else { self.regs.pc += 2 }} // CALL NZ a16
            0xD4 => { if !self.regs.get_carry_flag() { self.op_call(memory); } else { self.regs.pc += 2 }} // CALL NC a16
            0xCC => { if self.regs.get_zero_flag() { self.op_call(memory); } else { self.regs.pc += 2 }}  // CALL Z a16
            0xDC => { if self.regs.get_carry_flag() { self.op_call(memory); } else { self.regs.pc += 2 }} // CALL C a16
            0xCD => { self.op_call(memory); } // CALL a16

            // Ret
            0xC0 => { if !self.regs.get_zero_flag() { self.op_ret(memory); }} // RET NZ
            0xD0 => { if !self.regs.get_carry_flag() { self.op_ret(memory); }} // RET NC
            0xC8 => { if self.regs.get_zero_flag() { self.op_ret(memory); }} // RET Z
            0xD8 => { if self.regs.get_carry_flag() { self.op_ret(memory); }} // RET C
            0xC9 => { self.op_ret(memory);} // RET
            0xD9 => { self.op_ret(memory);} // RETI

            // Restore (call preset locations at start)
            0xC7 => { self.op_restore(memory, 0x00) } // RST 0
            0xD7 => { self.op_restore(memory, 0x10) } // RST 2
            0xE7 => { self.op_restore(memory, 0x20) } // RST 4
            0xF7 => { self.op_restore(memory, 0x30) } // RST 6
            0xCF => { self.op_restore(memory, 0x08) } // RST 1
            0xDF => { self.op_restore(memory, 0x18) } // RST 3
            0xEF => { self.op_restore(memory, 0x28) } // RST 5
            0xFF => { self.op_restore(memory, 0x38) } // RST 7

            // Relative jumps
            0x20 => { if !self.regs.get_zero_flag() { self.op_jump_relative(memory); } else { self.regs.pc += 1; }}  // JR NZ s8 
            0x30 => { if !self.regs.get_carry_flag() { self.op_jump_relative(memory); } else { self.regs.pc += 1; }}  // JR NC s8
            0x18 => { self.op_jump_relative(memory)} // JR s8
            0x28 => { if self.regs.get_zero_flag() { self.op_jump_relative(memory); } else { self.regs.pc += 1; }}  // JR Z s8
            0x38 => { if self.regs.get_carry_flag() { self.op_jump_relative(memory); } else { self.regs.pc += 1; }}  // JR C s8

            // Absolute jumps
            0xC2 => { if !self.regs.get_zero_flag() { self.op_jump(memory); } else { self.regs.pc += 2; }} // JP NZ a16
            0xD2 => { if !self.regs.get_carry_flag() { self.op_jump(memory); } else { self.regs.pc += 2; }} // JP NC a16
            0xC3 => { self.op_jump(memory); } // JP a16
            0xCA => { if self.regs.get_zero_flag() { self.op_jump(memory); } else { self.regs.pc += 2; }} // JP Z a16
            0xDA => { if self.regs.get_carry_flag() { self.op_jump(memory); } else { self.regs.pc += 2; }}  // JP C a16
            0xE9 => { self.regs.pc = self.regs.get_hl()}

            other => panic!("Instruction {:2X} is not implemented", other)
          }
    }

    /// Execute the extra 256 instructions, which are preceeded by the CB instruction
    fn execute_cb(&mut self, opcode : u8, memory: &mut memory::Memory) {
        match opcode { 
            // RLC
            0x00 => { self.regs.b = self.op_rlc(self.regs.b)} // RLC B
            0x01 => { self.regs.c = self.op_rlc(self.regs.c)} // RLC C
            0x02 => { self.regs.d = self.op_rlc(self.regs.d)} // RLC D
            0x03 => { self.regs.e = self.op_rlc(self.regs.e)} // RLC E
            0x04 => { self.regs.h = self.op_rlc(self.regs.h)} // RLC H
            0x05 => { self.regs.l = self.op_rlc(self.regs.l)} // RLC L
            0x06 => { // RLC (HL)
                let val = self.op_rlc(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x07 => { self.regs.a = self.op_rlc(self.regs.a)} // RLC A

            // RRC
            0x08 => { self.regs.b = self.op_rrc(self.regs.b)} // RRC B
            0x09 => { self.regs.c = self.op_rrc(self.regs.c)} // RRC C
            0x0A => { self.regs.d = self.op_rrc(self.regs.d)} // RRC D
            0x0B => { self.regs.e = self.op_rrc(self.regs.e)} // RRC E
            0x0C => { self.regs.h = self.op_rrc(self.regs.h)} // RRC H
            0x0D => { self.regs.l = self.op_rrc(self.regs.l)} // RRC L
            0x0E => { // RRC (HL)
                let val = self.op_rrc(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x0F => { self.regs.a = self.op_rrc(self.regs.a)} // RRC A

            // RL
            0x10 => { self.regs.b = self.op_rl(self.regs.b)} // RL B
            0x11 => { self.regs.c = self.op_rl(self.regs.c)} // RL C
            0x12 => { self.regs.d = self.op_rl(self.regs.d)} // RL D
            0x13 => { self.regs.e = self.op_rl(self.regs.e)} // RL E
            0x14 => { self.regs.h = self.op_rl(self.regs.h)} // RL H
            0x15 => { self.regs.l = self.op_rl(self.regs.l)} // RL L
            0x16 => { // RL (HL)
                let val = self.op_rl(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x17 => { self.regs.a = self.op_rl(self.regs.a)} // RL A

            // RR
            0x18 => { self.regs.b = self.op_rr(self.regs.b)} // RR B
            0x19 => { self.regs.c = self.op_rr(self.regs.c)} // RR C
            0x1A => { self.regs.d = self.op_rr(self.regs.d)} // RR D
            0x1B => { self.regs.e = self.op_rr(self.regs.e)} // RR E
            0x1C => { self.regs.h = self.op_rr(self.regs.h)} // RR H
            0x1D => { self.regs.l = self.op_rr(self.regs.l)} // RR L
            0x1E => { // RR (HL)
                let val = self.op_rr(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x1F => { self.regs.a = self.op_rr(self.regs.a)} // RR A

            // SLA
            0x20 => { self.regs.b = self.op_sla(self.regs.b)} // SLA B
            0x21 => { self.regs.c = self.op_sla(self.regs.c)} // SLA C
            0x22 => { self.regs.d = self.op_sla(self.regs.d)} // SLA D
            0x23 => { self.regs.e = self.op_sla(self.regs.e)} // SLA E
            0x24 => { self.regs.h = self.op_sla(self.regs.h)} // SLA H
            0x25 => { self.regs.l = self.op_sla(self.regs.l)} // SLA L
            0x26 => { // SLA (HL)
                let val = self.op_sla(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x27 => { self.regs.a = self.op_sla(self.regs.a)} // SLA A

            // SRA
            0x28 => { self.regs.b = self.op_sra(self.regs.b)} // SRA B
            0x29 => { self.regs.c = self.op_sra(self.regs.c)} // SRA C
            0x2A => { self.regs.d = self.op_sra(self.regs.d)} // SRA D
            0x2B => { self.regs.e = self.op_sra(self.regs.e)} // SRA E
            0x2C => { self.regs.h = self.op_sra(self.regs.h)} // SRA H
            0x2D => { self.regs.l = self.op_sra(self.regs.l)} // SRA L
            0x2E => { // SRA (HL)
                let val = self.op_sra(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x2F => { self.regs.a = self.op_sra(self.regs.a)} // SRA A
            

            // SWAP
            0x30 => { self.regs.b = self.op_swap(self.regs.b); } // SWAP B
            0x31 => { self.regs.c = self.op_swap(self.regs.c); } // SWAP C
            0x32 => { self.regs.d = self.op_swap(self.regs.d); } // SWAP D
            0x33 => { self.regs.e = self.op_swap(self.regs.e); } // SWAP E
            0x34 => { self.regs.h = self.op_swap(self.regs.h); } // SWAP H
            0x35 => { self.regs.l = self.op_swap(self.regs.l); } // SWAP L
            0x36 => { // SWAP (HL)
                let val = memory.read_byte(self.regs.get_hl()); 
                memory.write_byte(self.regs.get_hl(), self.op_swap(val));
            }
            0x37 => { self.regs.a = self.op_swap(self.regs.a);} // SWAP A

            // SRL
            0x38 => { self.regs.b = self.op_srl(self.regs.b)} // SRL B
            0x39 => { self.regs.c = self.op_srl(self.regs.c)} // SRL C
            0x3A => { self.regs.d = self.op_srl(self.regs.d)} // SRL D
            0x3B => { self.regs.e = self.op_srl(self.regs.e)} // SRL E
            0x3C => { self.regs.h = self.op_srl(self.regs.h)} // SRL H
            0x3D => { self.regs.l = self.op_srl(self.regs.l)} // SRL L
            0x3E => { // SRL (HL)
                let val = self.op_srl(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x3F => { self.regs.a = self.op_srl(self.regs.a)} // SRL A

            // BIT 0
            0x40 => { self.op_read_bit(self.regs.b, 0b0000_0001); } // BIT 0 B
            0x41 => { self.op_read_bit(self.regs.c, 0b0000_0001); } // BIT 0 C
            0x42 => { self.op_read_bit(self.regs.d, 0b0000_0001); } // BIT 0 D
            0x43 => { self.op_read_bit(self.regs.e, 0b0000_0001); } // BIT 0 E
            0x44 => { self.op_read_bit(self.regs.h, 0b0000_0001); } // BIT 0 H
            0x45 => { self.op_read_bit(self.regs.l, 0b0000_0001); } // BIT 0 L
            0x46 => { // BIT 0 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0000_0001); }
            0x47 => { self.op_read_bit(self.regs.a, 0b0000_0001); } // BIT 0 A

            // BIT 1
            0x48 => { self.op_read_bit(self.regs.b, 0b0000_0010); } // BIT 1 B
            0x49 => { self.op_read_bit(self.regs.c, 0b0000_0010); } // BIT 1 C
            0x4A => { self.op_read_bit(self.regs.d, 0b0000_0010); } // BIT 1 D
            0x4B => { self.op_read_bit(self.regs.e, 0b0000_0010); } // BIT 1 E
            0x4C => { self.op_read_bit(self.regs.h, 0b0000_0010); } // BIT 1 H
            0x4D => { self.op_read_bit(self.regs.l, 0b0000_0010); } // BIT 1 L
            0x4E => { // BIT 1 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0000_0010); }
            0x4F => { self.op_read_bit(self.regs.a, 0b0000_0010); } // BIT 1 A

            // BIT 2
            0x50 => { self.op_read_bit(self.regs.b, 0b0000_0100); } // BIT 2 B
            0x51 => { self.op_read_bit(self.regs.c, 0b0000_0100); } // BIT 2 C
            0x52 => { self.op_read_bit(self.regs.d, 0b0000_0100); } // BIT 2 D
            0x53 => { self.op_read_bit(self.regs.e, 0b0000_0100); } // BIT 2 E
            0x54 => { self.op_read_bit(self.regs.h, 0b0000_0100); } // BIT 2 H
            0x55 => { self.op_read_bit(self.regs.l, 0b0000_0100); } // BIT 2 L
            0x56 => { // BIT 2 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0000_0100); }
            0x57 => { self.op_read_bit(self.regs.a, 0b0000_0100); } // BIT 2 A

            // BIT 3
            0x58 => { self.op_read_bit(self.regs.b, 0b0000_1000); } // BIT 3 B
            0x59 => { self.op_read_bit(self.regs.c, 0b0000_1000); } // BIT 3 C
            0x5A => { self.op_read_bit(self.regs.d, 0b0000_1000); } // BIT 3 D
            0x5B => { self.op_read_bit(self.regs.e, 0b0000_1000); } // BIT 3 E
            0x5C => { self.op_read_bit(self.regs.h, 0b0000_1000); } // BIT 3 H
            0x5D => { self.op_read_bit(self.regs.l, 0b0000_1000); } // BIT 3 L
            0x5E => { // BIT 3 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0000_1000); }
            0x5F => { self.op_read_bit(self.regs.a, 0b0000_1000); } // BIT 3 A

            // BIT 4
            0x60 => { self.op_read_bit(self.regs.b, 0b0001_0000); } // BIT 4 B
            0x61 => { self.op_read_bit(self.regs.c, 0b0001_0000); } // BIT 4 C
            0x62 => { self.op_read_bit(self.regs.d, 0b0001_0000); } // BIT 4 D
            0x63 => { self.op_read_bit(self.regs.e, 0b0001_0000); } // BIT 4 E
            0x64 => { self.op_read_bit(self.regs.h, 0b0001_0000); } // BIT 4 H
            0x65 => { self.op_read_bit(self.regs.l, 0b0001_0000); } // BIT 4 L
            0x66 => { // BIT 4 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0001_0000); }
            0x67 => { self.op_read_bit(self.regs.a, 0b0001_0000); } // BIT 4 A

            // BIT 5
            0x68 => { self.op_read_bit(self.regs.b, 0b0010_0000); } // BIT 5 B
            0x69 => { self.op_read_bit(self.regs.c, 0b0010_0000); } // BIT 5 C
            0x6A => { self.op_read_bit(self.regs.d, 0b0010_0000); } // BIT 5 D
            0x6B => { self.op_read_bit(self.regs.e, 0b0010_0000); } // BIT 5 E
            0x6C => { self.op_read_bit(self.regs.h, 0b0010_0000); } // BIT 5 H
            0x6D => { self.op_read_bit(self.regs.l, 0b0010_0000); } // BIT 5 L
            0x6E => { // BIT 5 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0010_0000); }
            0x6F => { self.op_read_bit(self.regs.a, 0b0010_0000); } // BIT 5 A


            // BIT 6
            0x70 => { self.op_read_bit(self.regs.b, 0b0100_0000); } // BIT 6 B
            0x71 => { self.op_read_bit(self.regs.c, 0b0100_0000); } // BIT 6 C
            0x72 => { self.op_read_bit(self.regs.d, 0b0100_0000); } // BIT 6 D
            0x73 => { self.op_read_bit(self.regs.e, 0b0100_0000); } // BIT 6 E
            0x74 => { self.op_read_bit(self.regs.h, 0b0100_0000); } // BIT 6 H
            0x75 => { self.op_read_bit(self.regs.l, 0b0100_0000); } // BIT 6 L
            0x76 => { // BIT 6 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b0100_0000); }
            0x77 => { self.op_read_bit(self.regs.a, 0b0100_0000); } // BIT 6 A

            // BIT 7
            0x78 => { self.op_read_bit(self.regs.b, 0b1000_0000); } // BIT 7 B
            0x79 => { self.op_read_bit(self.regs.c, 0b1000_0000); } // BIT 7 C
            0x7A => { self.op_read_bit(self.regs.d, 0b1000_0000); } // BIT 7 D
            0x7B => { self.op_read_bit(self.regs.e, 0b1000_0000); } // BIT 7 E
            0x7C => { self.op_read_bit(self.regs.h, 0b1000_0000); } // BIT 7 H
            0x7D => { self.op_read_bit(self.regs.l, 0b1000_0000); } // BIT 7 L
            0x7E => { // BIT 7 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.op_read_bit(val, 0b1000_0000); }
            0x7F => { self.op_read_bit(self.regs.a, 0b1000_0000); } // BIT 7 A

            // Reset Bit (Set to 0) Instructions
            // RES 0
            0x80 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0000_0001); } // RES 0 B
            0x81 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0000_0001); } // RES 0 C
            0x82 => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0000_0001); } // RES 0 D
            0x83 => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0000_0001); } // RES 0 E
            0x84 => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0000_0001); } // RES 0 H
            0x85 => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0000_0001); } // RES 0 L
            0x86 => { // RES 0 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0000_0001)); }
            0x87 => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0000_0001); } // RES 0 A

            // RES 1
            0x88 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0000_0010); } // RES 1 B
            0x89 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0000_0010); } // RES 1 C
            0x8A => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0000_0010); } // RES 1 D
            0x8B => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0000_0010); } // RES 1 E
            0x8C => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0000_0010); } // RES 1 H
            0x8D => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0000_0010); } // RES 1 L
            0x8E => { // RES 1 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0000_0010)); }
            0x8F => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0000_0010); } // RES 1 A

            // ERS 2
            0x90 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0000_0100); } // RES 2 B
            0x91 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0000_0100); } // RES 2 C
            0x92 => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0000_0100); } // RES 2 D
            0x93 => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0000_0100); } // RES 2 E
            0x94 => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0000_0100); } // RES 2 H
            0x95 => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0000_0100); } // RES 2 L
            0x96 => { // RES 2 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0000_0100)); }
            0x97 => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0000_0100); } // RES 2 A

            // RES 3
            0x98 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0000_1000); } // RES 3 B
            0x99 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0000_1000); } // RES 3 C
            0x9A => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0000_1000); } // RES 3 D
            0x9B => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0000_1000); } // RES 3 E
            0x9C => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0000_1000); } // RES 3 H
            0x9D => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0000_1000); } // RES 3 L
            0x9E => { // RES 3 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0000_1000)); }
            0x9F => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0000_1000); } // RES 3 A

            // RES 4
            0xA0 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0001_0000); } // RES 4 B
            0xA1 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0001_0000); } // RES 4 C
            0xA2 => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0001_0000); } // RES 4 D
            0xA3 => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0001_0000); } // RES 4 E
            0xA4 => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0001_0000); } // RES 4 H
            0xA5 => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0001_0000); } // RES 4 L
            0xA6 => { // RES 4 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0001_0000)); }
            0xA7 => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0001_0000); } // RES 4 A

            // RES 5
            0xA8 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0010_0000); } // RES 5 B
            0xA9 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0010_0000); } // RES 5 C
            0xAA => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0010_0000); } // RES 5 D
            0xAB => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0010_0000); } // RES 5 E
            0xAC => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0010_0000); } // RES 5 H
            0xAD => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0010_0000); } // RES 5 L
            0xAE => { // RES 5 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0010_0000)); }
            0xAF => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0010_0000); } // RES 5 A

            // RES 6
            0xB0 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b0100_0000); } // RES 6 B
            0xB1 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b0100_0000); } // RES 6 C
            0xB2 => { self.regs.d = self.op_reset_bit(self.regs.d, 0b0100_0000); } // RES 6 D
            0xB3 => { self.regs.e = self.op_reset_bit(self.regs.e, 0b0100_0000); } // RES 6 E
            0xB4 => { self.regs.h = self.op_reset_bit(self.regs.h, 0b0100_0000); } // RES 6 H
            0xB5 => { self.regs.l = self.op_reset_bit(self.regs.l, 0b0100_0000); } // RES 6 L
            0xB6 => { // RES 6 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b0100_0000)); }
            0xB7 => { self.regs.a = self.op_reset_bit(self.regs.a, 0b0100_0000); } // RES 6 A

            // RES 7
            0xB8 => { self.regs.b = self.op_reset_bit(self.regs.b, 0b1000_0000); } // RES 7 B
            0xB9 => { self.regs.c = self.op_reset_bit(self.regs.c, 0b1000_0000); } // RES 7 C
            0xBA => { self.regs.d = self.op_reset_bit(self.regs.d, 0b1000_0000); } // RES 7 D
            0xBB => { self.regs.e = self.op_reset_bit(self.regs.e, 0b1000_0000); } // RES 7 E
            0xBC => { self.regs.h = self.op_reset_bit(self.regs.h, 0b1000_0000); } // RES 7 H
            0xBD => { self.regs.l = self.op_reset_bit(self.regs.l, 0b1000_0000); } // RES 7 L
            0xBE => { // RES 7 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_reset_bit(val, 0b1000_0000)); }
            0xBF => { self.regs.a = self.op_reset_bit(self.regs.a, 0b1000_0000); } // RES 7 A

            // Set bit N to 1
            // SET 0
            0xC0 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0000_0001); } // SET 0 B
            0xC1 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0000_0001); } // SET 0 C
            0xC2 => { self.regs.d = self.op_set_bit(self.regs.d, 0b0000_0001); } // SET 0 D
            0xC3 => { self.regs.e = self.op_set_bit(self.regs.e, 0b0000_0001); } // SET 0 E
            0xC4 => { self.regs.h = self.op_set_bit(self.regs.h, 0b0000_0001); } // SET 0 H
            0xC5 => { self.regs.l = self.op_set_bit(self.regs.l, 0b0000_0001); } // SET 0 L
            0xC6 => { // SET 0 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0000_0001)); }
            0xC7 => { self.regs.a = self.op_set_bit(self.regs.a, 0b0000_0001); } // SET 0 A

            // SET 1
            0xC8 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0000_0010); } // SET 1 B
            0xC9 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0000_0010); } // SET 1 C
            0xCA => { self.regs.d = self.op_set_bit(self.regs.d, 0b0000_0010); } // SET 1 D
            0xCB => { self.regs.e = self.op_set_bit(self.regs.e, 0b0000_0010); } // SET 1 E
            0xCC => { self.regs.h = self.op_set_bit(self.regs.h, 0b0000_0010); } // SET 1 H
            0xCD => { self.regs.l = self.op_set_bit(self.regs.l, 0b0000_0010); } // SET 1 L
            0xCE => { // SET 1 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0000_0010)); }
            0xCF => { self.regs.a = self.op_set_bit(self.regs.a, 0b0000_0010); } // SET 1 A

            // SET 2
            0xD0 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0000_0100); } // SET 2 B
            0xD1 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0000_0100); } // SET 2 C
            0xD2 => { self.regs.d = self.op_set_bit(self.regs.d, 0b0000_0100); } // SET 2 D
            0xD3 => { self.regs.e = self.op_set_bit(self.regs.e, 0b0000_0100); } // SET 2 E
            0xD4 => { self.regs.h = self.op_set_bit(self.regs.h, 0b0000_0100); } // SET 2 H
            0xD5 => { self.regs.l = self.op_set_bit(self.regs.l, 0b0000_0100); } // SET 2 L
            0xD6 => { // SET 2 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0000_0100)); }
            0xD7 => { self.regs.a = self.op_set_bit(self.regs.a, 0b0000_0100); } // SET 2 A

            // SET 3
            0xD8 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0000_1000); } // SET 3 B
            0xD9 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0000_1000); } // SET 3 C
            0xDA => { self.regs.d = self.op_set_bit(self.regs.d, 0b0000_1000); } // SET 3 D
            0xDB => { self.regs.e = self.op_set_bit(self.regs.e, 0b0000_1000); } // SET 3 E
            0xDC => { self.regs.h = self.op_set_bit(self.regs.h, 0b0000_1000); } // SET 3 H
            0xDD => { self.regs.l = self.op_set_bit(self.regs.l, 0b0000_1000); } // SET 3 L
            0xDE => { // SET 3 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0000_1000)); }
            0xDF => { self.regs.a = self.op_set_bit(self.regs.a, 0b0000_1000); } // SET 3 A

            // SET 4
            0xE0 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0001_0000); } // SET 4 B
            0xE1 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0001_0000); } // SET 4 C
            0xE2 => { self.regs.d = self.op_set_bit(self.regs.d, 0b0001_0000); } // SET 4 D
            0xE3 => { self.regs.e = self.op_set_bit(self.regs.e, 0b0001_0000); } // SET 4 E
            0xE4 => { self.regs.h = self.op_set_bit(self.regs.h, 0b0001_0000); } // SET 4 H
            0xE5 => { self.regs.l = self.op_set_bit(self.regs.l, 0b0001_0000); } // SET 4 L
            0xE6 => { // SET 4 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0001_0000)); }
            0xE7 => { self.regs.a = self.op_set_bit(self.regs.a, 0b0001_0000); } // SET 4 A

            // SET 5
            0xE8 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0010_0000); } // SET 5 B
            0xE9 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0010_0000); } // SET 5 C
            0xEA => { self.regs.d = self.op_set_bit(self.regs.d, 0b0010_0000); } // SET 5 D
            0xEB => { self.regs.e = self.op_set_bit(self.regs.e, 0b0010_0000); } // SET 5 E
            0xEC => { self.regs.h = self.op_set_bit(self.regs.h, 0b0010_0000); } // SET 5 H
            0xED => { self.regs.l = self.op_set_bit(self.regs.l, 0b0010_0000); } // SET 5 L
            0xEE => { // SET 5 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0010_0000)); }
            0xEF => { self.regs.a = self.op_set_bit(self.regs.a, 0b0010_0000); } // SET 5 A

            // SET 6
            0xF0 => { self.regs.b = self.op_set_bit(self.regs.b, 0b0100_0000); } // SET 6 B
            0xF1 => { self.regs.c = self.op_set_bit(self.regs.c, 0b0100_0000); } // SET 6 C
            0xF2 => { self.regs.d = self.op_set_bit(self.regs.d, 0b0100_0000); } // SET 6 D
            0xF3 => { self.regs.e = self.op_set_bit(self.regs.e, 0b0100_0000); } // SET 6 E
            0xF4 => { self.regs.h = self.op_set_bit(self.regs.h, 0b0100_0000); } // SET 6 H
            0xF5 => { self.regs.l = self.op_set_bit(self.regs.l, 0b0100_0000); } // SET 6 L
            0xF6 => { // SET 6 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b0100_0000)); }
            0xF7 => { self.regs.a = self.op_set_bit(self.regs.a, 0b0100_0000); } // SET 6 A

            // SET 7
            0xF8 => { self.regs.b = self.op_set_bit(self.regs.b, 0b1000_0000); } // SET 7 B
            0xF9 => { self.regs.c = self.op_set_bit(self.regs.c, 0b1000_0000); } // SET 7 C
            0xFA => { self.regs.d = self.op_set_bit(self.regs.d, 0b1000_0000); } // SET 7 D
            0xFB => { self.regs.e = self.op_set_bit(self.regs.e, 0b1000_0000); } // SET 7 E
            0xFC => { self.regs.h = self.op_set_bit(self.regs.h, 0b1000_0000); } // SET 7 H
            0xFD => { self.regs.l = self.op_set_bit(self.regs.l, 0b1000_0000); } // SET 7 L
            0xFE => { // SET 7 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.op_set_bit(val, 0b1000_0000)); }
            0xFF => { self.regs.a = self.op_set_bit(self.regs.a, 0b1000_0000); } // SET 7 A

            other => panic!("Instruction 0xCB {0:#04x} is not implemented", other)
        }
    }

    /// Fetch the byte from memory specified by the pc, and increment pc
    /// Generally used to get the next instruction to execute
    pub fn fetchbyte(&mut self, memory: &memory::Memory) -> u8 
    {
        let byte = memory.read_byte(self.regs.pc);
        self.regs.pc += 1;
        return byte;
    }

    /// Fetch the word from memory specified by the pc, and increment pc
    fn fetchword(&mut self, memory: &memory::Memory) -> u16
    {
        let word = memory.read_word(self.regs.pc);
        self.regs.pc += 2;
        return word;
    }

    // Instruction/opcode helpers below

    /// JR Instruction helper, make a relative (signed) jump from imm s8
    fn op_jump_relative(&mut self, memory: &memory::Memory) {
        let offset = (self.fetchbyte(memory) as i8) as i32;
        self.regs.pc = ((self.regs.pc as u32) as i32 + offset) as u16;
    }

    /// JP Instruction helper, make an absolute jump from imm d16
    fn op_jump(&mut self, memory: &memory::Memory) {
        self.regs.pc = self.fetchword(memory);
    }

    /// Push value to stack, and adjust stack pointer
    fn push_stack(&mut self, memory: &mut memory::Memory, reg : u16) {
        self.regs.sp -= 2;
        memory.write_word(self.regs.sp, reg);
    } 

    /// Pop value from stack, and adjust stack pointer
    fn pop_stack(&mut self, memory: &memory::Memory) -> u16 {
        let v = memory.read_word(self.regs.sp);
        self.regs.sp += 2;
        return v;
    }

    /// Push the program counter and move the program counter to the specified imm d16
    fn op_call(&mut self, memory: &mut memory::Memory) {
        self.push_stack(memory, self.regs.pc + 2); self.regs.pc = self.fetchword(memory);
    }

    /// Pop the program counter and return there
    fn op_ret(&mut self, memory : &memory::Memory) {
        self.regs.pc = self.pop_stack(memory);
    }

    /// Push the program counter and go to the address addr
    fn op_restore(&mut self, memory: &mut memory::Memory, addr : u16) {
        self.push_stack(memory, self.regs.pc); self.regs.pc = addr;
    }

    /// ADD Instruction helper.
    /// Z 0 H C
    fn op_add(&mut self, value: u8, use_carry: bool)
    {
        let carry = if use_carry && self.regs.get_carry_flag() { 1 } else { 0 };
        let new_value = self.regs.a.wrapping_add(value).wrapping_add(carry);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag((self.regs.a as u16) + (value as u16) + (carry as u16) > 0xFF);
        self.regs.set_halfcarry_flag((self.regs.a & 0xF) + (value & 0xF) + carry > 0xF);
        self.regs.a = new_value;
    }

    /// Flags: Z 0 H C.
    fn op_addword(&mut self, value: u16)
    {
        let hl = self.regs.get_hl();
        let new_value = hl.wrapping_add(value);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(hl > 0xFFFF - value);
        self.regs.set_halfcarry_flag((hl & 0x07FF) + (value & 0x07FF) > 0x07FF);
        self.regs.set_hl(new_value);
    }

    /// Flags: Z 0 H C.
    fn op_addwordimm(&mut self, memory: &mut memory::Memory, value: u16) -> u16 {
        let imm = self.fetchbyte(memory) as i8 as i16 as u16;
        let new_value = value.wrapping_add(imm);
        self.regs.set_subtract_flag(false);
        self.regs.set_zero_flag(false); 
        self.regs.set_carry_flag((value & 0x00FF) + (imm & 0x00FF) > 0x00FF);
        self.regs.set_halfcarry_flag((value & 0x000F) + (imm & 0x000F) > 0x000F);
        return new_value;
    }

    /// SUB Instruction helper.
    /// Flags: Z 1 H C
    fn op_sub(&mut self, value: u8, use_carry: bool)
    {
        let carry = if use_carry && self.regs.get_carry_flag() { 1 } else { 0 };
        let new_value = self.regs.a.wrapping_sub(value).wrapping_sub(carry);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(true);
        self.regs.set_carry_flag((self.regs.a as u16) < (value as u16) + (carry as u16));
        self.regs.set_halfcarry_flag((self.regs.a & 0x0F) < (value & 0x0F) + carry);
        self.regs.a = new_value;
    }

    /// AND Instruction helper.
    /// Flags: Z 0 1 0
    fn op_and(&mut self, value: u8)
    {
        let new_value = self.regs.a & value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(true);
        self.regs.a = new_value;
    }

    /// XOR Instruction helper.
    /// Flags: Z 0 0 0
    fn op_xor(&mut self, value: u8)
    {
        let new_value = self.regs.a ^ value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(false);
        self.regs.a = new_value;
    }

    /// OR Instruction helper.
    /// Flags: Z 0 0 0
    fn op_or(&mut self, value: u8)
    {
        let new_value = self.regs.a | value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(false);
        self.regs.a = new_value;
    }

    /// INC Instruction helper.
    /// Flags: Z 0 H -
    fn op_inc(&mut self, value: u8) -> u8
    {
        let new_value = value.wrapping_add(1);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_halfcarry_flag((value & 0x0F) + 1 > 0x0F);
        self.regs.set_subtract_flag(false);
        return new_value;
    }

    /// DEC Instruction helper.
    /// Flags: Z 1 H -
    fn op_dec(&mut self, value: u8) -> u8
    {
        let new_value = value.wrapping_sub(1);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_halfcarry_flag((value & 0x0F) == 0);
        self.regs.set_subtract_flag(true);
        return new_value;
    }

    /// Compare instruction helper.
    /// Flags: SUB handles the flags
    fn op_cp(&mut self, b: u8) {
        let r = self.regs.a;
        self.op_sub(b, false);
        self.regs.a = r;
    }

    // Turn A into proper BCD encoding after ADD/SUB has been done between two BCD numbers
    // BCD: Binary coded decimal. 4 bits (nibble) for one digit, 4 bits for another.
    // Ex: 0x91 = 91, 0b0100_0010 = 42
    // Flags: Z - 0 C 
    fn op_daa(&mut self)  {
        let mut a = self.regs.a;
        let mut adjust = if self.regs.get_carry_flag() { 0x60 } else { 0x00 };
        if self.regs.get_halfcarry_flag() { adjust |= 0x06; };

        if !self.regs.get_subtract_flag() { // Adjust for ADD
            if a & 0x0F > 0x09 { adjust |= 0x06; };
            if a > 0x99 { adjust |= 0x60; };
            a = a.wrapping_add(adjust);
        } 
        else { // Adjust for SUB
            a = a.wrapping_sub(adjust);
        }

        self.regs.set_carry_flag(adjust >= 0x60);
        self.regs.set_halfcarry_flag(false);
        self.regs.set_zero_flag(a == 0);
        self.regs.a = a;
    }

    /// RLC, Rotate left. Set carry as last bit.
    /// Flags: Z 0 0 C
    fn op_rlc(&mut self, value: u8) -> u8 {
        let last_bit = (value & 0x80) == 0x80;
        let new_value = value.rotate_left(1);
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag(last_bit);
        return new_value;
    }

    /// RRC, Rotate Right. Set carry as first bit.
    /// Flags: Z 0 0 C
    fn op_rrc(&mut self, value: u8) -> u8 {
        let last_bit = (value & 0x1) == 0x1;
        let new_value = value.rotate_right(1);
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag(last_bit);
        return new_value; 
    }

    /// RL, Rotate left through the carry. Treat as 9 bit number essentially.
    /// Flags: Z 0 0 C
    fn op_rl(&mut self, value: u8) -> u8 {
        let carry_bit = (self.regs.get_carry_flag() as u8) * 0x1;
        let new_value = value << 1 | carry_bit; // Left shift and set first bit to carry 
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x80) == 0x80); // Set carry to last bit
        return new_value;
    }

    /// RR, Rotate right through the carry. Treat as 9 bit number essentially.
    /// Flags: Z 0 0 C
    fn op_rr(&mut self, value: u8) -> u8 {
        let carry_bit = (self.regs.get_carry_flag() as u8) * 0x80;
        let new_value = value >> 1 | carry_bit; // Right shift and set first bit to carry 
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }

    /// SLA, Shift right, bit 0 becomes zero.
    /// Flags: Z 0 0 C
    fn op_sla(&mut self, value: u8) -> u8 {
        let new_value = value << 1;
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x80) == 0x80); // Set carry to first bit
        return new_value;
    }

    /// SRL, Shift left, bit 7 becomes zero.
    /// Flags: Z 0 0 C
    fn op_srl(&mut self, value: u8) -> u8 {
        let new_value = value >> 1;
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }

    /// SRA, Shift right, bit 7 retains old value.
    /// Flags: Z 0 0 C
    fn op_sra(&mut self, value: u8) -> u8 {
        let new_value = (value >> 1) | (value & 0x80); // Keep bit 7 from old value
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }


    /// SWAP, Swap place of the upper and lower nibble.
    /// Flags: Z 0 0 0
    fn op_swap(&mut self, value: u8) -> u8 {
        let upper = value & 0xF0;
        let lower = value & 0x0F;
        let swapped_value = (upper >> 4) | (lower << 4);
        self.regs.reset_flags_and_set_zero(swapped_value);
        return swapped_value;
    }

    /// BIT N, Set the zero flag to the complement of the selected bit.
    /// Flags: Z 0 1 -
    fn op_read_bit(&mut self, value: u8, bitmask: u8) {
        let val = value & bitmask;
        self.regs.set_zero_flag(val == 0);
        self.regs.set_halfcarry_flag(true);
        self.regs.set_subtract_flag(false);
    }

    /// RES N, Reset the selected bit to 0
    fn op_reset_bit(&mut self, value: u8, bitmask: u8) -> u8 {
        return value & (!bitmask);
    }

    /// SET N, Set the selected bit to 1
    fn op_set_bit(&mut self, value: u8, bitmask: u8) -> u8 {
        return value | bitmask;
    }
}

#[cfg(test)]
mod test
{
    use super::CPU;
    use super::memory::Memory;

    use std::cell::RefCell;

    #[test]
    fn blargg_cpu_instrs()
    {
        let mut output = Vec::<u8>::new();

        // Use lambda instead, this is literally impossible to get to work
        // Store vector as buffer in memory instead
        let mut boxtest = Box::new(output);
        {
            let mut memory = Memory::new();
            let mut cpu = CPU::new();
            memory.serial_callback = boxtest;
            memory.rom.read_from_file("roms/cpu_instrs/cpu_instrs.gb");

            for _i in 0..(63802933 * 2) {
                cpu.cycle(&mut memory);
            }
        }
        //let v: Vec::<u8> = boxtest.to_vec();
        //let s = String::from_utf8_lossy(v);
        //println!("{:?}", s);




    }
}