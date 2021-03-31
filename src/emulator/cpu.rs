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
            0x10 => {  } // HALT
            0xCB => { let wide_op = self.fetchbyte(memory); self.execute_wide(wide_op, memory);} // Wide instructions prefix

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
            0x09 => { let v = self.regs.get_bc(); self.addword(v);} // ADD HL BC
            0x19 => { let v = self.regs.get_de(); self.addword(v);} // ADD HL DE
            0x29 => { let v = self.regs.get_hl(); self.addword(v);} // ADD HL HL
            0x39 => { let v = self.regs.sp; self.addword(v);} // ADD HL SP

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
            0x37 => {self.regs.set_carry_flag(true); self.regs.set_halfcarry_flag(false); self.regs.set_subtract_flag(false);} // SCF

            // Rotate
            0x07 => { self.regs.a = self.rotate_left(self.regs.a); self.regs.set_zero_flag(false);} // RLCA
            0x0F => { self.regs.a = self.rotate_right(self.regs.a); self.regs.set_zero_flag(false); } // RRCA
            0x17 => { self.regs.a = self.rotate_left_with_carry(self.regs.a); self.regs.set_zero_flag(false);} // RLA
            0x1F => { self.regs.a = self.rotate_right_with_carry(self.regs.a); self.regs.set_zero_flag(false); } // RRA

            // Store stack pointer at address
            0x08 => {memory.write_word(self.regs.sp, self.fetchword(memory), )} // LD (a16), SP

            // Increment C, E, L, A
            0x0C => { self.regs.c = self.inc(self.regs.c)} // INC C
            0x1C => { self.regs.e = self.inc(self.regs.e)} // INC E
            0x2C => { self.regs.l = self.inc(self.regs.l)} // INC L
            0x3C => { self.regs.a = self.inc(self.regs.a)} // INC A

            // Decrement C, E, L, A
            0x0D => { self.regs.c = self.dec(self.regs.c)} // DEC C
            0x1D => { self.regs.e = self.dec(self.regs.e)} // DEC E
            0x2D => { self.regs.l = self.dec(self.regs.l)} // DEC L
            0x3D => { self.regs.a = self.dec(self.regs.a)} // DEC A

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
            0x80 => { self.add(self.regs.b, false); } // ADD B
            0x81 => { self.add(self.regs.c, false); } // ADD C
            0x82 => { self.add(self.regs.d, false); } // ADD D
            0x83 => { self.add(self.regs.e, false); } // ADD E
            0x84 => { self.add(self.regs.h, false); } // ADD H
            0x85 => { self.add(self.regs.l, false); } // ADD L
            0x86 => { let v = memory.read_byte(self.regs.get_hl()); self.add(v, false); } // ADD (HL)
            0x87 => { self.add(self.regs.a, false); } // ADD A
            0xC6 => { let v = self.fetchbyte(memory);  self.add(v, false)} // ADD A d8

            // Add with carry instruction
            0x88 => { self.add(self.regs.b, true); } // ADC B
            0x89 => { self.add(self.regs.c, true); } // ADC C
            0x8A => { self.add(self.regs.d, true); } // ADC D
            0x8B => { self.add(self.regs.e, true); } // ADC E
            0x8C => { self.add(self.regs.h, true); } // ADC H
            0x8D => { self.add(self.regs.l, true); } // ADC L
            0x8E => { let v = memory.read_byte(self.regs.get_hl()); self.add(v, true); } // ADC (HL)
            0x8F => { self.add(self.regs.a, true); } // ADC A
            0xCE => { let v = self.fetchbyte(memory); self.add(v, true)} // ADC A d8

            // Sub instruction
            0x90 => { self.sub(self.regs.b, false); } // SUB B
            0x91 => { self.sub(self.regs.c, false); } // SUB C
            0x92 => { self.sub(self.regs.d, false); } // SUB D
            0x93 => { self.sub(self.regs.e, false); } // SUB E
            0x94 => { self.sub(self.regs.h, false); } // SUB H
            0x95 => { self.sub(self.regs.l, false); } // SUB L
            0x96 => { let v = memory.read_byte(self.regs.get_hl()); self.sub(v, false); } // SUB (HL)
            0x97 => { self.sub(self.regs.a, false); } // SUB A
            0xD6 => { let v = self.fetchbyte(memory); self.sub(v, false) } // SUB d8

            // Sub with carry instruction
            0x98 => { self.sub(self.regs.b, true); } // SBC B
            0x99 => { self.sub(self.regs.c, true); } // SBC C
            0x9A => { self.sub(self.regs.d, true); } // SBC D
            0x9B => { self.sub(self.regs.e, true); } // SBC E
            0x9C => { self.sub(self.regs.h, true); } // SBC H
            0x9D => { self.sub(self.regs.l, true); } // SBC L
            0x9E => { let v = memory.read_byte(self.regs.get_hl()); self.sub(v, true); } // SUB (HL)
            0x9F => { self.sub(self.regs.a, true); } // SBC A
            0xDE => { let v = self.fetchbyte(memory); self.sub(v, true) } // SBC d8

            // AND instruction
            0xA0 => { self.and(self.regs.b); } // AND B
            0xA1 => { self.and(self.regs.c); } // AND C
            0xA2 => { self.and(self.regs.d); } // AND D
            0xA3 => { self.and(self.regs.e); } // AND E
            0xA4 => { self.and(self.regs.h); } // AND H
            0xA5 => { self.and(self.regs.l); } // AND L
            0xA6 => { let v = memory.read_byte(self.regs.get_hl()); self.and(v); } // AND (HL)
            0xA7 => { self.and(self.regs.a);} // AND A
            0xE6 => { let v = self.fetchbyte(memory); self.and(v)} // AND d8

            // XOR  instruction
            0xA8 => { self.xor(self.regs.b); } // XOR B
            0xA9 => { self.xor(self.regs.c); } // XOR C
            0xAA => { self.xor(self.regs.d); } // XOR D
            0xAB => { self.xor(self.regs.e); } // XOR E
            0xAC => { self.xor(self.regs.h); } // XOR H
            0xAD => { self.xor(self.regs.l); } // XOR L
            0xAE => { let v = memory.read_byte(self.regs.get_hl()); self.xor(v); } // XOR (HL)
            0xAF => { self.xor(self.regs.a) } // XOR A
            0xEE => { let v = self.fetchbyte(memory); self.xor(v)} // XOR d8

            // OR instruction
            0xB0 => { self.or(self.regs.b); } // OR B
            0xB1 => { self.or(self.regs.c); } // OR C
            0xB2 => { self.or(self.regs.d); } // OR D
            0xB3 => { self.or(self.regs.e); } // OR E
            0xB4 => { self.or(self.regs.h); } // OR H
            0xB5 => { self.or(self.regs.l); } // OR L
            0xB6 => { let v = memory.read_byte(self.regs.get_hl()); self.or(v); } // OR (HL)
            0xB7 => { self.or(self.regs.a) } // OR A
            0xF6 => { let v = self.fetchbyte(memory); self.or(v)} // OR d8

            // CP instruction (set zero flag if the registers are equal)
            0xB8 => { self.cp(self.regs.b); } // CP B
            0xB9 => { self.cp(self.regs.c); } // CP C
            0xBA => { self.cp(self.regs.d); } // CP D
            0xBB => { self.cp(self.regs.e); } // CP E
            0xBC => { self.cp(self.regs.h); } // CP H
            0xBD => { self.cp(self.regs.l); } // CP L
            0xBE => { let v = memory.read_byte(self.regs.get_hl()); self.cp(v); } // CP (HL)
            0xBF => { self.cp(self.regs.a); } // CP A, (A == A)
            0xFE => { let v = self.fetchbyte(memory); self.cp(v); } // CP A d8

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
            0xE8 => { let v = self.fetchbyte(memory) as i8 as i16; 
                self.regs.pc = self.addwordimm(self.regs.pc, v as u16)} // ADD SP s8
            0xF8 => {
                let v = self.fetchbyte(memory) as i8 as i16; 
                let r = self.addwordimm(self.regs.pc, v as u16); 
                self.regs.set_hl(r);
            } //LD HL SP+s8

            0x27 => { self.daa()}// DAA

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
            0xC4 => { if !self.regs.get_zero_flag() { self.call(memory); } else { self.regs.pc += 2 }} // CALL NZ a16
            0xD4 => { if !self.regs.get_carry_flag() { self.call(memory); } else { self.regs.pc += 2 }} // CALL NC a16
            0xCC => { if self.regs.get_zero_flag() { self.call(memory); } else { self.regs.pc += 2 }}  // CALL Z a16
            0xDC => { if self.regs.get_carry_flag() { self.call(memory); } else { self.regs.pc += 2 }} // CALL C a16
            0xCD => { self.call(memory); } // CALL a16

            // Ret
            0xC0 => { if !self.regs.get_zero_flag() { self.ret(memory); }} // RET NZ
            0xD0 => { if !self.regs.get_carry_flag() { self.ret(memory); }} // RET NC
            0xC8 => { if self.regs.get_zero_flag() { self.ret(memory); }} // RET Z
            0xD8 => { if self.regs.get_carry_flag() { self.ret(memory); }} // RET C
            0xC9 => { self.ret(memory);} // RET
            0xD9 => { self.ret(memory);} // RETI

            // Restore (call preset locations at start)
            0xC7 => { self.restore(memory, 0x00) } // RST 0
            0xD7 => { self.restore(memory, 0x10) } // RST 2
            0xE7 => { self.restore(memory, 0x20) } // RST 4
            0xF7 => { self.restore(memory, 0x30) } // RST 6
            0xCF => { self.restore(memory, 0x08) } // RST 1
            0xDF => { self.restore(memory, 0x18) } // RST 3
            0xEF => { self.restore(memory, 0x28) } // RST 5
            0xFF => { self.restore(memory, 0x38) } // RST 7

            // Relative jumps
            0x20 => { if !self.regs.get_zero_flag() { self.jump_relative(memory); } else { self.regs.pc += 1; }}  // JR NZ s8 
            0x30 => { if !self.regs.get_carry_flag() { self.jump_relative(memory); } else { self.regs.pc += 1; }}  // JR NC s8
            0x18 => { self.jump_relative(memory)} // JR s8
            0x28 => { if self.regs.get_zero_flag() { self.jump_relative(memory); } else { self.regs.pc += 1; }}  // JR Z s8
            0x38 => { if self.regs.get_carry_flag() { self.jump_relative(memory); } else { self.regs.pc += 1; }}  // JR C s8

            // Absolute jumps
            0xC2 => { if !self.regs.get_zero_flag() { self.jump(memory); } else { self.regs.pc += 2; }} // JP NZ a16
            0xD2 => { if !self.regs.get_carry_flag() { self.jump(memory); } else { self.regs.pc += 2; }} // JP NC a16
            0xC3 => { self.jump(memory); } // JP a16
            0xCA => { if self.regs.get_zero_flag() { self.jump(memory); } else { self.regs.pc += 2; }} // JP Z a16
            0xDA => { if self.regs.get_carry_flag() { self.jump(memory); } else { self.regs.pc += 2; }}  // JP C a16
            0xE9 => { self.regs.pc = self.regs.get_hl()}

            other => panic!("Instruction {:2X} is not implemented", other)
          }
    }

    fn execute_wide(&mut self, opcode : u8, memory: &mut memory::Memory) {
        match opcode { 
            // RLC
            0x00 => { self.regs.b = self.rotate_left(self.regs.b)} // RLC B
            0x01 => { self.regs.c = self.rotate_left(self.regs.c)} // RLC C
            0x02 => { self.regs.d = self.rotate_left(self.regs.d)} // RLC D
            0x03 => { self.regs.e = self.rotate_left(self.regs.e)} // RLC E
            0x04 => { self.regs.h = self.rotate_left(self.regs.h)} // RLC H
            0x05 => { self.regs.l = self.rotate_left(self.regs.l)} // RLC L
            0x06 => { // RLC (HL)
                let val = self.rotate_left(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x07 => { self.regs.a = self.rotate_left(self.regs.a)} // RLC A

            // RRC
            0x08 => { self.regs.b = self.rotate_right(self.regs.b)} // RRC B
            0x09 => { self.regs.c = self.rotate_right(self.regs.c)} // RRC C
            0x0A => { self.regs.d = self.rotate_right(self.regs.d)} // RRC D
            0x0B => { self.regs.e = self.rotate_right(self.regs.e)} // RRC E
            0x0C => { self.regs.h = self.rotate_right(self.regs.h)} // RRC H
            0x0D => { self.regs.l = self.rotate_right(self.regs.l)} // RRC L
            0x0E => { // RRC (HL)
                let val = self.rotate_right(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x0F => { self.regs.a = self.rotate_right(self.regs.a)} // RRC A

            // RL
            0x10 => { self.regs.b = self.rotate_left_with_carry(self.regs.b)} // RL B
            0x11 => { self.regs.c = self.rotate_left_with_carry(self.regs.c)} // RL C
            0x12 => { self.regs.d = self.rotate_left_with_carry(self.regs.d)} // RL D
            0x13 => { self.regs.e = self.rotate_left_with_carry(self.regs.e)} // RL E
            0x14 => { self.regs.h = self.rotate_left_with_carry(self.regs.h)} // RL H
            0x15 => { self.regs.l = self.rotate_left_with_carry(self.regs.l)} // RL L
            0x16 => { // RL (HL)
                let val = self.rotate_left_with_carry(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x17 => { self.regs.a = self.rotate_left_with_carry(self.regs.a)} // RL A

            // RR
            0x18 => { self.regs.b = self.rotate_right_with_carry(self.regs.b)} // RR B
            0x19 => { self.regs.c = self.rotate_right_with_carry(self.regs.c)} // RR C
            0x1A => { self.regs.d = self.rotate_right_with_carry(self.regs.d)} // RR D
            0x1B => { self.regs.e = self.rotate_right_with_carry(self.regs.e)} // RR E
            0x1C => { self.regs.h = self.rotate_right_with_carry(self.regs.h)} // RR H
            0x1D => { self.regs.l = self.rotate_right_with_carry(self.regs.l)} // RR L
            0x1E => { // RR (HL)
                let val = self.rotate_right_with_carry(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x1F => { self.regs.a = self.rotate_right_with_carry(self.regs.a)} // RR A

            // SLA
            0x20 => { self.regs.b = self.sla(self.regs.b)} // SLA B
            0x21 => { self.regs.c = self.sla(self.regs.c)} // SLA C
            0x22 => { self.regs.d = self.sla(self.regs.d)} // SLA D
            0x23 => { self.regs.e = self.sla(self.regs.e)} // SLA E
            0x24 => { self.regs.h = self.sla(self.regs.h)} // SLA H
            0x25 => { self.regs.l = self.sla(self.regs.l)} // SLA L
            0x26 => { // SLA (HL)
                let val = self.sla(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x27 => { self.regs.a = self.sla(self.regs.a)} // SLA A

            // SRA
            0x28 => { self.regs.b = self.sra(self.regs.b)} // SRA B
            0x29 => { self.regs.c = self.sra(self.regs.c)} // SRA C
            0x2A => { self.regs.d = self.sra(self.regs.d)} // SRA D
            0x2B => { self.regs.e = self.sra(self.regs.e)} // SRA E
            0x2C => { self.regs.h = self.sra(self.regs.h)} // SRA H
            0x2D => { self.regs.l = self.sra(self.regs.l)} // SRA L
            0x2E => { // SRA (HL)
                let val = self.sra(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x2F => { self.regs.a = self.sra(self.regs.a)} // SRA A
            

            // SWAP
            0x30 => { self.regs.b = self.swap(self.regs.b); } // SWAP B
            0x31 => { self.regs.c = self.swap(self.regs.c); } // SWAP C
            0x32 => { self.regs.d = self.swap(self.regs.d); } // SWAP D
            0x33 => { self.regs.e = self.swap(self.regs.e); } // SWAP E
            0x34 => { self.regs.h = self.swap(self.regs.h); } // SWAP H
            0x35 => { self.regs.l = self.swap(self.regs.l); } // SWAP L
            0x36 => { // SWAP (HL)
                let val = memory.read_byte(self.regs.get_hl()); 
                memory.write_byte(self.regs.get_hl(), self.swap(val));
            }
            0x37 => { self.regs.a = self.swap(self.regs.a);} // SWAP A

            // SRL
            0x38 => { self.regs.b = self.srl(self.regs.b)} // SRL B
            0x39 => { self.regs.c = self.srl(self.regs.c)} // SRL C
            0x3A => { self.regs.d = self.srl(self.regs.d)} // SRL D
            0x3B => { self.regs.e = self.srl(self.regs.e)} // SRL E
            0x3C => { self.regs.h = self.srl(self.regs.h)} // SRL H
            0x3D => { self.regs.l = self.srl(self.regs.l)} // SRL L
            0x3E => { // SRL (HL)
                let val = self.srl(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} 
            0x3F => { self.regs.a = self.srl(self.regs.a)} // SRL A

            // BIT 0
            0x40 => { self.read_bit(self.regs.b, 0b0000_0001); } // BIT 0 B
            0x41 => { self.read_bit(self.regs.c, 0b0000_0001); } // BIT 0 C
            0x42 => { self.read_bit(self.regs.d, 0b0000_0001); } // BIT 0 D
            0x43 => { self.read_bit(self.regs.e, 0b0000_0001); } // BIT 0 E
            0x44 => { self.read_bit(self.regs.h, 0b0000_0001); } // BIT 0 H
            0x45 => { self.read_bit(self.regs.l, 0b0000_0001); } // BIT 0 L
            0x46 => { // BIT 0 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0000_0001); }
            0x47 => { self.read_bit(self.regs.a, 0b0000_0001); } // BIT 0 A

            // BIT 1
            0x48 => { self.read_bit(self.regs.b, 0b0000_0010); } // BIT 1 B
            0x49 => { self.read_bit(self.regs.c, 0b0000_0010); } // BIT 1 C
            0x4A => { self.read_bit(self.regs.d, 0b0000_0010); } // BIT 1 D
            0x4B => { self.read_bit(self.regs.e, 0b0000_0010); } // BIT 1 E
            0x4C => { self.read_bit(self.regs.h, 0b0000_0010); } // BIT 1 H
            0x4D => { self.read_bit(self.regs.l, 0b0000_0010); } // BIT 1 L
            0x4E => { // BIT 1 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0000_0010); }
            0x4F => { self.read_bit(self.regs.a, 0b0000_0010); } // BIT 1 A

            // BIT 2
            0x50 => { self.read_bit(self.regs.b, 0b0000_0100); } // BIT 2 B
            0x51 => { self.read_bit(self.regs.c, 0b0000_0100); } // BIT 2 C
            0x52 => { self.read_bit(self.regs.d, 0b0000_0100); } // BIT 2 D
            0x53 => { self.read_bit(self.regs.e, 0b0000_0100); } // BIT 2 E
            0x54 => { self.read_bit(self.regs.h, 0b0000_0100); } // BIT 2 H
            0x55 => { self.read_bit(self.regs.l, 0b0000_0100); } // BIT 2 L
            0x56 => { // BIT 2 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0000_0100); }
            0x57 => { self.read_bit(self.regs.a, 0b0000_0100); } // BIT 2 A

            // BIT 3
            0x58 => { self.read_bit(self.regs.b, 0b0000_1000); } // BIT 3 B
            0x59 => { self.read_bit(self.regs.c, 0b0000_1000); } // BIT 3 C
            0x5A => { self.read_bit(self.regs.d, 0b0000_1000); } // BIT 3 D
            0x5B => { self.read_bit(self.regs.e, 0b0000_1000); } // BIT 3 E
            0x5C => { self.read_bit(self.regs.h, 0b0000_1000); } // BIT 3 H
            0x5D => { self.read_bit(self.regs.l, 0b0000_1000); } // BIT 3 L
            0x5E => { // BIT 3 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0000_1000); }
            0x5F => { self.read_bit(self.regs.a, 0b0000_1000); } // BIT 3 A

            // BIT 4
            0x60 => { self.read_bit(self.regs.b, 0b0001_0000); } // BIT 4 B
            0x61 => { self.read_bit(self.regs.c, 0b0001_0000); } // BIT 4 C
            0x62 => { self.read_bit(self.regs.d, 0b0001_0000); } // BIT 4 D
            0x63 => { self.read_bit(self.regs.e, 0b0001_0000); } // BIT 4 E
            0x64 => { self.read_bit(self.regs.h, 0b0001_0000); } // BIT 4 H
            0x65 => { self.read_bit(self.regs.l, 0b0001_0000); } // BIT 4 L
            0x66 => { // BIT 4 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0001_0000); }
            0x67 => { self.read_bit(self.regs.a, 0b0001_0000); } // BIT 4 A

            // BIT 5
            0x68 => { self.read_bit(self.regs.b, 0b0010_0000); } // BIT 5 B
            0x69 => { self.read_bit(self.regs.c, 0b0010_0000); } // BIT 5 C
            0x6A => { self.read_bit(self.regs.d, 0b0010_0000); } // BIT 5 D
            0x6B => { self.read_bit(self.regs.e, 0b0010_0000); } // BIT 5 E
            0x6C => { self.read_bit(self.regs.h, 0b0010_0000); } // BIT 5 H
            0x6D => { self.read_bit(self.regs.l, 0b0010_0000); } // BIT 5 L
            0x6E => { // BIT 5 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0010_0000); }
            0x6F => { self.read_bit(self.regs.a, 0b0010_0000); } // BIT 5 A


            // BIT 6
            0x70 => { self.read_bit(self.regs.b, 0b0100_0000); } // BIT 6 B
            0x71 => { self.read_bit(self.regs.c, 0b0100_0000); } // BIT 6 C
            0x72 => { self.read_bit(self.regs.d, 0b0100_0000); } // BIT 6 D
            0x73 => { self.read_bit(self.regs.e, 0b0100_0000); } // BIT 6 E
            0x74 => { self.read_bit(self.regs.h, 0b0100_0000); } // BIT 6 H
            0x75 => { self.read_bit(self.regs.l, 0b0100_0000); } // BIT 6 L
            0x76 => { // BIT 6 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b0100_0000); }
            0x77 => { self.read_bit(self.regs.a, 0b0100_0000); } // BIT 6 A

            // BIT 7
            0x78 => { self.read_bit(self.regs.b, 0b1000_0000); } // BIT 7 B
            0x79 => { self.read_bit(self.regs.c, 0b1000_0000); } // BIT 7 C
            0x7A => { self.read_bit(self.regs.d, 0b1000_0000); } // BIT 7 D
            0x7B => { self.read_bit(self.regs.e, 0b1000_0000); } // BIT 7 E
            0x7C => { self.read_bit(self.regs.h, 0b1000_0000); } // BIT 7 H
            0x7D => { self.read_bit(self.regs.l, 0b1000_0000); } // BIT 7 L
            0x7E => { // BIT 7 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                self.read_bit(val, 0b1000_0000); }
            0x7F => { self.read_bit(self.regs.a, 0b1000_0000); } // BIT 7 A

            // Reset Bit (Set to 0) Instructions
            // RES 0
            0x80 => { self.regs.b = self.reset_bit(self.regs.b, 0b0000_0001); } // RES 0 B
            0x81 => { self.regs.c = self.reset_bit(self.regs.c, 0b0000_0001); } // RES 0 C
            0x82 => { self.regs.d = self.reset_bit(self.regs.d, 0b0000_0001); } // RES 0 D
            0x83 => { self.regs.e = self.reset_bit(self.regs.e, 0b0000_0001); } // RES 0 E
            0x84 => { self.regs.h = self.reset_bit(self.regs.h, 0b0000_0001); } // RES 0 H
            0x85 => { self.regs.l = self.reset_bit(self.regs.l, 0b0000_0001); } // RES 0 L
            0x86 => { // RES 0 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0000_0001)); }
            0x87 => { self.regs.a = self.reset_bit(self.regs.a, 0b0000_0001); } // RES 0 A

            // RES 1
            0x88 => { self.regs.b = self.reset_bit(self.regs.b, 0b0000_0010); } // RES 1 B
            0x89 => { self.regs.c = self.reset_bit(self.regs.c, 0b0000_0010); } // RES 1 C
            0x8A => { self.regs.d = self.reset_bit(self.regs.d, 0b0000_0010); } // RES 1 D
            0x8B => { self.regs.e = self.reset_bit(self.regs.e, 0b0000_0010); } // RES 1 E
            0x8C => { self.regs.h = self.reset_bit(self.regs.h, 0b0000_0010); } // RES 1 H
            0x8D => { self.regs.l = self.reset_bit(self.regs.l, 0b0000_0010); } // RES 1 L
            0x8E => { // RES 1 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0000_0010)); }
            0x8F => { self.regs.a = self.reset_bit(self.regs.a, 0b0000_0010); } // RES 1 A

            // ERS 2
            0x90 => { self.regs.b = self.reset_bit(self.regs.b, 0b0000_0100); } // RES 2 B
            0x91 => { self.regs.c = self.reset_bit(self.regs.c, 0b0000_0100); } // RES 2 C
            0x92 => { self.regs.d = self.reset_bit(self.regs.d, 0b0000_0100); } // RES 2 D
            0x93 => { self.regs.e = self.reset_bit(self.regs.e, 0b0000_0100); } // RES 2 E
            0x94 => { self.regs.h = self.reset_bit(self.regs.h, 0b0000_0100); } // RES 2 H
            0x95 => { self.regs.l = self.reset_bit(self.regs.l, 0b0000_0100); } // RES 2 L
            0x96 => { // RES 2 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0000_0100)); }
            0x97 => { self.regs.a = self.reset_bit(self.regs.a, 0b0000_0100); } // RES 2 A

            // RES 3
            0x98 => { self.regs.b = self.reset_bit(self.regs.b, 0b0000_1000); } // RES 3 B
            0x99 => { self.regs.c = self.reset_bit(self.regs.c, 0b0000_1000); } // RES 3 C
            0x9A => { self.regs.d = self.reset_bit(self.regs.d, 0b0000_1000); } // RES 3 D
            0x9B => { self.regs.e = self.reset_bit(self.regs.e, 0b0000_1000); } // RES 3 E
            0x9C => { self.regs.h = self.reset_bit(self.regs.h, 0b0000_1000); } // RES 3 H
            0x9D => { self.regs.l = self.reset_bit(self.regs.l, 0b0000_1000); } // RES 3 L
            0x9E => { // RES 3 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0000_1000)); }
            0x9F => { self.regs.a = self.reset_bit(self.regs.a, 0b0000_1000); } // RES 3 A

            // RES 4
            0xA0 => { self.regs.b = self.reset_bit(self.regs.b, 0b0001_0000); } // RES 4 B
            0xA1 => { self.regs.c = self.reset_bit(self.regs.c, 0b0001_0000); } // RES 4 C
            0xA2 => { self.regs.d = self.reset_bit(self.regs.d, 0b0001_0000); } // RES 4 D
            0xA3 => { self.regs.e = self.reset_bit(self.regs.e, 0b0001_0000); } // RES 4 E
            0xA4 => { self.regs.h = self.reset_bit(self.regs.h, 0b0001_0000); } // RES 4 H
            0xA5 => { self.regs.l = self.reset_bit(self.regs.l, 0b0001_0000); } // RES 4 L
            0xA6 => { // RES 4 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0001_0000)); }
            0xA7 => { self.regs.a = self.reset_bit(self.regs.a, 0b0001_0000); } // RES 4 A

            // RES 5
            0xA8 => { self.regs.b = self.reset_bit(self.regs.b, 0b0010_0000); } // RES 5 B
            0xA9 => { self.regs.c = self.reset_bit(self.regs.c, 0b0010_0000); } // RES 5 C
            0xAA => { self.regs.d = self.reset_bit(self.regs.d, 0b0010_0000); } // RES 5 D
            0xAB => { self.regs.e = self.reset_bit(self.regs.e, 0b0010_0000); } // RES 5 E
            0xAC => { self.regs.h = self.reset_bit(self.regs.h, 0b0010_0000); } // RES 5 H
            0xAD => { self.regs.l = self.reset_bit(self.regs.l, 0b0010_0000); } // RES 5 L
            0xAE => { // RES 5 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0010_0000)); }
            0xAF => { self.regs.a = self.reset_bit(self.regs.a, 0b0010_0000); } // RES 5 A

            // RES 6
            0xB0 => { self.regs.b = self.reset_bit(self.regs.b, 0b0100_0000); } // RES 6 B
            0xB1 => { self.regs.c = self.reset_bit(self.regs.c, 0b0100_0000); } // RES 6 C
            0xB2 => { self.regs.d = self.reset_bit(self.regs.d, 0b0100_0000); } // RES 6 D
            0xB3 => { self.regs.e = self.reset_bit(self.regs.e, 0b0100_0000); } // RES 6 E
            0xB4 => { self.regs.h = self.reset_bit(self.regs.h, 0b0100_0000); } // RES 6 H
            0xB5 => { self.regs.l = self.reset_bit(self.regs.l, 0b0100_0000); } // RES 6 L
            0xB6 => { // RES 6 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b0100_0000)); }
            0xB7 => { self.regs.a = self.reset_bit(self.regs.a, 0b0100_0000); } // RES 6 A

            // RES 7
            0xB8 => { self.regs.b = self.reset_bit(self.regs.b, 0b1000_0000); } // RES 7 B
            0xB9 => { self.regs.c = self.reset_bit(self.regs.c, 0b1000_0000); } // RES 7 C
            0xBA => { self.regs.d = self.reset_bit(self.regs.d, 0b1000_0000); } // RES 7 D
            0xBB => { self.regs.e = self.reset_bit(self.regs.e, 0b1000_0000); } // RES 7 E
            0xBC => { self.regs.h = self.reset_bit(self.regs.h, 0b1000_0000); } // RES 7 H
            0xBD => { self.regs.l = self.reset_bit(self.regs.l, 0b1000_0000); } // RES 7 L
            0xBE => { // RES 7 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.reset_bit(val, 0b1000_0000)); }
            0xBF => { self.regs.a = self.reset_bit(self.regs.a, 0b1000_0000); } // RES 7 A

            // Set bit N to 1
            // SET 0
            0xC0 => { self.regs.b = self.set_bit(self.regs.b, 0b0000_0001); } // SET 0 B
            0xC1 => { self.regs.c = self.set_bit(self.regs.c, 0b0000_0001); } // SET 0 C
            0xC2 => { self.regs.d = self.set_bit(self.regs.d, 0b0000_0001); } // SET 0 D
            0xC3 => { self.regs.e = self.set_bit(self.regs.e, 0b0000_0001); } // SET 0 E
            0xC4 => { self.regs.h = self.set_bit(self.regs.h, 0b0000_0001); } // SET 0 H
            0xC5 => { self.regs.l = self.set_bit(self.regs.l, 0b0000_0001); } // SET 0 L
            0xC6 => { // SET 0 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0000_0001)); }
            0xC7 => { self.regs.a = self.set_bit(self.regs.a, 0b0000_0001); } // SET 0 A

            // SET 1
            0xC8 => { self.regs.b = self.set_bit(self.regs.b, 0b0000_0010); } // SET 1 B
            0xC9 => { self.regs.c = self.set_bit(self.regs.c, 0b0000_0010); } // SET 1 C
            0xCA => { self.regs.d = self.set_bit(self.regs.d, 0b0000_0010); } // SET 1 D
            0xCB => { self.regs.e = self.set_bit(self.regs.e, 0b0000_0010); } // SET 1 E
            0xCC => { self.regs.h = self.set_bit(self.regs.h, 0b0000_0010); } // SET 1 H
            0xCD => { self.regs.l = self.set_bit(self.regs.l, 0b0000_0010); } // SET 1 L
            0xCE => { // SET 1 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0000_0010)); }
            0xCF => { self.regs.a = self.set_bit(self.regs.a, 0b0000_0010); } // SET 1 A

            // SET 2
            0xD0 => { self.regs.b = self.set_bit(self.regs.b, 0b0000_0100); } // SET 2 B
            0xD1 => { self.regs.c = self.set_bit(self.regs.c, 0b0000_0100); } // SET 2 C
            0xD2 => { self.regs.d = self.set_bit(self.regs.d, 0b0000_0100); } // SET 2 D
            0xD3 => { self.regs.e = self.set_bit(self.regs.e, 0b0000_0100); } // SET 2 E
            0xD4 => { self.regs.h = self.set_bit(self.regs.h, 0b0000_0100); } // SET 2 H
            0xD5 => { self.regs.l = self.set_bit(self.regs.l, 0b0000_0100); } // SET 2 L
            0xD6 => { // SET 2 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0000_0100)); }
            0xD7 => { self.regs.a = self.set_bit(self.regs.a, 0b0000_0100); } // SET 2 A

            // SET 3
            0xD8 => { self.regs.b = self.set_bit(self.regs.b, 0b0000_1000); } // SET 3 B
            0xD9 => { self.regs.c = self.set_bit(self.regs.c, 0b0000_1000); } // SET 3 C
            0xDA => { self.regs.d = self.set_bit(self.regs.d, 0b0000_1000); } // SET 3 D
            0xDB => { self.regs.e = self.set_bit(self.regs.e, 0b0000_1000); } // SET 3 E
            0xDC => { self.regs.h = self.set_bit(self.regs.h, 0b0000_1000); } // SET 3 H
            0xDD => { self.regs.l = self.set_bit(self.regs.l, 0b0000_1000); } // SET 3 L
            0xDE => { // SET 3 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0000_1000)); }
            0xDF => { self.regs.a = self.set_bit(self.regs.a, 0b0000_1000); } // SET 3 A

            // SET 4
            0xE0 => { self.regs.b = self.set_bit(self.regs.b, 0b0001_0000); } // SET 4 B
            0xE1 => { self.regs.c = self.set_bit(self.regs.c, 0b0001_0000); } // SET 4 C
            0xE2 => { self.regs.d = self.set_bit(self.regs.d, 0b0001_0000); } // SET 4 D
            0xE3 => { self.regs.e = self.set_bit(self.regs.e, 0b0001_0000); } // SET 4 E
            0xE4 => { self.regs.h = self.set_bit(self.regs.h, 0b0001_0000); } // SET 4 H
            0xE5 => { self.regs.l = self.set_bit(self.regs.l, 0b0001_0000); } // SET 4 L
            0xE6 => { // SET 4 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0001_0000)); }
            0xE7 => { self.regs.a = self.set_bit(self.regs.a, 0b0001_0000); } // SET 4 A

            // SET 5
            0xE8 => { self.regs.b = self.set_bit(self.regs.b, 0b0010_0000); } // SET 5 B
            0xE9 => { self.regs.c = self.set_bit(self.regs.c, 0b0010_0000); } // SET 5 C
            0xEA => { self.regs.d = self.set_bit(self.regs.d, 0b0010_0000); } // SET 5 D
            0xEB => { self.regs.e = self.set_bit(self.regs.e, 0b0010_0000); } // SET 5 E
            0xEC => { self.regs.h = self.set_bit(self.regs.h, 0b0010_0000); } // SET 5 H
            0xED => { self.regs.l = self.set_bit(self.regs.l, 0b0010_0000); } // SET 5 L
            0xEE => { // SET 5 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0010_0000)); }
            0xEF => { self.regs.a = self.set_bit(self.regs.a, 0b0010_0000); } // SET 5 A

            // SET 6
            0xF0 => { self.regs.b = self.set_bit(self.regs.b, 0b0100_0000); } // SET 6 B
            0xF1 => { self.regs.c = self.set_bit(self.regs.c, 0b0100_0000); } // SET 6 C
            0xF2 => { self.regs.d = self.set_bit(self.regs.d, 0b0100_0000); } // SET 6 D
            0xF3 => { self.regs.e = self.set_bit(self.regs.e, 0b0100_0000); } // SET 6 E
            0xF4 => { self.regs.h = self.set_bit(self.regs.h, 0b0100_0000); } // SET 6 H
            0xF5 => { self.regs.l = self.set_bit(self.regs.l, 0b0100_0000); } // SET 6 L
            0xF6 => { // SET 6 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b0100_0000)); }
            0xF7 => { self.regs.a = self.set_bit(self.regs.a, 0b0100_0000); } // SET 6 A

            // SET 7
            0xF8 => { self.regs.b = self.set_bit(self.regs.b, 0b1000_0000); } // SET 7 B
            0xF9 => { self.regs.c = self.set_bit(self.regs.c, 0b1000_0000); } // SET 7 C
            0xFA => { self.regs.d = self.set_bit(self.regs.d, 0b1000_0000); } // SET 7 D
            0xFB => { self.regs.e = self.set_bit(self.regs.e, 0b1000_0000); } // SET 7 E
            0xFC => { self.regs.h = self.set_bit(self.regs.h, 0b1000_0000); } // SET 7 H
            0xFD => { self.regs.l = self.set_bit(self.regs.l, 0b1000_0000); } // SET 7 L
            0xFE => { // SET 7 (HL)
                let val = memory.read_byte(self.regs.get_hl());
                memory.write_byte(self.regs.get_hl(), self.set_bit(val, 0b1000_0000)); }
            0xFF => { self.regs.a = self.set_bit(self.regs.a, 0b1000_0000); } // SET 7 A

            0x38 => { self.regs.b = self.sra(self.regs.b);} // SRL B
            other => panic!("Instruction 0xCB {0:#04x} is not implemented", other)
        }
    }

    pub fn fetchbyte(&mut self, memory: &memory::Memory) -> u8 
    {
        let byte = memory.read_byte(self.regs.pc);
        self.regs.pc += 1;
        return byte;
    }

    fn fetchword(&mut self, memory: &memory::Memory) -> u16
    {
        let word = memory.read_word(self.regs.pc);
        self.regs.pc += 2;
        return word;
    }

    // JR Instruction
    fn jump_relative(&mut self, memory: &memory::Memory) {
        let offset = (self.fetchbyte(memory) as i8) as i32;
        self.regs.pc = ((self.regs.pc as u32) as i32 + offset) as u16;
    }

    // JP Instruction
    fn jump(&mut self, memory: &memory::Memory) {
        self.regs.pc = self.fetchword(memory);
    }

    fn push_stack(&mut self, memory: &mut memory::Memory, reg : u16) {
        self.regs.sp -= 2;
        memory.write_word(self.regs.sp, reg);
    } 

    fn pop_stack(&mut self, memory: &memory::Memory) -> u16 {
        let v = memory.read_word(self.regs.sp);
        self.regs.sp += 2;
        return v;
    }

    fn call(&mut self, memory: &mut memory::Memory) {
        // self.pushstack(self.reg.pc + 2); self.reg.pc = self.fetchword();
        self.push_stack(memory, self.regs.pc + 2); self.regs.pc = self.fetchword(memory);
    }

    fn ret(&mut self, memory : &memory::Memory) {
        self.regs.pc = self.pop_stack(memory);
    }

    fn restore(&mut self, memory: &mut memory::Memory, addr : u16) {
        self.push_stack(memory, self.regs.pc); self.regs.pc = addr;
    }

    // ADD Instruction
    fn add(&mut self, value: u8, use_carry: bool)
    {
        let carry = if use_carry && self.regs.get_carry_flag() { 1 } else { 0 };
        let new_value = self.regs.a.wrapping_add(value).wrapping_add(carry);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag((self.regs.a as u16) + (value as u16) + (carry as u16) > 0xFF);
        self.regs.set_halfcarry_flag((self.regs.a & 0xF) + (value & 0xF) + carry > 0xF);
        self.regs.a = new_value;
    }

    fn addword(&mut self, value: u16)
    {
        let hl = self.regs.get_hl();
        let new_value = hl.wrapping_add(value);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(hl > 0xFFFF - value);
        self.regs.set_halfcarry_flag((hl & 0x07FF) + (value & 0x07FF) > 0x07FF);
        self.regs.set_hl(new_value);
    }

    fn addwordimm(&mut self, value: u16, imm: u16) -> u16 {
        let new_value = value.wrapping_add(imm);
        self.regs.set_subtract_flag(false);
        self.regs.set_zero_flag(false);
        self.regs.set_carry_flag((value & 0x00FF) + (imm & 0x00FF) > 0x00FF);
        self.regs.set_halfcarry_flag((value & 0x000F) + (imm & 0x000F) > 0x000F);
        return new_value;
    }

    // SUB Instruction
    fn sub(&mut self, value: u8, use_carry: bool)
    {
        let carry = if use_carry && self.regs.get_carry_flag() { 1 } else { 0 };
        let new_value = self.regs.a.wrapping_sub(value).wrapping_sub(carry);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(true);
        self.regs.set_carry_flag((self.regs.a as u16) < (value as u16) + (carry as u16));
        self.regs.set_halfcarry_flag((self.regs.a & 0x0F) < (value & 0x0F) + carry);
        self.regs.a = new_value;
    }

    // AND Instruction
    fn and(&mut self, value: u8)
    {
        let new_value = self.regs.a & value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(true);
        self.regs.a = new_value;
    }

    // XOR Instruction
    fn xor(&mut self, value: u8)
    {
        let new_value = self.regs.a ^ value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(false);
        self.regs.a = new_value;
    }

    // OR Instruction
    fn or(&mut self, value: u8)
    {
        let new_value = self.regs.a | value;
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(false);
        self.regs.set_carry_flag(false);
        self.regs.set_halfcarry_flag(false);
        self.regs.a = new_value;
    }

    // INC Instruction
    fn inc(&mut self, value: u8) -> u8
    {
        let new_value = value.wrapping_add(1);
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_halfcarry_flag((value & 0x0F) + 1 > 0x0F);
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

    // Compare instruction.
    fn cp(&mut self, b: u8) {
        let r = self.regs.a;
        self.sub(b, false);
        self.regs.a = r;
    }

    // Turn A into proper BCD encoding after ADD/SUB has been done between two BCD numbers
    // BCD: Binary coded decimal. 4 bits (nibble) for one digit, 4 bits for another.
    // Ex: 0x91 = 91, 0b0100_0010 = 82
    fn daa(&mut self)  {
        let mut a = self.regs.a;
        if !self.regs.get_subtract_flag() { // ADD
            if self.regs.get_carry_flag() || a > 0x99 { // Upper nibble has carry
                a = a.wrapping_add(0x60);
                self.regs.set_carry_flag(true);
            }
            if self.regs.get_halfcarry_flag() || (a & 0x0f) > 0x09 { // Lower nibble has carry
                a = a.wrapping_add(0x6); 
            }
        }
        else { // SUB
            if self.regs.get_carry_flag() { // Upper nibble has carry
                a = a.wrapping_add(0x60); 
                self.regs.set_carry_flag(true);
            }
            if self.regs.get_halfcarry_flag() { // Lower nibble has carry
                a = a.wrapping_add(0x6); 
            }
        }
        self.regs.set_halfcarry_flag(false);
        self.regs.set_zero_flag(a == 0);
        self.regs.a = a;
    }

    // RLC
    fn rotate_left(&mut self, value: u8) -> u8 {
        let last_bit = (value & 0x80) == 0x80;
        let new_value = value.rotate_left(1);
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag(last_bit);
        return new_value;
    }

    // RRC
    fn rotate_right(&mut self, value: u8) -> u8 {
        let last_bit = (value & 0x1) == 0x1;
        let new_value = value.rotate_right(1);
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag(last_bit);
        return new_value; 
    }

    // RL
    // Rotate left through the carry. Treat as 9 bit number essentially
    fn rotate_left_with_carry(&mut self, value: u8) -> u8 {
        let carry_bit = (self.regs.get_carry_flag() as u8) * 0x1;
        let new_value = value << 1 | carry_bit; // Left shift and set first bit to carry 
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x80) == 0x80); // Set carry to last bit
        return new_value;
    }

    // RR
    // Rotate right through the carry. Treat as 9 bit number essentially
    fn rotate_right_with_carry(&mut self, value: u8) -> u8 {
        let carry_bit = (self.regs.get_carry_flag() as u8) * 0x80;
        let new_value = value >> 1 | carry_bit; // Right shift and set first bit to carry 
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }

    // SLA
    // Shift rigght, bit 0 becomes zero
    fn sla(&mut self, value: u8) -> u8 {
        let new_value = value << 1;
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x80) == 0x80); // Set carry to first bit
        return new_value;
    }

    // SRL
    // Shift left, bit 7 becomes zero
    fn srl(&mut self, value: u8) -> u8 {
        let new_value = value >> 1;
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }

    // SRA
    // Shift right, bit 7 retains old value
    fn sra(&mut self, value: u8) -> u8 {
        let new_value = (value >> 1) | (value & 0x80); // Keep bit 7 from old value
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }


    // SWAP
    // Swap place of the upper and lower nibble
    fn swap(&mut self, value: u8) -> u8 {
        let upper = value & 0xF0;
        let lower = value & 0x0F;
        let swapped_value = (upper >> 4) | (lower << 4);
        self.regs.reset_flags_and_set_zero(swapped_value);
        return swapped_value;
    }

    // BIT N. Set the zero flag to the complement of the selected bit
    fn read_bit(&mut self, value: u8, bitmask: u8) {
        let val = value & bitmask;
        self.regs.set_zero_flag(val == 0);
        self.regs.set_halfcarry_flag(true);
        self.regs.set_subtract_flag(false);
    }

    fn set_bit(&mut self, value: u8, bitmask: u8) -> u8 {
        return value | bitmask;
    }

    fn reset_bit(&mut self, value: u8, bitmask: u8) -> u8 {
        return value & (!bitmask);
    }
}