
pub struct Registers {
    pub a : u8,
    pub b : u8,
    pub c : u8,
    pub d : u8,
    pub e : u8,
    pub f : u8,
    pub h : u8,
    pub l : u8,
}

impl Registers {
    pub fn new() -> Registers
    {
        Registers {a: 0, b: 0, c: 0, d: 0, e:0, f: 0, h: 0, l: 0}
    }
    pub fn get_bc(&mut self) -> u16
    {
        return (self.b as u16) << 8 | (self.c as u16);
    }
    pub fn set_bc(&mut self, bc : u16)
    {
        self.b = ((bc & 0xFF00) >> 8) as u8;
        self.c = (bc & 0x00FF) as u8;
    }
    pub fn get_af(&mut self) -> u16
    {
        return (self.a as u16) << 8 | (self.f as u16);
    }
    pub fn set_af(&mut self, af : u16)
    {
        self.a = ((af & 0xFF00) >> 8) as u8;
        self.f = (af & 0x00FF) as u8;
    }
    pub fn get_de(&mut self) -> u16
    {
        return (self.d as u16) << 8 | (self.e as u16);
    }
    pub fn set_de(&mut self, de : u16)
    {
        self.d = ((de & 0xFF00) >> 8) as u8;
        self.e = (de & 0x00FF) as u8;
    }
    pub fn get_hl(&mut self) -> u16
    {
        return (self.h as u16) << 8 | (self.l as u16);
    }
    pub fn set_hl(&mut self, hl : u16)
    {
        self.h = ((hl & 0xFF00) >> 8) as u8;
        self.l = (hl & 0x00FF) as u8;
    }
    pub fn debug_display(&mut self)
    {
        println!("a:  {:#010b}", self.a);
        println!("f:  {:#010b}", self.f);
        println!("b:  {:#010b}", self.b);
        println!("c:  {:#010b}", self.c);
        println!("d:  {:#010b}", self.d);
        println!("e:  {:#010b}", self.e);
        println!("h:  {:#010b}", self.h);
        println!("l:  {:#010b}", self.l);
        println!("af: {:#018b}", self.get_af());
        println!("bc: {:#018b}", self.get_bc());
        println!("de: {:#018b}", self.get_de());
        println!("hl: {:#018b}", self.get_hl());
    }
  }