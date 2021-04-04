
mod timer;

pub struct Devices {
    pub timer: timer::Timer,
    other_device_ram: [u8; 128],
}

impl Devices {
    pub fn new() -> Devices
    {
        Devices { timer: timer::Timer::new(), other_device_ram: [0; 128]}
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        match address {
            // Timer
            0xFF04 => { return self.timer.div; }
            0xFF05 => { return self.timer.tima; }
            0xFF06 => { return self.timer.tma; }
            0xFF07 => { return self.timer.tac; }

            0xFF00 ..= 0xFF7F => { return self.other_device_ram[address - 0xFF00]}
            _ => { return 0; }
        }
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        match address {
            // Timer
            0xFF04 => { self.timer.div = 0; }
            0xFF05 => { self.timer.tima = val; }
            0xFF06 => { self.timer.tma = val; }
            0xFF07 => { self.timer.set_tac(val); }
            
            0xFF00 ..= 0xFF7F => { self.other_device_ram[address - 0xFF00] = val;}
            _ => {  }
        }
    }
}