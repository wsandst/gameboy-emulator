mod cpu;

pub struct Emulator
{
    registers : cpu::Registers
}

impl Emulator
{
    pub fn new() -> Emulator
    {
        Emulator {registers : cpu::Registers {a: 0, b: 0, c: 0, d: 0, e:0, f: 0, h: 0, l: 0}}
    }
    pub fn run(&mut self)
    {
        self.registers.a = 5;
        self.registers.f = 3;
        self.registers.set_af(60000);
        self.registers.set_bc(60001);
        self.registers.set_de(60002);
        self.registers.set_hl(60003);
        self.registers.debug_display();
    }
}
    