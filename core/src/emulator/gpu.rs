
#[derive(Copy, Clone, PartialEq, Debug)]
enum GPUMode {
    HBlankPeriod,
    VBlankPeriod,
    UsingOAMPeriod,
    UsingVRAMPeriod,
}

/// Represents the PPU/GPU of a Gameboy/Gameboy Color.
pub struct GPU {
    pub video_ram: [u8; 8192], // 8kb, 0x8000 - 0x9FFF
    oam_ram: [u8; 160], // 160 bytes, 0xFE00 - 0xFE9F

    pub lcd_control: u8, // 0xFF40 LCDC
    pub lcd_status: u8, // 0xFF41 STAT

    pub scroll_y: u8, // 0xFF42 Scroll Y (Background upper left pos)
    pub scroll_x: u8, // 0xFF43 Scroll X (Background upper left pos)
    pub ly: u8, // xFF44, Current Vertical Line
    pub lyc: u8, // 0xFF45, Compared with ly, if same then STAT interrupt
    pub window_y: u8, // 0xFF4A Window Y (Window upper left pos)
    pub window_x: u8, // 0xFF4B Window X (Window upper left pos)

    pub oam_transfer_request: u8, //0xFF46

    pub background_palette: u8, // 0xFF47 BGP
    pub sprite_palette_1: u8, // 0xFF48
    pub sprite_palette_2: u8, // 0xFF49

    clock_cycles: u16,
    pub scanline_draw_requested: bool,
    pub screen_draw_requested: bool,

    pub vblank_interrupt_requested: bool,

}

impl GPU {
    pub fn new() -> GPU {
        GPU { video_ram: [0; 8192], oam_ram: [0; 160], 
            lcd_control: 0, lcd_status: 0, scroll_y: 0, scroll_x: 0, ly: 0, lyc: 0,
            window_y: 0, window_x: 0, oam_transfer_request: 0, background_palette: 0,
            sprite_palette_1: 0, sprite_palette_2: 0, clock_cycles: 0, 
            scanline_draw_requested: false, screen_draw_requested: false, vblank_interrupt_requested: false }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        match address {
            0x8000 ..= 0x9FFF => { return self.video_ram[address - 0x8000] }
            0xFE00 ..= 0xFE9F => { return self.oam_ram[address - 0xFE00] }
            _ => { panic!("Illegal memory access at addr {} in GPU", address)}
        }
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        match address {
            0x8000 ..= 0x9FFF => { self.video_ram[address - 0x8000] = value; }
            0xFE00 ..= 0xFE9F => { self.oam_ram[address - 0xFE00] = value; }
            _ => { panic!("Illegal memory write at addr {} in GPU", address) }
        }
    }

    pub fn cycle(&mut self, cycles : u16) {
        if !self.get_lcd_display_enable() {
            return; // Display disabled, do not cycle
        } 

        self.clock_cycles += cycles;

        match self.get_lcd_mode_flag() {

            // Horizontal blank period, Scanline not active, 204 cycles
            GPUMode::HBlankPeriod => {
                if self.clock_cycles >= 204 {
                    self.clock_cycles -= 204;
                    self.set_lcd_mode_flag(GPUMode::UsingVRAMPeriod);
                    self.ly += 1;

                    if self.ly > 143 {
                        // Enter vblank
                        self.set_lcd_mode_flag(GPUMode::VBlankPeriod);
                        // Render entire frame
                        self.screen_draw_requested = true;
                        self.vblank_interrupt_requested = true;
                    }
                    else {
                        self.set_lcd_mode_flag(GPUMode::UsingOAMPeriod);
                    }
                }
            }

            // Vertical blank period, Scanline not active, 10 lines * 456 cycles
            GPUMode::VBlankPeriod => {
                if self.clock_cycles >= 456 {
                    self.clock_cycles -= 456;
                    self.ly += 1;
                    if self.ly == 153 { // After 10 lines of VBlank, start drawing again
                        self.ly = 0;
                        self.set_lcd_mode_flag(GPUMode::UsingOAMPeriod);
                    }
                }
            }

            // Read from OAM, Scanline Active, 80 cycles
            GPUMode::UsingOAMPeriod => { 
                if self.clock_cycles >= 80 {
                    self.clock_cycles = self.clock_cycles - 80;
                    self.set_lcd_mode_flag(GPUMode::UsingVRAMPeriod);
                }
            }

            // Read from VRAM, Scanline Active, 172 cycles
            GPUMode::UsingVRAMPeriod => {
                if self.clock_cycles >= 172 {
                    self.clock_cycles -= 172;
                    self.set_lcd_mode_flag(GPUMode::HBlankPeriod);
                    self.scanline_draw_requested = true;
                }
            }
        }

    }

    fn get_lcd_display_enable(&self) -> bool {
        return self.lcd_control & 0b1000_0000 != 0; // Bit 7 of LCDC
    }

    fn get_lcd_mode_flag(&self) -> GPUMode {
        match self.lcd_status & 0b0000_0011 {
            0 => GPUMode::HBlankPeriod,
            1 => GPUMode::VBlankPeriod,
            2 => GPUMode::UsingOAMPeriod,
            3 => GPUMode::UsingVRAMPeriod,
            _ => panic!("Error: Impossible GPUMode detected"),
        }
    }

    fn set_lcd_mode_flag(&mut self, mode : GPUMode) {
        let f = match mode {
            GPUMode::HBlankPeriod => 0,
            GPUMode::VBlankPeriod => 1,
            GPUMode::UsingOAMPeriod => 2,
            GPUMode::UsingVRAMPeriod => 3,
        };
        self.lcd_status = self.lcd_status & 0b1111_1100 | f;
    }
}

#[cfg(test)]
mod test
{
    use super::super::memory;
    use super::GPUMode;

    #[test]
    fn gpu_mode_switching()
    {
        let mut mem = memory::Memory::new();
        mem.write_byte(0xFF40, 0b1000_0000); // Enable LCD, bit 7
        assert_eq!(mem.gpu.get_lcd_mode_flag(), GPUMode::HBlankPeriod); 
        mem.gpu.cycle(204);
        assert_eq!(mem.gpu.get_lcd_mode_flag(), GPUMode::UsingOAMPeriod);
        assert_eq!(mem.gpu.ly, 1);
        mem.gpu.cycle(81); // One extra to carry over
        assert_eq!(mem.gpu.get_lcd_mode_flag(), GPUMode::UsingVRAMPeriod);
        mem.gpu.cycle(171); // One less to adjust for previous
        assert_eq!(mem.gpu.get_lcd_mode_flag(), GPUMode::HBlankPeriod);
        for _i in 0..8*143 {
            mem.gpu.cycle(57); // Should be at VBlank now
        }
        assert_eq!(mem.gpu.get_lcd_mode_flag(), GPUMode::VBlankPeriod);
        assert_eq!(mem.gpu.ly, 144);
    }
}