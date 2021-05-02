// Temporary
#![allow(dead_code)]
/// Represents the Gameboy Audio Device/APU
/// 
/// The Gameboy Audio Device has 4 channels in total.
/// 2 Square Wave channels, 1 Pulse Wave channel and
/// 1 Noise channel.
/// Sample every 87 clock cycles  ~= 22 M-cycles
/// Then every 1024 samples, output to queue

const CYCLES_PER_SAMPLE: usize = 86;
const SAMPLES_PER_PUSH: usize = 2048;

use modular_bitfield::prelude::*;
use std::convert::TryInto;

/* 
NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
NR51 FF25 NW21 NW21 Left enables, Right enables
NR52 FF26 P--- NW21 Power control/status, Channel length statuses
*/
#[bitfield]
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
    right_noise_channel: bool, // N
    left_pulse_channel1_enable: bool, // 1
    left_pulse_channel2_enable: bool, // 2
    left_wave_channel_enable: bool, // W
    left_noise_channel: bool, // N
    // 0xFF26
    length_pulse_channel1_status: bool, // 1
    length_pulse_channel2_statuse: bool, // 2
    length_wave_channel_status: bool, // W
    length_noise_channel_status: bool, // N
    #[skip] __: B3,
    power_status: bool,
    // 0xFF
}

#[bitfield]
struct PulseOptions {
    // 0xFF10
    sweep_shift: B3,
    sweep_negate: B1,
    sweep_time: B3,
    #[skip] __: B1,
    // 0xFF11, 0xFF16
    // How long should we play? Decrement this, stop playing when 0
    // Only decrement and stop playing if length_enable is on
    length: B6,
    // The duty of the rectangle wave, how many % should stay over the zero line (1/2 for square wave etc)
    duty: B2,
    // 0xFF12, 0xFF17
    // Period of wave
    period: B3,
    envelope_mode: B1,
    envelope_counter: B4,
    // 0xFF13, 0xFF18
    // In hertz?
    frequency: B11,
    // 0xFF14, 0xFF19
    //frequency_msb: B3,
    #[skip] __: B3,
    length_enable: B1,
    trigger: B1,
}

const DUTY0: [i16; 8] = [0,0,0,0,0,0,0,1]; // 12.5 %
const DUTY1: [i16; 8] = [1,0,0,0,0,0,0,1]; // 25 %
const DUTY2: [i16; 8] = [1,0,0,0,0,1,1,1]; // 50 %
const DUTY3: [i16; 8] = [0,1,1,1,1,1,1,0]; // 75 %
const DUTY_OPTIONS: [[i16; 8]; 4] = [DUTY0, DUTY1, DUTY2, DUTY3]; 

struct PulseChannel {
    options: PulseOptions,
    
}

impl PulseChannel {
    pub fn new() -> PulseChannel {
        return PulseChannel { options : PulseOptions::new()}
    }

    pub fn update_options(&mut self, mem : [u8; 5]) {
        self.options = PulseOptions::from_bytes(mem);
    }

    pub fn sample(&mut self, x: usize) -> i16 {
        let freq = self.options.frequency();
        let period = 4194304 / (((2048-self.options.frequency())*8) as usize);
        //println!("{}", period);
        return DUTY_OPTIONS[self.options.duty() as usize][((x*CYCLES_PER_SAMPLE)/period) % 8] * 1_000;
    }
}

#[bitfield]
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

struct WaveChannel {
    options: WaveOptions,
}

impl WaveChannel {
    pub fn new() -> WaveChannel {
        return WaveChannel { options : WaveOptions::new()}
    }

    pub fn update_options(&mut self, mem : &[u8; 5]) {
        self.options = WaveOptions::from_bytes(*mem);
    }

    pub fn sample(&mut self) {
        
    }
}

#[bitfield]
struct NoiseOptions {
    // 0xFF1F
    #[skip] __: u8,
    // 0xFF20
    length: B6,
    #[skip] __: B2,
    // 0xFF21
    period: B3,
    envelope_mode: B1,
    envelope_counter: B4,
    // 0xFF22
    divisor_code: B3,
    width_mode: B1,
    clock_shift: B4,
    // 0xFF23
    #[skip] __: B6,
    length_enable: B1,
    trigger: B1,
}

