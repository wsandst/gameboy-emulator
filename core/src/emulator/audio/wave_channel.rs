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
    // 0xFF1D
    frequency_lsb: u8,
    // 0xFF1E
    frequency_msb: B3,
    #[skip] __: B3,
    length_enable: B1,
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

fn serde_blipbuf_default() -> BlipBuf {
    let mut buf = BlipBuf::new(BLIP_BUFFER_SIZE);
    buf.set_rates(CLOCK_RATE as f64, DEFAULT_SAMPLE_RATE as f64);
    return buf;
}