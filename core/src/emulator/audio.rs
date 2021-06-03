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

mod sample_buf;

// Conditionally use BlipBuf or SampleBuf
// BlipBuf gives better sound quality but does not work with WASM,
// so SampleBuf is used for WASM fallback
#[cfg(not(target_arch = "wasm32"))]
use blip_buf::BlipBuf;

#[cfg(target_arch = "wasm32")]
type BlipBuf = sample_buf::SampleBuf;

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

/*
const DUTY0: [i32; 8] = [-1,-1,-1,-1, 1,-1,-1,-1]; // 12.5 %
const DUTY1: [i32; 8] = [-1,-1,-1,-1, 1, 1,-1,-1]; // 25 %
const DUTY2: [i32; 8] = [-1,-1, 1, 1, 1, 1,-1,-1]; // 50 %
const DUTY3: [i32; 8] = [ 1, 1, 1, 1,-1,-1, 1, 1]; // 75 %
*/

const DUTY0: [i32; 8] = [0,0,0,0,0,0,0,1]; // 12.5 %
const DUTY1: [i32; 8] = [1,0,0,0,0,0,0,1]; // 25 %
const DUTY2: [i32; 8] = [1,0,0,0,0,1,1,1]; // 50 %
const DUTY3: [i32; 8] = [0,1,1,1,1,1,1,0]; // 75 %

const DUTY_OPTIONS: [[i32; 8]; 4] = [DUTY0, DUTY1, DUTY2, DUTY3]; 

#[derive(Serialize, Deserialize)]
struct VolumeEnvelope {
    volume: u8,
    delay: u8,
}

impl VolumeEnvelope {
    pub fn new() -> VolumeEnvelope {
        return VolumeEnvelope { 
            volume: 0,
            delay: 0,
        }
    }

    pub fn step(&mut self, period: u8, mode: bool) {
        if self.delay > 1 {
            self.delay -= 1;
        }
        else if self.delay == 1 {
            self.delay = period;
            if mode && self.volume < 15 { // Increasing
                self.volume += 1;
            }
            else if !mode && self.volume > 0 { // Decreasing
                self.volume -= 1;
            }
        }
    }
}

/// Pulse wave channel (also known as rectangle/square wave)
#[derive(Serialize, Deserialize)]
struct PulseChannel {
    options: PulseOptions,
    #[serde(with = "BigArray")]
    sample_buf: [i16; SAMPLES_PER_PUSH],

    #[serde(default = "serde_blipbuf_default")]
    #[serde(skip)]
    blipbuf : BlipBuf,

    duty_index: usize,
    last_amp: i32,
    enabled: bool,
    delay: usize,
    volume_envelope: VolumeEnvelope,
    length: usize,

    sweep: bool,
    sweep_delay: usize,
    sweep_frequency: usize,
}

impl PulseChannel {
    pub fn new(sweep: bool) -> PulseChannel {
        return PulseChannel { 
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
        }
    }

