/// Represents the Noise Channel in the Gameboy Audio Hardware
/// Plays white noise using a pseudo-random LFSR number generation

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
use super::volume_envelope::VolumeEnvelope;

const SAMPLES_PER_PUSH: usize = 1024;

const CLOCK_RATE : usize = 4194304;
const DEFAULT_SAMPLE_RATE : usize = 48000;
const BLIP_BUFFER_SIZE : u32 = (DEFAULT_SAMPLE_RATE / 5) as u32;

#[bitfield]
#[derive(Serialize, Deserialize)]
struct NoiseOptions {
    // 0xFF1F
    #[skip] __: u8,
    // 0xFF20
    length_load: B6,
    #[skip] __: B2,
    // 0xFF21
    envelope_period: B3,
    envelope_mode: bool,
    envelope_starting_vol: B4,
    // 0xFF22
    divisor_code: B3,
    width_mode: bool,
    clock_shift: B4,
    // 0xFF23
    #[skip] __: B6,
    length_enable: bool,
    trigger: B1,
}

#[derive(Serialize, Deserialize)]
pub struct NoiseChannel {
    options: NoiseOptions,
    #[serde(with = "BigArray")]
    pub sample_buf: [i16; SAMPLES_PER_PUSH],
    #[serde(default = "serde_blipbuf_default")]
    #[serde(skip)]
    pub blipbuf : BlipBuf,
    pub enabled: bool,
    delay: usize,
    last_amp: i32,
    volume_envelope: VolumeEnvelope,
    length: usize,
    lfsr: u16,
}

impl NoiseChannel {
    pub fn new() -> NoiseChannel {
        return NoiseChannel { 
            options : NoiseOptions::new(),
            blipbuf : BlipBuf::new(BLIP_BUFFER_SIZE),
            sample_buf: [0; SAMPLES_PER_PUSH],
            enabled: false,
            delay: 0,
            last_amp: 0,
            length: 0,
            volume_envelope: VolumeEnvelope::new(),
            lfsr: 0xFF,
        }
    }

    pub fn update_options(&mut self, byte : u8, index : usize) {
        self.options.bytes[index] = byte;
        if index == 4 && (byte & 0b1000_0000 != 0) {
            self.length = 64 - self.options.length_load() as usize;
            self.enabled = true;

            self.volume_envelope.delay = self.options.envelope_period();
            self.volume_envelope.volume = self.options.envelope_starting_vol();

            self.lfsr = 0xFF;
            self.delay = 0;
        }
        // Length load
        else if index == 1 {
            self.length = 64 - self.options.length_load() as usize;
        }
    }

    pub fn calculate_period(&mut self) -> usize {
        let divisor = match self.options.divisor_code() {
            0 => 8,
            n => n * 16
        };
        let period = (divisor as usize) << (self.options.clock_shift() as usize);
        return period;
    }

    /// Peform 1 bit Linear Feedback Shift Register Random generation
    pub fn rng_lfsr(&mut self) {
        let lfsr_xored = (self.lfsr ^ (self.lfsr >> 1)) & 1;
        let lfsr_shifted = self.lfsr >> 1;
        let mut result = lfsr_shifted | (lfsr_xored << 14);
        if self.options.width_mode() {
            result = (lfsr_shifted & !(1 << 6)) | (lfsr_xored << 6);
        } 
        self.lfsr = result;
    }

    pub fn sample(&mut self, cycles: usize) {
        let period = self.calculate_period();
        // Set amp to 0 if disabled
        if !self.enabled || period == 0 || self.volume_envelope.volume == 0 {
            if self.last_amp != 0 {
                self.blipbuf.add_delta(0, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        }
        else {
            let mut time = self.delay;

            while time < cycles {
                self.rng_lfsr();
                let mut amp = match self.options.width_mode() {
                    true => !(self.lfsr >> 6) & 1,
                    false => !(self.lfsr >> 14) & 1,
                } as i32;

                amp = match amp {
                    0 => 0,
                    _ => 1,
                } * self.volume_envelope.volume as i32;

                if amp != self.last_amp {
                    self.blipbuf.add_delta(time as u32, amp - self.last_amp);
                    self.last_amp = amp;
                }
                time += period;
            }
            self.delay = time - cycles;
        }
    }

    // Step at 256 hz
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
    }

    // Step at 64 hz
    pub fn step_volume(&mut self) {
        self.volume_envelope.step(
            self.options.envelope_period(), 
            self.options.envelope_mode()
        );
    }

    pub fn generate_output_buffer(&mut self) -> usize {
        return self.blipbuf.read_samples(&mut self.sample_buf, false) as usize;
    }
}

fn serde_blipbuf_default() -> BlipBuf {
    let mut buf = BlipBuf::new(BLIP_BUFFER_SIZE);
    buf.set_rates(CLOCK_RATE as f64, DEFAULT_SAMPLE_RATE as f64);
    return buf;
}