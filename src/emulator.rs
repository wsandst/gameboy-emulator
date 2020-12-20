mod cpu;

pub struct Emulator
{
    cpu : cpu::CPU
}

impl Emulator
{
    pub fn new() -> Emulator
    {
        Emulator {cpu : cpu::CPU::new()}
    }

    pub fn run(&mut self)
    {
        //self.cpu.registers.set_af(60000);
        self.cpu.regs.b = 10;
        self.cpu.regs.d = 15;
        self.cpu.regs.debug_display();

        self.cpu.execute(0x04);
        self.cpu.execute(0x04);
        self.cpu.execute(0x14);
        self.cpu.execute(0x4A);
        self.cpu.regs.debug_display();
    }
}
    