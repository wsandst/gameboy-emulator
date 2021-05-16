mod cpu;
mod memory;
mod rom;
mod timer;
mod interrupts;
mod gpu;
mod screen;
mod joypad;
mod audio;

use serde::{Serialize, Deserialize};
use flate2::write::ZlibEncoder;
use flate2::write::ZlibDecoder;
use std::io::Write;

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

#[derive(Serialize, Deserialize)]
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

    pub fn get_rom_name(&mut self) -> &str {
        let romname = self.memory.rom.filename.split("/").last().unwrap();
        return &romname[..romname.len()-3];
    }

    /// Serialize the entire emulator using bincode
    /// DrawHelper and BlipBuf state is not saved
    pub fn serialize(&mut self) -> Vec<u8> {
        // Serialize using serde bincode format
        let serialized_bytes = bincode::serialize(&self).unwrap();
        // Compress using flate2
        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(&serialized_bytes).unwrap();
        let compressed_bytes = encoder.finish().unwrap();
        return compressed_bytes;
    }

    /// Deserialize a compressed serde bincode save file into a new emulator
    pub fn deserialize(bytes: &Vec<u8>) -> Emulator {
        // Decompress
        let mut decoder = ZlibDecoder::new(Vec::<u8>::new());
        decoder.write_all(&bytes).unwrap();
        let bincode_bytes = decoder.finish().unwrap();
        // Deserialize
        let mut em : Emulator = bincode::deserialize(&bincode_bytes).unwrap();
        em.memory.gpu.init_draw_helper();
        return em;
    }
}
    