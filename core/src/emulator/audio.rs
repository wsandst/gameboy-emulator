// Temporary
#![allow(dead_code)]
/// Represents the Gameboy Audio Device/APU
/// 
/// The Gameboy Audio Device has 4 channels in total.
/// 2 Square Wave channels, 1 Pulse Wave channel and
/// 1 Noise channel.
/// Sample every 87 clock cycles  ~= 22 M-cycles
/// Then every 1024 samples, output to queue

const CYCLES_PER_SAMPLE: usize = 87;
const SAMPLES_PER_PUSH: usize = 1024;

const CLOCK_RATE : usize = 4194304;
const DEFAULT_SAMPLE_RATE : usize = 48000;
const BLIP_BUFFER_SIZE : u32 = (DEFAULT_SAMPLE_RATE / 5) as u32;

const ENABLE_SQUARE_CHANNEL1 : bool = true;
const ENABLE_SQUARE_CHANNEL2 : bool = true;
const ENABLE_WAVE_CHANNEL : bool = true;
const ENABLE_NOISE_CHANNEL : bool = true;


mod sample_buf;
mod square_channel;
mod noise_channel;
mod wave_channel;
mod volume_envelope;

use modular_bitfield::prelude::*;

use std::convert::TryInto;
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

/* 
NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
NR51 FF25 NW21 NW21 Left enables, Right enables
NR52 FF26 P--- NW21 Power control/status, Channel length statuses
*/
#[bitfield]
#[derive(Debug, Serialize, Deserialize)]
struct ControlOptions {
    // 0xFF24
    right_vol: B3,
    vin_right_enable: bool,
    left_vol: B3,
    vin_left_enable: bool,
    // 0xFF25
    right_pulse_channel1_enable: bool, // 1
    right_pulse_channel2_enable: bool, // 2
    right_wave_channel_enable: bool, // W
    right_noise_channel_enable: bool, // N
    left_pulse_channel1_enable: bool, // 1
    left_pulse_channel2_enable: bool, // 2
    left_wave_channel_enable: bool, // W
    left_noise_channel_enable: bool, // N
    // 0xFF26
    length_pulse_channel1_status: bool, // 1
    length_pulse_channel2_status: bool, // 2
    length_wave_channel_status: bool, // W
    length_noise_channel_status: bool, // N
    #[skip] __: B3,
    power_status: bool,
    // 0xFF
}

#[derive(Serialize, Deserialize)]
pub struct AudioDevice {
    #[serde(with = "BigArray")]
    memory: [u8; 48],
    options: ControlOptions,
    square_channel1 : square_channel::SquareChannel,
    square_channel2 : square_channel::SquareChannel,
    wave_channel : wave_channel::WaveChannel,
    noise_channel : noise_channel::NoiseChannel,

    clock_cycles: usize,
    length_step_counter: usize,
    vol_step_counter: usize,
    sweep_step_counter: usize,

    gen_rate: usize,
    pub sound_queue_push_requested: bool,
    sample_queue: Vec<f32>,
    sample_count: usize,
    sample_rate: usize,
}

impl AudioDevice {
    pub fn new() -> AudioDevice {
        let mut device = AudioDevice { 
            memory: [0; 48], 
            options: ControlOptions::new(),
            square_channel1 : square_channel::SquareChannel::new(true),
            square_channel2 : square_channel::SquareChannel::new(false),
            wave_channel: wave_channel::WaveChannel::new(),
            noise_channel: noise_channel::NoiseChannel::new(),
            clock_cycles: 0,
            length_step_counter: 0,
            vol_step_counter: 0,
            sweep_step_counter: 0,

            gen_rate: 0,
            sound_queue_push_requested: false,
            sample_queue: vec![0 as f32; SAMPLES_PER_PUSH*2],
            sample_count: 0,
            sample_rate: DEFAULT_SAMPLE_RATE,
        };
        device.update_output_samplerate();
        return device;
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        return match address {
            0xFF26 => {
                self.options.bytes[2]
            }
            _ => self.memory[address - 0xFF10]
        }
    }

