use super::rom;
use super::gpu;
use super::interrupts;
use super::timer;
use std::io::{self, Write};

pub struct Memory
{
    // 64kb (2^16) address-able space
    pub rom: rom::Rom, // ROM, can be switched, 8kb ROM 0x0 - 0x7FFF, External RAM 0xA000 - BFFF
    pub gpu: gpu::GPU, // GPU/PPU. VRAM 0x8000 - 0x9FFF, OAM 0xFE00 - 0xFE9F
    working_ram: [u8; 8192], // 8kb, 0xC000 - 0xDFFFF
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
            working_ram: [1; 8192],
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
            // Timer
            0xFF04 => { return self.timer.div; }
            0xFF05 => { return self.timer.tima; }
            0xFF06 => { return self.timer.tma; }
            0xFF07 => { return self.timer.tac; }

            // PPU/GPU
            0xFF40 => { return self.gpu.lcd_control; }
            0xFF41 => { return self.gpu.lcd_status; }
            0xFF42 => { return self.gpu.scroll_y; }
            0xFF43 => { return self.gpu.scroll_x; }
            0xFF44 => { return self.gpu.ly; }
            0xFF45 => { return self.gpu.lyc; }
            0xFF46 => { return self.gpu.oam_transfer_request; }
            0xFF47 => { return self.gpu.background_palette; }
            0xFF48 => { return self.gpu.sprite_palette_0; }
            0xFF49 => { return self.gpu.lyc; }
            0xFF4A => { return self.gpu.window_y; }
            0xFF4B => { return self.gpu.window_x; }

            0xFF00 ..= 0xFF7F => { return self.device_ram[address - 0xFF00]}
            _ => { return 0; }
        }
    }

    /**
    lcd_control: u8, // 0xFF40 LCDC
    lcd_status: u8, // 0xFF41 STAT

    scroll_y: u8, // 0xFF42 Scroll Y (Background upper left pos)
    scroll_x: u8, // 0xFF43 Scroll X (Background upper left pos)
    ly: u8, // xFF44, Current Vertical Line
    lyc: u8, // 0xFF45, Compared with ly, if same then STAT interrupt
    window_y: u8, // 0xFF4A Window Y (Window upper left pos)
    window_x: u8, // 0xFF4B Window X (Window upper left pos)

    oam_transfer_request: u8, //0xFF46

    background_palette: u8, // 0xFF47 BGP
    sprite_palette_0: u8, // 0xFF48
    sprite_palette_1: u8, // 0xFF49
     */
    pub fn write_byte_devices(&mut self, address : usize, val: u8) {
        match address {
            // Timer
            0xFF04 => { self.timer.div = 0; }
            0xFF05 => { self.timer.tima = val; }
            0xFF06 => { self.timer.tma = val; }
            0xFF07 => { self.timer.set_tac(val); }
            
            // PPU/GPU
            0xFF40 => { self.gpu.lcd_control = val; }
            0xFF41 => { self.gpu.lcd_status = val; }
            0xFF42 => { self.gpu.scroll_y = val; }
            0xFF43 => { self.gpu.scroll_x = val; }
            0xFF44 => { self.gpu.ly = val; }
            0xFF45 => { self.gpu.lyc = val; }
            0xFF46 => { self.gpu.oam_transfer_request = val; }
            0xFF47 => { self.gpu.background_palette = val; }
            0xFF48 => { self.gpu.sprite_palette_0 = val; }
            0xFF49 => { self.gpu.lyc = val; }
            0xFF4A => { self.gpu.window_y = val; }
            0xFF4B => { self.gpu.window_x = val; }

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
    }

    pub fn cycle_devices(&mut self, machine_cycles: u16) {
        self.timer.increment_by_cycles(machine_cycles*4);
        self.gpu.cycle(machine_cycles*4);
        self.propagate_interrupt_requests();
    }
}