struct NoiseChannel {
    options: NoiseOptions,
}

impl NoiseChannel {
    pub fn new() -> NoiseChannel {
        return NoiseChannel { options : NoiseOptions::new()}
    }

    pub fn update_options(&mut self, mem : &[u8; 5]) {
        self.options = NoiseOptions::from_bytes(*mem);
    }

    pub fn sample(&mut self) {

    }
}

pub struct AudioDevice {
    memory: [u8; 48],
    clock_cycles: usize,
    options: ControlOptions,
    square_channel1 : PulseChannel,
    square_channel2 : PulseChannel,
    wave_channel : WaveChannel,
    noise_channel : NoiseChannel,
    pub sound_queue_push_requested: bool,
    pub sample_queue: Vec<i16>,
    sample_index: usize,
    sample_count: usize,
}

impl AudioDevice {
    pub fn new() -> AudioDevice {
        AudioDevice { 
            memory: [0; 48], 
            clock_cycles: 0,
            options: ControlOptions::new(),
            square_channel1 : PulseChannel::new(),
            square_channel2 : PulseChannel::new(),
            wave_channel: WaveChannel::new(),
            noise_channel: NoiseChannel::new(),
            sound_queue_push_requested: false,
            sample_queue: vec![0; SAMPLES_PER_PUSH],
            sample_index: 0,
            sample_count: 0,
        }
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        return self.memory[address - 0xFF10];
        
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        self.memory[address - 0xFF10] = val;

        match address {
            0xFF10 ..= 0xFF14 => { self.square_channel1.update_options(self.memory[0..5].try_into().unwrap()) },
            // 0xFF15 is not used for the second square channel
            0xFF16 ..= 0xFF19 => { self.square_channel2.update_options(self.memory[5..10].try_into().unwrap()) },
            0xFF1A ..= 0xFF1E => { self.wave_channel.update_options(self.memory[10..15].try_into().unwrap()) },
            0xFF20 ..= 0xFF23 => { self.noise_channel.update_options(self.memory[15..20].try_into().unwrap()) },
            0xFF24 ..= 0xFF26 => { self.update_options() },
            _ => {}
        }
    }

    pub fn update_options(&mut self) {
        self.options = ControlOptions::from_bytes(self.memory[20..23].try_into().unwrap());
    }


    pub fn cycle(&mut self, cycles : usize) {
        self.clock_cycles += cycles;
        if self.clock_cycles > CYCLES_PER_SAMPLE { // Push a sample every 87 clock cycles
            self.clock_cycles -= CYCLES_PER_SAMPLE;
            self.generate_sample();
            if self.sample_index >= SAMPLES_PER_PUSH { // Push the sound every 1024 samples
                self.sound_queue_push_requested = true;
                self.sample_index = 0;
            }
        }
    }

    pub fn generate_sample(&mut self) {
        let mut samples : [i16; 4] = [0, 0, 0, 0];
        let mut sample_index : usize = 0;
        //println!("p: {}, rv: {}", self.options.power_status(), self.options.right_vol());
        if self.options.power_status() {
            if self.options.right_pulse_channel1_enable() || self.options.left_pulse_channel1_enable() {
                samples[sample_index] = self.square_channel1.sample(self.sample_count);
                sample_index += 1;
            }
            if self.options.right_pulse_channel2_enable() || self.options.left_pulse_channel2_enable() {
                samples[sample_index] = self.square_channel2.sample(self.sample_count);
                sample_index += 1;
            }
            if sample_index > 0 {
                self.sample_queue[self.sample_index] = samples.iter().sum::<i16>() / (sample_index as i16)
            }
            else {
                self.sample_queue[self.sample_index] = 0
            }
        }
        else { 
            self.sample_queue[self.sample_index] = 0;
        }
        self.sample_index += 1;
        self.sample_count += 1;
    }
}




