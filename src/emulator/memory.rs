mod rom;
use std::io::{self, Write};

const WRAM_SIZE: usize = 8192; // 8 kb
const VRAM_SIZE: usize = 8192; // 8 kb
const ERAM_SIZE: usize = 8192; // 8 kb

pub struct Memory<'a>
{
    // 64kb (2^16) address-able space
    pub rom: rom::Rom, // ROM, can be switched, 8kb, 0x0 - 0x7FFF
    video_ram: [u8; VRAM_SIZE], // 8kb, 0x8000 - 0x9FFF
    external_ram: [u8; ERAM_SIZE], // 8kb, (usually from cartridge), 0xA000 - BFFF
    working_ram: [u8; WRAM_SIZE], // 8kb, 0xC000 - 0xDFFFF
    oam_ram: [u8; 160], // 160 bytes, 0xFE00 - 0xFE9F
    device_ram: [u8; 128], // 128 bytes, 0xFF00 - 0xFF7F
    high_ram: [u8; 127], // 127 bytes, 0xFF80 - 0xFFFE
    pub interrupt_flag: u8,
    // Serial callback object.
    // stdout implements the trait io::write, but also vector, which makes it useful for debugging
    pub serial_callback: Box<dyn io::Write + 'a>,
}

impl<'a> Memory<'a> {
    pub fn new() -> Memory<'a>
    {
        Memory { 
            rom: rom::Rom::new(),
            video_ram: [1; 8192],
            external_ram: [0; 8192],
            working_ram: [1; 8192],
            oam_ram: [0; 160],
            device_ram: [0; 128],
            high_ram: [0; 127],
            interrupt_flag: 0,
            serial_callback: Box::new(io::stdout()),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8
    {
        let address = address as usize;
        match address {
            0x0000 ..= 0x7FFF => { return self.rom.read_byte(address - 0x0000)}
            0x8000 ..= 0x9FFF => { return self.video_ram[address - 0x8000]}
            0xA000 ..= 0xBFFF => { return self.external_ram[address - 0xA000]}
            0xC000 ..= 0xDFFF => { return self.working_ram[address - 0xC000]}
            0xE000 ..= 0xFDFF => { return self.working_ram[address - 0xE000]} // Echo ram
            0xFE00 ..= 0xFE9F => { return self.oam_ram[address - 0xFE00]}
            0xFEA0 ..= 0xFEFF => {} // Unused RAM
            0xFF00 ..= 0xFF7F => { return self.device_ram[address - 0xFF00]}
            0xFF80 ..= 0xFFFE => { return self.high_ram[address - 0xFF80]}
            0xFFFF => { return self.interrupt_flag}
            _ => {},
        }
        return 0;
    }

    pub fn read_word(&self, address: u16) -> u16 
    {
        (self.read_byte(address) as u16) | ((self.read_byte(address+1) as u16) << 8)
    }

    pub fn write_byte(&mut self, address: u16, value : u8)
    {
        let address = address as usize;
        match address {
            0x0000 ..= 0x7FFF => {self.rom.write_byte(address - 0x0000, value)}
            0x8000 ..= 0x9FFF => {self.video_ram[address - 0x8000] = value}
            0xA000 ..= 0xBFFF => {self.external_ram[address - 0xA000] = value}
            0xC000 ..= 0xDFFF => {self.working_ram[address - 0xC000] = value}
            0xE000 ..= 0xFDFF => {self.working_ram[address - 0xE000] = value} // Echo ram
            0xFE00 ..= 0xFE9F => {self.oam_ram[address - 0xFE00] = value}
            0xFEA0 ..= 0xFEFF => {} // Unused RAM
            0xFF02 if value == 0x81 => { self.link_cable_serial(self.read_byte(0xFF01)); }
            0xFF00 ..= 0xFF7F => {self.device_ram[address - 0xFF00] = value}
            0xFF80 ..= 0xFFFE => {self.high_ram[address - 0xFF80] = value}
            0xFFFF => {self.interrupt_flag = value}
            _ => {},
        }
    }
    pub fn write_word(&mut self, address: u16, value : u16)
    {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    // /Write to link cable, used as debug output
    pub fn link_cable_serial(&mut self, c: u8) {
        write!(self.serial_callback, "{}", c as char);
        io::stdout().flush().expect("Unable to flush stdout");
    }
}