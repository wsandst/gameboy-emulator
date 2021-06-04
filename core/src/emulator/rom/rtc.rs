/// Represents a Real-Time Clock (RTC) for MBC3 cart

use modular_bitfield::prelude::*;
use serde::{Serialize, Deserialize};

#[bitfield]
#[derive(Serialize, Deserialize, Debug)]
struct RealTimeClockMem {
    seconds: u8, // 08h
    minutes: u8, // 09h
    hours: u8, // 0Ah
    days: B9, // 0Bh and 0Ch
    #[skip] __: B5,
    halt: bool,
    day_carry: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RealTimeClock {
    mem : RealTimeClockMem,
    latch_zero: bool,
}

impl RealTimeClock {
    pub fn new() -> RealTimeClock {
        RealTimeClock { 
            mem: RealTimeClockMem::new(),
            latch_zero: false
        }
    }

    pub fn read_reg(&self, ram_bank: usize) -> u8 {
        return self.mem.bytes[ram_bank - 8];
    }

    pub fn write_latch(&mut self, val: u8) {
        // Latch the time when first 0 and then 1 is written
        if val == 0 {
            self.latch_zero = true;
        }
        if self.latch_zero && val == 1 {
            self.latch_current_time();
            self.latch_zero = false;
        }
    }

    fn set_unix_time(&mut self) {
        
    }

    fn latch_current_time(&mut self) {

    }

    /// Every 128 cycles is equivalent to a clock cycle
    fn cycle(&mut self) {

    }
}