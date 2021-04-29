// Temporary
#![allow(dead_code)]
/// Represents the Gameboy Audio Device/APU
/// 
/// The Gameboy Audio Device has 4 channels in total.
/// 2 Square Wave channels, 1 Pulse Wave channel and
/// 1 Noise channel.
/// Sample every 87 clock cycles  ~= 22 M-cycles
/// Then every 1024 samples, output to queue



use modular_bitfield::prelude::*;
use std::convert::TryInto;

#[bitfield]
struct SquareOptions {
    // 0xFF10
    sweep_shift: B3,
    sweep_negate: B1,
    sweep_time: B3,
    #[skip] __: B1,
    // 0xFF11, 0xFF16
    length: B6,
    duty: B2,
    // 0xFF12, 0xFF17
    period: B3,
    envelope_mode: B1,
    envelope_counter: B4,
    // 0xFF13, 0xFF18
    frequency_lsb: u8,
    // 0xFF14, 0xFF19
     frequency_msb: B3,
    #[skip] __: B3,
    length_enable: B1,
    trigger: B1,
}

struct SquareChannel {
    options: SquareOptions,
}

impl SquareChannel {
    pub fn new() -> SquareChannel {
        return SquareChannel { options : SquareOptions::new()}
    }

    pub fn update_options(&mut self, mem : [u8; 5]) {
        self.options = SquareOptions::from_bytes(mem);
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
}

#[allow(dead_code)]
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
}

pub struct AudioDevice {
    memory: [u8; 48],
    square_channel1 : SquareChannel,
    square_channel2 : SquareChannel,
    wave_channel : WaveChannel,
    noise_channel : NoiseChannel,
    pub sound_queue_push_requested: bool,
    pub sound_queue: Vec<i16>,
}

pub fn gen_wave(bytes_to_write: i32) -> Vec<i16> {
    // Generate a square wave
    let tone_volume = 1_000i16;
    let period = 48_000 / 256;
    let sample_count = bytes_to_write;
    let mut result = Vec::new();

    for x in 0..sample_count {
        result.push(if (x / period) % 2 == 0 {
            tone_volume
        } else {
            -tone_volume
        });
    }
    result
}

impl AudioDevice {
    pub fn new() -> AudioDevice {
        AudioDevice { 
            memory: [0; 48], 
            square_channel1 : SquareChannel::new(),
            square_channel2 : SquareChannel::new(),
            wave_channel: WaveChannel::new(),
            noise_channel: NoiseChannel::new(),
            sound_queue_push_requested: false,
            sound_queue: vec![0; 1024],
        }
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        return self.memory[address - 0xFF10];
        
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        self.memory[address - 0xFF10] = val;
        self.sound_queue = gen_wave(1000);
        self.sound_queue_push_requested = true;

        match address {
            0xFF10 ..= 0xFF14 => { self.square_channel1.update_options(self.memory[0..5].try_into().unwrap()) },
            // 0xFF15 is not used for the second square channel
            0xFF16 ..= 0xFF19 => { self.square_channel2.update_options(self.memory[5..10].try_into().unwrap()) },
            0xFF1A ..= 0xFF1E => { self.wave_channel.update_options(self.memory[10..15].try_into().unwrap()) },
            0xFF20 ..= 0xFF23 => { self.noise_channel.update_options(self.memory[15..20].try_into().unwrap()) },
            _ => {}
        }
    }

    pub fn cycle(&mut self) {

    }
}




