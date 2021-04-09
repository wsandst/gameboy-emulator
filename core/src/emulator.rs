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

    pub fn run(&mut self, steps : u32)
    {
        for _i in 1..steps {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let machine_cycles = self.cpu.cycle(&mut self.memory);
        self.memory.cycle_devices(machine_cycles as u16);
    }

    pub fn run_until_draw(&mut self) {
        loop {
            self.step();
            if self.memory.gpu.screen_draw_requested {
                break;
            }
        }
        self.draw_frame();
        self.memory.gpu.screen_draw_requested = false;
    }

    pub fn draw_frame(&mut self) {
        self.screen.draw_frame(&self.memory.gpu);
    }

    pub fn js_test(&mut self) -> u32 {
        return 10;
    }
}
    