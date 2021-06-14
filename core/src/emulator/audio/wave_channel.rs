/// Represents the Wave Channel in the Gameboy Audio Hardware
/// This channel can play digital sounds from 32 bytes of
/// wave ram

// Conditionally use BlipBuf or SampleBuf
// BlipBuf gives better sound quality but does not work with WASM,
// so SampleBuf is used for WASM fallback
#[cfg(not(target_arch = "wasm32"))]
use blip_buf::BlipBuf;

#[cfg(target_arch = "wasm32")]
type BlipBuf = sample_buf::SampleBuf;

use modular_bitfield::prelude::*;

use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

const SAMPLES_PER_PUSH: usize = 1024;

const CLOCK_RATE : usize = 4194304;
const DEFAULT_SAMPLE_RATE : usize = 48000;
const BLIP_BUFFER_SIZE : u32 = (DEFAULT_SAMPLE_RATE / 5) as u32;

#[bitfield]
#[derive(Serialize, Deserialize)]
struct WaveOptions {
    // 0xFF1A
    #[skip] __: B7,
    dac_power: B1,
    // 0xFF1B
    length_load: B8,
    // 0xFF1C
    #[skip] __: B5,
    volume_code: B2,
    #[skip] __: B1,
    // 0xFF1D && 0xFF1E
    frequency: B11,
    #[skip] __: B3,
    length_enable: bool,
    trigger: B1,
}

#[derive(Serialize, Deserialize)]
pub struct WaveChannel {
    options: WaveOptions,
    #[serde(with = "BigArray")]
    pub sample_buf: [i16; SAMPLES_PER_PUSH],
    #[serde(default = "serde_blipbuf_default")]
    #[serde(skip)]
    pub blipbuf : BlipBuf,
    wave_ram: [u8; 32],
    pub enabled: bool,
    length: usize,

    delay: usize,
    last_amp: i32,
    wave_index: usize,
}

impl WaveChannel {
    pub fn new() -> WaveChannel {
        return WaveChannel { 
            options : WaveOptions::new(),
            blipbuf : BlipBuf::new(BLIP_BUFFER_SIZE),
            sample_buf: [0; SAMPLES_PER_PUSH],
            wave_ram: [0; 32],
            enabled: false,
            length: 0,

            last_amp: 0,
            delay: 0,
            wave_index: 0,
        }
    }

    pub fn update_options(&mut self, byte : u8, index : usize) {
        self.options.bytes[index] = byte;
        // Trigger
        if index == 4 && (byte & 0b1000_0000 != 0) {
            self.trigger();
        }
        // Length load
        else if index == 1 {
            self.length = 256 - self.options.length_load() as usize;
        }
    }

    pub fn write_wave_ram(&mut self, addr: usize, val: u8) {
        // Each value is two digits, separate them
        let new_addr = (addr - 0xFF30)*2;
        self.wave_ram[new_addr+0] = val >> 4;
        self.wave_ram[new_addr+1] = val & 0xF;
    }

    pub fn trigger(&mut self) {
        self.length = 256 - self.options.length_load() as usize;
        self.enabled = true;
        self.delay = 0;
    }

    pub fn sample(&mut self, cycles: usize) {
        let period = self.calculate_period();
        // Set amp to 0 if disabled
        if !self.enabled || period == 0 || self.options.volume_code() == 0 {
            if self.last_amp != 0 {
                self.blipbuf.add_delta(0, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        }
        else {
            let mut time = self.delay;
            while time < cycles {
                let amp = (self.wave_ram[self.wave_index] >> (self.options.volume_code() - 1)) as i32;
                if amp != self.last_amp {
                    self.blipbuf.add_delta(time as u32, amp - self.last_amp);
                    self.last_amp = amp;
                }
                time += period;
                self.wave_index = (self.wave_index + 1) % 32;
            }
            self.delay = time - cycles;
        }
    }

    pub fn generate_output_buffer(&mut self) -> usize {
        return self.blipbuf.read_samples(&mut self.sample_buf, false) as usize;
    }
    
    pub fn step_length(&mut self) {
        if self.options.length_enable() {
            if self.length == 0 {
                self.enabled = false;
            }
            else {
                self.enabled = true;
                self.length -= 1;
            }
        }
        else {
            self.enabled = true;
        }
    }

    pub fn calculate_period(&self) -> usize {
        if self.options.frequency() > 2048 { 
            return 0; 
        }
        else {
            return (2048 - self.options.frequency() as usize)*2;
        }
    }
}

fn serde_blipbuf_default() -> BlipBuf {
    let mut buf = BlipBuf::new(BLIP_BUFFER_SIZE);
    buf.set_rates(CLOCK_RATE as f64, DEFAULT_SAMPLE_RATE as f64);
    return buf;
}