

// The Gameboy CPU is 8 bit and has 8 registers, which can also be accessed at 16 bit (ex ab)
pub struct Registers {
    pub a : u8,
    pub b : u8,
    pub c : u8,
    pub d : u8,
    pub e : u8,
    pub h : u8,
    pub l : u8,
    pub f : u8, // Flags register
    pub pc: u16, // Program counter
    pub sp: u16 // Stack pointer
}

impl Registers {
    pub fn new() -> Registers
    {
        Registers {a: 0, b: 0, c: 0, d: 0, e:0, h: 0, l: 0, f : 0, pc: 0, sp: 0xFFFE}
    }
    
    // Setters and getters for the 16 bit combined registers af, bc, de and hl
    pub fn get_af(&self) -> u16
    {
        return (self.a as u16) << 8 | (self.f as u16);
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

    // Debug helper which prints the registers
    pub fn debug_display(&mut self)
    {
        println!("a:  {:#010b} ({0})", self.a,);
        println!("f:  {:#010b} ({0})", self.f);
        println!("b:  {:#010b} ({0})", self.b);
        println!("c:  {:#010b} ({0})", self.c);
        println!("d:  {:#010b} ({0})", self.d);
        println!("e:  {:#010b} ({0})", self.e);
        println!("h:  {:#010b} ({0})", self.h);
        println!("l:  {:#010b} ({0})", self.l);
        println!("af: {:#018b} ({0})", self.get_af());
        println!("bc: {:#018b} ({0})", self.get_bc());
        println!("de: {:#018b} ({0})", self.get_de());
        println!("hl: {:#018b} ({0})", self.get_hl());
        println!("sp: {0}", self.sp);
        println!("pc: {0}", self.pc);
        println!("flags: carry={}, halfcarry={}, subtract={}, zero={}", self.get_carry_flag(), self.get_halfcarry_flag(), self.get_subtract_flag(), self.get_zero_flag());
    }
  }