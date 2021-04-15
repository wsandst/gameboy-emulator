use super::rom;
use super::gpu;
use super::interrupts;
use super::timer;
use std::io::{self, Write};

const KB : usize = 1024;

pub struct Memory
{
    // 64kb (2^16) address-able space
    pub rom: rom::Rom, // ROM, can be switched, 8kb ROM 0x0 - 0x7FFF, External RAM 0xA000 - BFFF
    pub gpu: gpu::GPU, // GPU/PPU. VRAM 0x8000 - 0x9FFF, OAM 0xFE00 - 0xFE9F
    working_ram: [u8; 8*KB], // 8kb, 0xC000 - 0xDFFFF
    high_ram: [u8; 127], // 127 bytes, 0xFF80 - 0xFFFE
    device_ram: [u8; 128],
    // Interrupt related
    pub interrupt_handler : interrupts::InterruptHandler,
    pub timer: timer::Timer,
    // Serial callback object.
    // stdout implements the trait io::write, but also vector, which makes it useful for debugging
    pub serial_buffer: Vec<u8>,
    pub output_serial_to_stdout: bool,
}

impl Memory {
    pub fn new() -> Memory
    {
        Memory { 
            rom: rom::Rom::new(),
            gpu: gpu::GPU::new(),
            working_ram: [1; 8*KB],
            high_ram: [0; 127],
            device_ram: [0; 128],
            interrupt_handler : interrupts::InterruptHandler::new(),
            timer: timer::Timer::new(),
            serial_buffer: Vec::new(),
            output_serial_to_stdout: true,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8
    {
        let address = address as usize;
        match address {
            0x0000 ..= 0x7FFF | 
            0xA000 ..= 0xBFFF => { return self.rom.read_byte(address)} // ROM and External RAM in rom
            0x8000 ..= 0x9FFF |
            0xFE00 ..= 0xFE9F => { return self.gpu.read_byte(address)} // VRAM and OAM in GPU
            0xC000 ..= 0xDFFF => { return self.working_ram[address - 0xC000]}
            0xE000 ..= 0xFDFF => { return self.working_ram[address - 0xE000]} // Echo ram
            0xFEA0 ..= 0xFEFF => {} // Unused RAM
            0xFF0F => { return self.interrupt_handler.interrupt_flag }
            0xFF00 ..= 0xFF7F => { return self.read_byte_devices(address)}
            0xFF80 ..= 0xFFFE => { return self.high_ram[address - 0xFF80]}
            0xFFFF => { return self.interrupt_handler.interrupt_enable}
            _ => {},
        }
        return 0xFF;
    }

    pub fn read_word(&self, address: u16) -> u16 
    {
        (self.read_byte(address) as u16) | ((self.read_byte(address+1) as u16) << 8)
    }

    pub fn write_byte(&mut self, address: u16, value : u8)
    {
        let address = address as usize;
        match address {
            0x0000 ..= 0x7FFF | 
            0xA000 ..= 0xBFFF => { self.rom.write_byte(address, value)} // ROM and External RAM in rom
            0x8000 ..= 0x9FFF |
            0xFE00 ..= 0xFE9F => { self.gpu.write_byte(address, value)} // VRAM and OAM in GPU
            0xC000 ..= 0xDFFF => { self.working_ram[address - 0xC000] = value}
            0xE000 ..= 0xFDFF => { self.working_ram[address - 0xE000] = value} // Echo ram
            0xFEA0 ..= 0xFEFF => {} // Unused RAM
            0xFF02 if value == 0x81 => { self.link_cable_serial(self.read_byte(0xFF01)); }
            0xFF0F => { self.interrupt_handler.interrupt_flag = value}
            0xFF00 ..= 0xFF7F => { self.write_byte_devices(address, value);}
            0xFF80 ..= 0xFFFE => { self.high_ram[address - 0xFF80] = value}
            0xFFFF => { self.interrupt_handler.interrupt_enable = value}
            _ => {},
        }
    }
    pub fn write_word(&mut self, address: u16, value : u16)
    {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    pub fn read_byte_devices(&self, address : usize) -> u8 {
        match address {
            0xFF01 => { return self.device_ram[1]; }
            // Timer 
            0xFF04 ..= 0xFF07 => { return self.timer.read_byte(address); }

            // PPU/GPU
            0xFF40 ..= 0xFF4B => { return self.gpu.read_byte(address)}

            0xFF00 ..= 0xFF7F => { return 0xFF;}//return self.device_ram[address - 0xFF00]}
            _ => { return 0xFF; }
        }
    }

    pub fn write_byte_devices(&mut self, address : usize, val: u8) {
        match address {
            // Timer
            0xFF04 ..= 0xFF07 => {self.timer.write_byte(address, val); }
            
            // PPU/GPU
            0xFF40 ..= 0xFF4B => { self.gpu.write_byte(address, val); }

            0xFF00 ..= 0xFF7F => { self.device_ram[address - 0xFF00] = val;}
            _ => {  }
        }
    }

    // /Write to link cable, used as debug output
    pub fn link_cable_serial(&mut self, c: u8) {
        self.serial_buffer.push(c);
        if self.output_serial_to_stdout {
            print!("{}", c as char);
            io::stdout().flush().expect("Unable to flush stdout");
        }
    }

    pub fn propagate_interrupt_requests(&mut self) {
        if self.timer.request_interrupt {
            self.interrupt_handler.trigger_interrupt(interrupts::InterruptTypes::Timer);
            self.timer.request_interrupt = false;
        }
        if self.gpu.vblank_interrupt_requested  {
            self.interrupt_handler.trigger_interrupt(interrupts::InterruptTypes::VBlank);
            self.gpu.vblank_interrupt_requested = false;
        }
    }

    pub fn cycle_devices(&mut self, machine_cycles: u16) {
        self.timer.increment_by_cycles(machine_cycles*4);
        self.gpu.cycle(machine_cycles*4);
        self.propagate_interrupt_requests();
    }
}