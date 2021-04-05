mod cpu;
mod memory;
mod devices;
mod interrupts;

pub struct Emulator
{
    pub cpu : cpu::CPU,
    pub memory: memory::Memory,
}

impl Emulator
{
    pub fn new() -> Emulator
    {
        Emulator {cpu : cpu::CPU::new(), memory: memory::Memory::new()}
    }

    pub fn run(&mut self)
    {
        for _i in 1..63802933 {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.cpu.cycle(&mut self.memory);
    }
}
    