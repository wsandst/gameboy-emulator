/// Represents the two Square/Pulse Channel in the Gameboy Audio Hardware
/// Plays a rectangular wave


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
#[derive(Debug, Serialize, Deserialize)]
struct PulseOptions {
    // 0xFF10
    sweep_shift: B3,
    sweep_negate: bool,
    sweep_period: B3,
    #[skip] __: B1,
    // 0xFF11, 0xFF16
    // How long should we play? Decrement this, stop playing when 0
    // Only decrement and stop playing if length_enable is on
    length_load: B6,
    // The duty of the rectangle wave, how many % should stay over the zero line (1/2 for square wave etc)
    duty: B2,
    // 0xFF12, 0xFF17
    // Period of wave
    envelope_period: B3,
    envelope_mode: bool,
    envelope_starting_vol: B4,
    // 0xFF13, 0xFF18
    // In hertz?
    frequency: B11,
    // 0xFF14, 0xFF19
    //frequency_msb: B3,
    #[skip] __: B3,
    length_enable: bool,
    trigger: B1,
}

const DUTY0: [i32; 8] = [0,0,0,0,0,0,0,1]; // 12.5 %
const DUTY1: [i32; 8] = [1,0,0,0,0,0,0,1]; // 25 %
const DUTY2: [i32; 8] = [1,0,0,0,0,1,1,1]; // 50 %
const DUTY3: [i32; 8] = [0,1,1,1,1,1,1,0]; // 75 %

const DUTY_OPTIONS: [[i32; 8]; 4] = [DUTY0, DUTY1, DUTY2, DUTY3]; 

/// Square/Pulse wave channel
#[derive(Serialize, Deserialize)]
pub struct SquareChannel {
    options: PulseOptions,
    #[serde(with = "BigArray")]
    pub sample_buf: [i16; SAMPLES_PER_PUSH],

    #[serde(default = "serde_blipbuf_default")]
    #[serde(skip)]
    pub blipbuf : BlipBuf,

    duty_index: usize,
    last_amp: i32,
    enabled: bool,
    delay: usize,
    volume_envelope: VolumeEnvelope,
    length: usize,

    sweep: bool,
    sweep_delay: usize,
    sweep_frequency: usize,

    has_triggered: bool,
}

impl SquareChannel {
    pub fn new(sweep: bool) -> SquareChannel {
        return SquareChannel { 
            options : PulseOptions::new(), 
            duty_index: 0, 
            blipbuf : BlipBuf::new(BLIP_BUFFER_SIZE*2),
            sample_buf: [0; SAMPLES_PER_PUSH],
            last_amp: 0,
            delay: 0,
            enabled: false,
            volume_envelope: VolumeEnvelope::new(),
            length: 0,
            sweep: sweep,
            sweep_delay: 0,
            sweep_frequency: 0,
            has_triggered: false,
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
            self.length = 63 - self.options.length_load() as usize;
        }
        // Update volume envelope
        else if index == 2 {
            self.volume_envelope.delay = self.options.envelope_period();
            self.volume_envelope.volume = self.options.envelope_starting_vol();
        }
    }

    pub fn trigger(&mut self) {
        self.length = 63 - self.options.length_load() as usize;
        self.enabled = true;
        self.volume_envelope.delay = self.options.envelope_period();
        self.volume_envelope.volume = self.options.envelope_starting_vol();

        self.sweep_frequency = self.options.frequency() as usize;
        if self.sweep && self.options.sweep_period() > 0 && self.options.sweep_shift() > 0 {
            self.sweep_delay = 1;
            self.step_sweep();
        }
        self.has_triggered = true;
    }

    pub fn calculate_period(&self) -> usize {
        if self.options.frequency() > 2048 { 
            return 0; 
        }
        else {
            return ((2048 - self.options.frequency() as u32)*4) as usize;
        }
    }

    pub fn sample(&mut self, cycles: usize, channel_enable: bool) {
        let period = self.calculate_period();

        // Set amp to 0 if disabled
        if !self.enabled || !channel_enable || period == 0 || self.volume_envelope.volume == 0 || !self.has_triggered {
            if self.last_amp != 0 {
                self.blipbuf.add_delta(0, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        }
        else {
            let mut time = self.delay;

            while time < cycles {
                let amp = DUTY_OPTIONS[self.options.duty() as usize][self.duty_index] * self.volume_envelope.volume as i32;
                if amp != self.last_amp {
                    self.blipbuf.add_delta(time as u32, amp - self.last_amp);
                    self.last_amp = amp;
                }
                time += period;
                self.duty_index = (self.duty_index + 1) % 8;
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
        else {
            self.enabled = true;
        }
    }

    // Step at 64 hz
    pub fn step_volume(&mut self) {
        self.volume_envelope.step(
            self.options.envelope_period(), 
            self.options.envelope_mode()
        );
    }

    // Step at 128 hz
    pub fn step_sweep(&mut self) {
        if self.options.sweep_period() == 0 {
            return;
        }
        if self.sweep_delay > 1 {
            self.sweep_delay -= 1;
        }
        else {
            self.sweep_delay = self.options.sweep_period() as usize;
            if self.sweep_frequency == 2048 {
                self.enabled = false;
                self.options.set_frequency(2047);
            }
            else {
                self.options.set_frequency(self.sweep_frequency as u16);
            }

            let offset = self.sweep_frequency >> self.options.sweep_shift();

            if self.options.sweep_negate() {
                // F ~ (2048 - f)
                // Increase in frequency means subtracting the offset
                if self.sweep_frequency <= offset {
                    self.sweep_frequency = 0;
                }
                else {
                    self.sweep_frequency -= offset;
                }
            }
            else {
                if self.sweep_frequency >= 2048 - offset {
                    self.sweep_frequency = 2048;
                }
                else {
                    self.sweep_frequency += offset;
                }
            }
        }
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