    pub fn update_options(&mut self, byte : u8, index : usize) {
        self.options.bytes[index] = byte;
        // Trigger
        if index == 4 && (byte & 0b1000_0000 != 0) {
            self.length = 64 - self.options.length_load() as usize;
            self.volume_envelope.delay = self.options.envelope_period();
            self.volume_envelope.volume = self.options.envelope_starting_vol();
            self.enabled = true;

            self.sweep_frequency = self.options.frequency() as usize;
            if self.sweep && self.options.sweep_period() > 0 && self.options.sweep_shift() > 0 {
                self.sweep_delay = 1;
                self.step_sweep();
            }
        }
        // Length load
        else if index == 1 {
            self.length = 64 - self.options.length_load() as usize;
        }
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
        if !self.enabled || !channel_enable || period == 0 || self.volume_envelope.volume == 0 {
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
    // 0xFF1D
    frequency_lsb: u8,
    // 0xFF1E
    frequency_msb: B3,
    #[skip] __: B3,
    length_enable: B1,
    trigger: B1,
}

#[derive(Serialize, Deserialize)]
struct WaveChannel {
    options: WaveOptions,
    #[serde(with = "BigArray")]
    sample_buf: [i16; SAMPLES_PER_PUSH],
    #[serde(default = "serde_blipbuf_default")]
    #[serde(skip)]
    blipbuf : BlipBuf,
}

impl WaveChannel {
    pub fn new() -> WaveChannel {
        return WaveChannel { 
            options : WaveOptions::new(),
            blipbuf : BlipBuf::new(BLIP_BUFFER_SIZE),
            sample_buf: [0; SAMPLES_PER_PUSH],
        }
    }

    pub fn update_options(&mut self, byte : u8, index : usize) {
        self.options.bytes[index] = byte;
    }

    pub fn sample(&mut self) {
        
    }

    pub fn generate_output_buffer(&mut self) -> usize {
        return self.blipbuf.read_samples(&mut self.sample_buf, false) as usize;
    }
}

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
struct NoiseChannel {
    options: NoiseOptions,
    #[serde(with = "BigArray")]
    sample_buf: [i16; SAMPLES_PER_PUSH],
    #[serde(default = "serde_blipbuf_default")]
    #[serde(skip)]
    blipbuf : BlipBuf,
    enabled: bool,
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

    pub fn sample(&mut self, cycles: usize, channel_enable: bool) {
        let period = self.calculate_period();
        // Set amp to 0 if disabled
        if !self.enabled || !channel_enable || period == 0 || self.volume_envelope.volume == 0 {
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

    pub fn generate_output_buffer(&mut self) -> usize {
        return self.blipbuf.read_samples(&mut self.sample_buf, false) as usize;
    }
}

#[derive(Serialize, Deserialize)]
pub struct AudioDevice {
    #[serde(with = "BigArray")]
    memory: [u8; 48],
    options: ControlOptions,
    square_channel1 : PulseChannel,
    square_channel2 : PulseChannel,
    wave_channel : WaveChannel,
    noise_channel : NoiseChannel,

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
            square_channel1 : PulseChannel::new(true),
            square_channel2 : PulseChannel::new(false),
            wave_channel: WaveChannel::new(),
            noise_channel: NoiseChannel::new(),
            clock_cycles: 0,
            length_step_counter: 0,
            vol_step_counter: 0,
            sweep_step_counter: 0,

            gen_rate: 0,
            sound_queue_push_requested: false,
            sample_queue: vec![0 as f32; SAMPLES_PER_PUSH],
            sample_count: 0,
            sample_rate: DEFAULT_SAMPLE_RATE,
        };
        device.update_output_samplerate();
        return device;
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        return self.memory[address - 0xFF10];
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        self.memory[address - 0xFF10] = val;

        match address {
            0xFF10 ..= 0xFF14 => { self.square_channel1.update_options(val, address-0xFF10) },
            // 0xFF15 is not used for the second square channel
            0xFF16 ..= 0xFF19 => { self.square_channel2.update_options(val, address-0xFF15) },
            0xFF1A ..= 0xFF1E => { self.wave_channel.update_options(val, address-0xFF1A) },
            0xFF20 ..= 0xFF23 => { self.noise_channel.update_options(val, address-0xFF1F) },
            0xFF24 ..= 0xFF26 => { self.update_options() },
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
            //self.wave_channel.step_length();
            self.noise_channel.step_length();
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
        self.square_channel1.sample(sample_count, self.options.left_pulse_channel1_enable() || self.options.right_pulse_channel1_enable());
        self.square_channel2.sample(sample_count, self.options.left_pulse_channel2_enable() || self.options.right_pulse_channel2_enable());
        self.noise_channel.sample(sample_count, self.options.left_noise_channel_enable() || self.options.right_noise_channel_enable());
        self.square_channel1.blipbuf.end_frame((sample_count) as u32);
        self.square_channel2.blipbuf.end_frame((sample_count) as u32);
        self.noise_channel.blipbuf.end_frame((sample_count) as u32);
    }

    /// Get 1024 samples from channel blipbufs and mix them
    fn mix_samples(&mut self) {
        self.sample_count = self.square_channel1.generate_output_buffer();
        self.square_channel2.generate_output_buffer();
        //self.wave_channel.generate_output_buffer();
        self.noise_channel.generate_output_buffer();

        let mut sample : f32 = 0.0;
        for i in 0..self.sample_count {
            sample += self.square_channel1.sample_buf[i] as f32;
            sample += self.square_channel2.sample_buf[i] as f32;
            //sample += self.wave_channel.sample_buf[i];
            sample += self.noise_channel.sample_buf[i] as f32;
            sample = (sample * 0.10) / 3.0;
            self.sample_queue[i] = sample;
            sample = 0.0;
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

fn serde_blipbuf_default() -> BlipBuf {
    let mut buf = BlipBuf::new(BLIP_BUFFER_SIZE);
    buf.set_rates(CLOCK_RATE as f64, DEFAULT_SAMPLE_RATE as f64);
    return buf;
}


