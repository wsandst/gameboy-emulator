mod cpu;
mod memory;
mod rom;
mod timer;
mod interrupts;
mod gpu;
mod screen;
mod joypad;
mod audio;

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
}

impl Emulator
{
    pub fn new() -> Emulator
    {
        Emulator {cpu : cpu::CPU::new(), memory: memory::Memory::new(), screen: screen::Screen::new(), frame_counter: 0}
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
        self.memory.joypad.clear_all_keys();
    }

    pub fn run_until_frontend_event(&mut self) -> FrontendEvent {
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
                self.memory.joypad.clear_all_keys();
                return FrontendEvent::Render;
            }
            if self.memory.audio_device.sound_queue_push_requested {
                return FrontendEvent::QueueSound;
            }
        }
    }

    /// Register a keypress from UI
    pub fn press_key(&mut self, key : KeyPress) {
        self.memory.joypad.press_key(key);
    }

    pub fn load_rom_from_vec(&mut self, vec: &Vec<u8>) {
        self.memory.rom.load_from_data(vec);
    }

    pub fn get_sound_queue(&mut self) -> &Vec<i16> {
        return &self.memory.audio_device.sound_queue;
    }
}
    