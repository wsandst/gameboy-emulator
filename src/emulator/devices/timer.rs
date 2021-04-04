
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

pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { div: 0, tima: 0, tma: 0, tac: 0 }
    }
}