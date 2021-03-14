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
        //self.memory.write_byte(0xFF01, 'u' as u8);
        //self.memory.write_byte(0xFF02, 0x81);
        self.memory.rom.read_from_file("roms/cpu_instrs/individual/06-ld r,r.gb");
        //self.cpu.regs.a = 123;
        //self.cpu.regs.e = 123;
        //self.cpu.regs.debug_display();

        //self.cpu.execute(0x3E, &mut self.memory);
        //self.cpu.execute(0xB3, &mut self.memory);
        //self.cpu.regs.debug_display();
    }
}
    