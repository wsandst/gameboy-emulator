mod cpu;
mod memory;
mod rom;
mod timer;
mod interrupts;
mod gpu;
mod screen;
mod joypad;
mod audio;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum KeyPress {
    Down,
    Up,
    Left,
    Right,
    Start,
    Select,
    B,
    A
}

pub enum FrontendEvent {
    Render,
    QueueSound
}

pub struct Emulator
{
    pub cpu : cpu::CPU,
    pub memory: memory::Memory,
    pub screen: screen::Screen,
    pub frame_counter: usize,
    pub using_bootrom: bool,
    pub paused: bool,
}

impl Emulator
{
    pub fn new(use_bootrom: bool) -> Emulator
    {
        let mut em = Emulator {
            cpu : cpu::CPU::new(), 
            memory: memory::Memory::new(), 
            screen: screen::Screen::new(), 
            frame_counter: 0,
            using_bootrom: use_bootrom,
            paused: false,
        };
        if use_bootrom {
            em.cpu.regs.pc = 0;
            em.memory.rom.using_boot_rom = true;
        }
        return em;
    }

    pub fn run(&mut self, steps : u32)
    {
        for _i in 1..steps {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let machine_cycles = self.cpu.cycle(&mut self.memory);
        self.memory.cycle_devices(machine_cycles as usize);
    }

    /// Run the Gameboy until next draw is requested
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

    pub fn run_until_frontend_event(&mut self) -> FrontendEvent {
        if self.paused {
            return FrontendEvent::Render;
        }
        loop {
            self.step();
            if self.memory.gpu.should_draw_scanline() {
                if self.memory.gpu.state_modified { // No point in drawing if nothing has changed
                    self.screen.draw_line(&self.memory.gpu); 
                }
                self.memory.gpu.scanline_draw_requested = false;
            }
            if self.memory.gpu.screen_draw_requested {
                self.memory.gpu.state_modified = false;
                self.memory.gpu.screen_draw_requested = false;
                return FrontendEvent::Render;
            }
            if self.memory.audio_device.sound_queue_push_requested {
                self.memory.audio_device.sound_queue_push_requested = false;
                return FrontendEvent::QueueSound;
            }
        }
    }

    /// Register a keypress from UI
    pub fn press_key(&mut self, key : KeyPress) {
        self.memory.joypad.press_key(key);
    }

    pub fn clear_key(&mut self, key: KeyPress) {
        self.memory.joypad.clear_key(key);
    }

    pub fn load_rom_from_vec(&mut self, vec: &Vec<u8>) {
        self.memory.rom.load_from_data(vec);
    }

    pub fn get_sound_queue(&mut self) -> &Vec<f32> {
        return &self.memory.audio_device.sample_queue;
    }
}
    