    fn update_channel_enables(&mut self) {
        self.options.set_length_pulse_channel1_status(self.square_channel1.enabled);
        self.options.set_length_pulse_channel2_status(self.square_channel2.enabled);
        self.options.set_length_wave_channel_status(self.wave_channel.enabled);
        self.options.set_length_noise_channel_status(self.noise_channel.enabled);
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        if !self.options.power_status() && address != 0xFF26 {
            return;
        }
        self.memory[address - 0xFF10] = val;

        match address {
            0xFF10 ..= 0xFF14 => { self.square_channel1.update_options(val, address-0xFF10) },
            // 0xFF15 is not used for the second square channel
            0xFF16 ..= 0xFF19 => { self.square_channel2.update_options(val, address-0xFF15) },
            0xFF1A ..= 0xFF1E => { self.wave_channel.update_options(val, address-0xFF1A) },
            0xFF20 ..= 0xFF23 => { self.noise_channel.update_options(val, address-0xFF1F) },
            0xFF24 ..= 0xFF26 => { self.update_options() },
            0xFF30 ..= 0xFF3F => { self.wave_channel.write_wave_ram(address, val) }
            _ => {}
        }
    }

    pub fn update_options(&mut self) {
        self.options = ControlOptions::from_bytes(self.memory[20..23].try_into().unwrap());
    }

    pub fn cycle(&mut self, cycles : usize) {
        if !self.options.power_status() {
            return;
        }
        self.clock_cycles += cycles;
        self.length_step_counter += cycles;
        self.vol_step_counter += cycles;
        self.sweep_step_counter += cycles;
        // Step the channel lengths, 256 hz
        if self.length_step_counter >= (CLOCK_RATE / 256) {
            self.square_channel1.step_length();
            self.square_channel2.step_length();
            self.wave_channel.step_length();
            self.noise_channel.step_length();
            self.update_channel_enables();
            self.length_step_counter -= CLOCK_RATE / 256;
        }
        // Steep sweep at 256 hz
        if self.sweep_step_counter >= (CLOCK_RATE / 128) {
            self.square_channel1.step_sweep();
            self.sweep_step_counter -= CLOCK_RATE / 128;
        }
        // Step the volume envelopes, 64 hz
        if self.vol_step_counter >= (CLOCK_RATE / 64) {
            self.square_channel1.step_volume();
            self.square_channel2.step_volume();
            self.noise_channel.step_volume();
            self.vol_step_counter -= CLOCK_RATE / 64;
        }
        // Generate 1024 samples for output every GEN_RATE cycles
        if self.clock_cycles > self.gen_rate {
            self.generate_samples(self.gen_rate);
            self.clock_cycles -= self.gen_rate;
            self.mix_samples();
            self.sound_queue_push_requested = true;
        }
    }

    pub fn generate_samples(&mut self, sample_count: usize) {
        // Run blipbufs
        self.square_channel1.sample(sample_count);
        self.square_channel2.sample(sample_count);
        self.wave_channel.sample(sample_count);
        self.noise_channel.sample(sample_count);
        
        self.square_channel1.blipbuf.end_frame((sample_count) as u32);
        self.square_channel2.blipbuf.end_frame((sample_count) as u32);
        self.wave_channel.blipbuf.end_frame((sample_count) as u32);
        self.noise_channel.blipbuf.end_frame((sample_count) as u32);
    }

