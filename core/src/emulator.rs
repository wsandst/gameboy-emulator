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
    pub paused: bool,
    pub prev_sound_frame_cycles: u64,
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
            paused: false,
            prev_sound_frame_cycles: 0,
        };
        if use_bootrom {
            em.cpu.regs.pc = 0;
            em.memory.rom.using_boot_rom = true;
        }
        return em;
    }

    pub fn run(&mut self, steps : u32) {
        for _i in 1..steps {
            self.step();
        }
    }

    pub fn enable_bootrom(&mut self) {
        self.cpu.regs.pc = 0;
        self.memory.rom.using_boot_rom = true;
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
                if self.memory.gpu.state_modified_last_frame || self.memory.gpu.state_modified {
                     // No point in drawing if nothing has changed
                    self.screen.draw_line(&self.memory.gpu); 
                }
                self.memory.gpu.scanline_draw_requested = false;
            }
            if self.memory.gpu.screen_draw_requested {
                self.memory.gpu.state_modified_last_frame = self.memory.gpu.state_modified;
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

    pub fn load_rom_from_data(&mut self, vec: &Vec<u8>) {
        self.memory.rom.load_from_data(vec);
    }

    pub fn load_bootrom_from_data(&mut self, vec: &Vec<u8>) {
        self.memory.rom.load_bootrom_from_data(vec);
    }

    pub fn get_sound_queue(&mut self) -> &[f32] {
        return &self.memory.audio_device.get_sample_queue();
    }

    /// Modify the output samplerate of the emulator
    /// This is always treated as 48000 hz, so increasing it
    /// will lead to a sound speedup/slowdown, which is can be used
    /// for hopefully unnoticeable video/sound syncing
    pub fn set_sound_output_sample_rate(&mut self, sample_rate: usize) {
        self.memory.audio_device.set_output_samplerate(sample_rate);
    }

    pub fn get_rom_name(&mut self) -> &str {
        return &self.memory.rom.romname;
    }

    pub fn set_rom_name(&mut self, romname : &str) {
        self.memory.rom.romname = romname.to_owned();
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
    
#[cfg(test)]
mod test
{
    // Test serialization and deserialization using serde
    use super::Emulator;
    
    #[test]
    fn serialization()
    {
        let mut em1 = Emulator::new(false);
        em1.memory.output_serial_to_stdout = false;
        em1.memory.rom.load_from_file("../roms/blargg/cpu_instrs.gb");

        // Run emulator for a while
        for _ in 0..300 {
            em1.run_until_frontend_event();
        }

        // Serialize emulator
        let serialized_bytes = em1.serialize();
        // Deserialize into second emulator
        let mut em2 = Emulator::deserialize(&serialized_bytes);

        // Run both for a few frames
        for _ in 0..20 {
            em1.run_until_frontend_event();
            em2.run_until_frontend_event();
        }

        // The emulator states should be identical, verify this
        // Verify Gameboy internal memory
        for addr in 0..0xFFFF {
            if em1.memory.read_byte(addr) != em2.memory.read_byte(addr) {
                assert!(false, "Deserialized emulator memory mismatch at addr {}", addr);
            }
        }
        // Verify bitmap
        for i in 0..160*144*3 {
            if em1.screen.bitmap[i] != em2.screen.bitmap[i] {
                assert!(false, "Deserialized emulator bitmap mismatch");
            }
        }
    }
}