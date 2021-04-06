

// The Gameboy CPU is 8 bit and has 8 registers, which can also be accessed at 16 bit (ex ab)
pub struct Registers {
    pub a : u8,
    pub b : u8,
    pub c : u8,
    pub d : u8,
    pub e : u8,
    pub h : u8,
    pub l : u8,
    pub f : u8, // Flags register, Bits: ZSHC
    pub pc: u16, // Program counter
    pub sp: u16 // Stack pointer
}

impl Registers {
    pub fn new() -> Registers
    {
        Registers {a: 0x01, b: 0x00, c: 0x13, d: 0x00, e:0xD8, h: 0x01, l: 0x4D, f : 0xB0, pc: 0x100, sp: 0xFFFE}
    }
    
    // Setters and getters for the 16 bit combined registers af, bc, de and hl
    pub fn get_af(&self) -> u16
    {
        return (self.a as u16) << 8 | ((self.f & 0xf0) as u16); // Last 4 bits of F should always be zero.
    }

    pub fn set_af(&mut self, af : u16)
    {
        self.a = ((af & 0xFF00) >> 8) as u8;
        self.f = (af & 0x00FF) as u8;
    }

    pub fn get_bc(&self) -> u16
    {
        return (self.b as u16) << 8 | (self.c as u16);
    }

    pub fn set_bc(&mut self, bc : u16)
    {
        self.b = ((bc & 0xFF00) >> 8) as u8;
        self.c = (bc & 0x00FF) as u8;
    }

    pub fn get_de(&self) -> u16
    {
        return (self.d as u16) << 8 | (self.e as u16);
    }

    pub fn set_de(&mut self, de : u16)
    {
        self.d = ((de & 0xFF00) >> 8) as u8;
        self.e = (de & 0x00FF) as u8;
    }

    pub fn get_hl(&self) -> u16
    {
        return (self.h as u16) << 8 | (self.l as u16);
    }

    pub fn set_hl(&mut self, hl : u16)
    {
        self.h = ((hl & 0xFF00) >> 8) as u8;
        self.l = (hl & 0x00FF) as u8;
    }

    pub fn set_carry_flag(&mut self, value : bool)
    {
        self.f = if value { self.f | 0b00010000 } else { self.f & 0b11101111 };
    }

    pub fn set_halfcarry_flag(&mut self, value : bool)
    {
        self.f = if value { self.f | 0b00100000 } else { self.f & 0b11011111 };
    }

    pub fn set_subtract_flag(&mut self, value : bool)
    {
        self.f = if value { self.f | 0b01000000 } else { self.f & 0b10111111 };
    }

    pub fn set_zero_flag(&mut self, value : bool)
    {
        self.f = if value { self.f | 0b10000000 } else { self.f & 0b01111111 };
    }

    pub fn get_carry_flag(&mut self) -> bool
    {
        (self.f & 0b00010000) != 0
    }

    pub fn get_halfcarry_flag(&mut self) -> bool
    {
        (self.f & 0b00100000) != 0
    }

    pub fn get_subtract_flag(&mut self) -> bool
    {
        (self.f & 0b01000000) != 0
    }

    pub fn get_zero_flag(&mut self) -> bool
    {
        (self.f & 0b10000000) != 0
    }

    pub fn reset_flags_and_set_zero(&mut self, zero_val: u8) {
        self.set_subtract_flag(false);
        self.set_halfcarry_flag(false);
        self.set_carry_flag(false);
        self.set_zero_flag(zero_val == 0);
    }

    // Debug helper which prints the registers
    pub fn debug_display(&mut self)
    {
        println!("a:  {0:#04x}, {0:3}, {0:#010b}", self.a);
        println!("f:  {0:#04x}, {0:3}, {0:#010b}", self.f);
        println!("b:  {0:#04x}, {0:3}, {0:#010b}", self.b);
        println!("c:  {0:#04x}, {0:3}, {0:#010b}", self.c);
        println!("d:  {0:#04x}, {0:3}, {0:#010b}", self.d);
        println!("e:  {0:#04x}, {0:3}, {0:#010b}", self.e);
        println!("h:  {0:#04x}, {0:3}, {0:#010b}", self.h);
        println!("l:  {0:#04x}, {0:3}, {0:#010b}", self.l);
        println!("af: {0:#08x}, {0:5}, {0:#010b}", self.get_af());
        println!("bc: {0:#08x}, {0:5}, {0:#010b}", self.get_bc());
        println!("de: {0:#08x}, {0:5}, {0:#010b}", self.get_de());
        println!("hl: {0:#08x}, {0:5}, {0:#010b}", self.get_hl());
        println!("sp: {0:#01x}, {0}", self.sp);
        println!("pc: {:#01x}, {0}", self.pc);
        println!("flags: carry={}, halfcarry={}, subtract={}, zero={}", self.get_carry_flag(), self.get_halfcarry_flag(), self.get_subtract_flag(), self.get_zero_flag());
    }
  }