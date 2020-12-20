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
        self.cpu.regs.a = 0b10001111;
        self.cpu.regs.b = 1;
        self.cpu.regs.debug_display();

        self.cpu.execute(0x80);
        self.cpu.regs.debug_display();
    }
}
    