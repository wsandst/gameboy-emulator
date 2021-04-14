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

    // Performance scheme:
    // We have to draw line by line, with CPU time inbetween
    // This means that the CPU can change memory while we are drawing
    
    // If the gpu memory has been changed during VBLANK (before drawing starts)
    // we generate our tile cache new. This takes ~700 us
    // We then start drawing our lines
    // If we detect a gpu memory change during this, we know we have to
    // redraw every tile. We set screen.recache_tiles_per_row to true,
    // and recalculate the cache for every tile ONCE throughout the frame.
    // Additionally, for every new state change we recalculate the
    // tiles covering our current lines. This means that we can
    // handle cases where the tiles are modified every line 
    // (which will be slow, ~20 ms frame), but also handles most
    // modification cases at 1 ms, and almost all under 4 ms.

    pub fn run_until_draw(&mut self) {
        loop {
            self.step();
            if self.memory.gpu.should_draw_scanline() {
                if self.memory.gpu.state_modified { // No point in drawing if nothing has changed
                    self.screen.draw_line(&self.memory.gpu); 
                }
                self.memory.gpu.scanline_draw_requested = false;
            }
            if self.memory.gpu.screen_draw_requested {
                break;
            }
        }
        self.memory.gpu.state_modified = false;
        self.memory.gpu.screen_draw_requested = false;
    }



    pub fn js_test(&mut self) -> u32 {
        return 10;
    }
}
    