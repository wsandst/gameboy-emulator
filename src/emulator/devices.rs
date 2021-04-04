
mod timer;

pub struct Devices {
    timer: timer::Timer,
    other_device_ram: [u8; 128],
}

impl Devices {
    pub fn new() -> Devices
    {
        Devices { timer: timer::Timer::new(), other_device_ram: [0; 128]}
    }

    pub fn read_byte(&self, address : usize) -> u8 {
        match address {
            0xFF00 ..= 0xFF7F => { return self.other_device_ram[address - 0xFF00]}
            _ => { return 0; }
        }
    }

    pub fn write_byte(&mut self, address : usize, val: u8) {
        match address {
            0xFF00 ..= 0xFF7F => { self.other_device_ram[address - 0xFF00] = val;}
            _ => {  }
        }
    }
}