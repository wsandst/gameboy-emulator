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
            0x10 => { println!("Program halted");  } // HALT
            0xCB => { let wide_op = self.fetchbyte(memory); self.execute_wide(wide_op, memory);}

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
            0x2F => { self.regs.a = !self.regs.a} // CPL

            // Complement carry flag
            0x3F => { let c = self.regs.get_carry_flag(); self.regs.set_carry_flag(!c)} // CCF

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

            0x1F => { self.regs.a = self.right_shift_from_carry(self.regs.a); self.regs.set_zero_flag(false);} // RRA

            other => panic!("Instruction {:2X} is not implemented", other)
          }
    }

    fn execute_wide(&mut self, opcode : u8, memory: &mut memory::Memory) {
        match opcode { 
            0x18 => { self.regs.b = self.right_shift_from_carry(self.regs.c)} // RR B
            0x19 => { self.regs.c = self.right_shift_from_carry(self.regs.c)} // RR C
            0x1A => { self.regs.d = self.right_shift_from_carry(self.regs.d)} // RR D
            0x1B => { self.regs.e = self.right_shift_from_carry(self.regs.e)} // RR E
            0x1C => { self.regs.h = self.right_shift_from_carry(self.regs.h)} // RR H
            0x1D => { self.regs.l = self.right_shift_from_carry(self.regs.l)} // RR L
            0x1E => { let val = self.right_shift_from_carry(memory.read_byte(self.regs.get_hl())); 
                memory.write_byte(self.regs.get_hl(), val)} // RR (HL)
            0x1F => { self.regs.a = self.right_shift_from_carry(self.regs.a)} // RR A

            0x38 => { self.regs.b = self.right_shift_into_carry(self.regs.b);} // SRL B
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

    fn cp(&mut self, b: u8) {
        let r = self.regs.a;
        self.sub(b, false);
        self.regs.a = r;
    }

    // SRL
    fn right_shift_into_carry(&mut self, value: u8) -> u8 {
        let new_value = value >> 1;
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }

    // RR
    fn right_shift_from_carry(&mut self, value: u8) -> u8 {
        let carry_bit = (self.regs.get_carry_flag() as u8) * 0x80;
        let new_value = value >> 1 | carry_bit; // Right shift and set first bit to carry 
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x1) == 0x1); // Set carry to last bit
        return new_value;
    }

    // SLA
    fn left_shift_into_carry(&mut self, value: u8) -> u8 {
        let new_value = value << 1;
        self.regs.reset_flags_and_set_zero(new_value);
        self.regs.set_carry_flag((value & 0x80) == 0x80); // Set carry to first bit
        return new_value;
    }
}