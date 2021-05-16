
// Important memory locations:
// DIV (general counter) : 0xFF04. Increments every 256 cycles
// TIMA: (timer counter) : 0xFF05. Increments from DIV based on TAC
// TMA: (timer modulo) : 0xFF06. When TIMA overflows, the value will be loaded from here.
// TAC: (timer control) : 0xFF07. Bit 2: Timer Enable. Bit 1 and 0 controls when TIMA is incremented.
// Bit 1 and 0:
// 00: CPU Clock / 1024 (DMG, CGB:   4096 Hz, SGB:   ~4194 Hz)
// 01: CPU Clock / 16   (DMG, CGB: 262144 Hz, SGB: ~268400 Hz)
// 10: CPU Clock / 64   (DMG, CGB:  65536 Hz, SGB:  ~67110 Hz)
// 11: CPU Clock / 256  (DMG, CGB:  16384 Hz, SGB:  ~16780 Hz)
// When TIMA overflows, a TIMER interrupt is sent

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub request_interrupt: bool,
    div_increment_counter: u16,
    tima_increment_counter: u16,
    enabled: bool,
    tima_step: u16,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { div: 0, tima: 0, tma: 0, tac: 0, request_interrupt: false, div_increment_counter: 0,
            tima_increment_counter: 0, enabled: false, tima_step: 256, }
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        match address {
            // Timer 
            0xFF04 => { 
                return self.div; }
            0xFF05 => { return self.tima; }
            0xFF06 => { return self.tma; }
            0xFF07 => { return self.tac; }
            _ => panic!("Invalid memory address encountered")
        }
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        match address {
            0xFF04 => { self.div = 0; }
            0xFF05 => { self.tima = val; }
            0xFF06 => { self.tma = val; }
            0xFF07 => { self.set_tac(val); }
            _ => panic!("Invalid memory address encountered")
        }
    }

    pub fn set_tac(&mut self, tac: u8) {
        self.tac = tac;
        self.enabled = tac & 0b100 == 0b100;
        match tac & 0b11 {
            0b00 => { self.tima_step = 1024}
            0b01 => { self.tima_step = 16}
            0b10 => { self.tima_step = 64}
            0b11 => { self.tima_step = 256}
            _ => { panic!("Timer: Incorrect tac");}
        }
    }

    pub fn increment_by_cycles(&mut self, cycles : u16) {
        self.div_increment_counter += cycles;
        while self.div_increment_counter >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_increment_counter -= 256
        }

        if self.enabled {
            self.tima_increment_counter += cycles;

            while self.tima_increment_counter >= self.tima_step {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.tima = self.tma;
                    self.request_interrupt = true;
                }
                self.tima_increment_counter -= self.tima_step;
            }
        }
    }
}