    /// Get 1024 samples from channel blipbufs and mix them
    fn mix_samples(&mut self) {
        let sample_count_mono = self.square_channel1.generate_output_buffer();
        self.square_channel2.generate_output_buffer();
        self.wave_channel.generate_output_buffer();
        self.noise_channel.generate_output_buffer();
        self.sample_count = sample_count_mono*2;

        let left_vol = (self.options.left_vol() as f32 / 7.0) * (1.0 / 15.0) * 0.25;
        let right_vol = (self.options.right_vol() as f32 / 7.0) * (1.0 / 15.0) * 0.25;
        // Keep bools immutable, this might improve performance of loop?
        let left_pulse_channel1_enable = self.options.left_pulse_channel1_enable();
        let right_pulse_channel1_enable = self.options.right_pulse_channel1_enable();
        let left_pulse_channel2_enable = self.options.left_pulse_channel2_enable();
        let right_pulse_channel2_enable = self.options.right_pulse_channel2_enable();
        let left_wave_channel_enable = self.options.left_wave_channel_enable();
        let right_wave_channel_enable = self.options.right_wave_channel_enable();
        let left_noise_channel_enable = self.options.left_noise_channel_enable();
        let right_noise_channel_enable = self.options.right_noise_channel_enable();
        let mut left_sample : f32 = 0.0;
        let mut right_sample : f32 = 0.0;
        for i in 0..sample_count_mono {
            // Pulse channel 1
            if left_pulse_channel1_enable && ENABLE_SQUARE_CHANNEL1 {
                left_sample += self.square_channel1.sample_buf[i] as f32;
            }
            if right_pulse_channel1_enable && ENABLE_SQUARE_CHANNEL1 {
                right_sample += self.square_channel1.sample_buf[i] as f32;
            }
            // Pulse channel 2
            if left_pulse_channel2_enable && ENABLE_SQUARE_CHANNEL2 {
                left_sample += self.square_channel2.sample_buf[i] as f32;
            }
            if right_pulse_channel2_enable && ENABLE_SQUARE_CHANNEL2 {
                right_sample += self.square_channel2.sample_buf[i] as f32;
            }
            // Wave channel
            if left_wave_channel_enable && ENABLE_WAVE_CHANNEL {
                left_sample += self.wave_channel.sample_buf[i] as f32;
            }
            if right_wave_channel_enable && ENABLE_WAVE_CHANNEL {
                right_sample += self.wave_channel.sample_buf[i] as f32;
            }
            // Noise channel
            if left_noise_channel_enable && ENABLE_NOISE_CHANNEL {
                left_sample += self.noise_channel.sample_buf[i] as f32;
            }
            if right_noise_channel_enable && ENABLE_NOISE_CHANNEL {
                right_sample += self.noise_channel.sample_buf[i] as f32;
            }
            self.sample_queue[i*2+0] = left_sample * left_vol;
            self.sample_queue[i*2+1] = right_sample * right_vol;
            left_sample = 0.0;
            right_sample = 0.0;
        }
        //self.clear_blipbufs();
    }

    pub fn get_sample_queue(&self) -> &[f32] {
        return &self.sample_queue[0..self.sample_count];
    }

    fn set_blipbuf_sample_rates(&mut self, sample_rate: usize) {
        self.square_channel1.blipbuf.set_rates(CLOCK_RATE as f64, sample_rate as f64);
        self.square_channel2.blipbuf.set_rates(CLOCK_RATE as f64, sample_rate as f64);
        self.wave_channel.blipbuf.set_rates(CLOCK_RATE as f64, sample_rate as f64);
        self.noise_channel.blipbuf.set_rates(CLOCK_RATE as f64, sample_rate as f64);
    }

    fn clear_blipbufs(&mut self) {
        self.square_channel1.blipbuf.clear();
        self.square_channel2.blipbuf.clear();
        self.wave_channel.blipbuf.clear();
        self.noise_channel.blipbuf.clear();
    }

    pub fn set_output_samplerate(&mut self, sample_rate: usize) {
        if sample_rate != self.sample_rate {
            self.sample_rate = sample_rate;
            self.update_output_samplerate();
        }
    }

    /// Modify the output sample rate
    /// This is only allowed between audio frames
    pub fn update_output_samplerate(&mut self) {
        self.gen_rate = ((CLOCK_RATE as u64 * SAMPLES_PER_PUSH as u64) / self.sample_rate as u64) as usize;
        self.set_blipbuf_sample_rates(self.sample_rate);
    }
}


