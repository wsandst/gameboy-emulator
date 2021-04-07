mod cpu;
mod memory;
mod rom;
mod timer;
mod interrupts;
mod gpu;
mod screen;

pub struct Emulator
{
    pub cpu : cpu::CPU,
    pub memory: memory::Memory,
    pub screen: screen::Screen,
}

impl Emulator
{
    pub fn new() -> Emulator
    {
        Emulator {cpu : cpu::CPU::new(), memory: memory::Memory::new(), screen: screen::Screen::new()}
    }

    pub fn run(&mut self)
    {
        for _i in 1..1000000 {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let machine_cycles = self.cpu.cycle(&mut self.memory);
        self.memory.cycle_devices(machine_cycles as u16);

        if self.memory.gpu.screen_draw_requested {
            self.draw_frame();
            self.memory.gpu.screen_draw_requested = false;
        }
    }

    pub fn draw_frame(&mut self) {
        self.screen.render_frame(&self.memory.gpu);
    }

    pub fn js_test(&mut self) -> u32 {
        return 10;
    }
}
    