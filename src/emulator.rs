mod cpu;
mod memory;

pub struct Emulator
{
    cpu : cpu::CPU,
    memory: memory::Memory,
}

impl Emulator
{
    pub fn new() -> Emulator
    {
        Emulator {cpu : cpu::CPU::new(), memory: memory::Memory::new()}
    }

    pub fn run(&mut self)
    {
        //self.cpu.registers.set_af(60000);
        self.cpu.regs.a = 123;
        self.cpu.regs.e = 123;
        self.cpu.regs.debug_display();

        self.cpu.execute(0x3E);
        self.cpu.execute(0xB3);
        self.cpu.regs.debug_display();
    }
}
    