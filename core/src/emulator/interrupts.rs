
use serde::{Serialize, Deserialize};

pub enum InterruptTypes {
    VBlank,
    Stat,
    Timer,
    Serial,
    Joypad,
    None,
}

#[derive(Serialize, Deserialize)]
pub struct InterruptHandler {
    pub interrupt_enable : u8, // IE, 0xFFFF, user controlled
    pub interrupt_flag: u8, // IF, 0xFF0F, hardware controlled
    pub interrupt_master_enable : bool, // IME, master toggle
    ei_requested: bool,
    di_requested: bool,
}

impl InterruptHandler {
    pub fn new() -> InterruptHandler
    {
        InterruptHandler { interrupt_enable : 0, interrupt_flag: 0, interrupt_master_enable: false, ei_requested: false, di_requested: false }
    }

    pub fn is_interrupt_pending(&self) -> bool {
        return self.interrupt_master_enable && (self.get_combined_interrupt_flag() > 0);
    }

    pub fn get_combined_interrupt_flag(&self) -> u8 {
        // Return IF & IE
        return self.interrupt_enable & self.interrupt_flag & 0b0001_1111;
    }

    pub fn get_highest_priority_interrupt(&self) -> InterruptTypes {
        let iflag = self.get_combined_interrupt_flag();
        if iflag & 0b0000_0001 != 0 {
            return InterruptTypes::VBlank;
        }
        else if iflag & 0b0000_0010 != 0 {
            return InterruptTypes::Stat;
        }
        else if iflag & 0b0000_0100 != 0 {
            return InterruptTypes::Timer;
        }
        else if iflag & 0b0000_1000 != 0 {
            return InterruptTypes::Serial;
        }
        else if iflag & 0b0001_0000 != 0 {
            return InterruptTypes::Joypad;
        }
        else {
            return InterruptTypes::None;
        }
    }

    pub fn clear_interrupt(&mut self, interrupt_type: InterruptTypes) {
        match interrupt_type {
            InterruptTypes::VBlank => { self.interrupt_flag = self.interrupt_flag ^ 0b0000_0001}
            InterruptTypes::Stat   => { self.interrupt_flag = self.interrupt_flag ^ 0b0000_0010}
            InterruptTypes::Timer  => { self.interrupt_flag = self.interrupt_flag ^ 0b0000_0100}
            InterruptTypes::Serial => { self.interrupt_flag = self.interrupt_flag ^ 0b0000_1000}
            InterruptTypes::Joypad => { self.interrupt_flag = self.interrupt_flag ^ 0b0001_0000}
            _ => { }
        }
    }

    pub fn trigger_interrupt(&mut self, interrupt_type : InterruptTypes) {
        match interrupt_type {
            InterruptTypes::VBlank => { self.interrupt_flag = self.interrupt_flag | 0b0000_0001}
            InterruptTypes::Stat   => { self.interrupt_flag = self.interrupt_flag | 0b0000_0010}
            InterruptTypes::Timer  => { self.interrupt_flag = self.interrupt_flag | 0b0000_0100}
            InterruptTypes::Serial => { self.interrupt_flag = self.interrupt_flag | 0b0000_1000}
            InterruptTypes::Joypad => { self.interrupt_flag = self.interrupt_flag | 0b0001_0000}
            _ => { }
        }
    }

    pub fn update_ime(&mut self) {
        if self.ei_requested {
            self.interrupt_master_enable = true;
            self.ei_requested = false;
        }
        else if self.di_requested {
            self.interrupt_master_enable = false;
            self.di_requested = false;
        }
    }

    pub fn request_ei(&mut self) {
        self.ei_requested = true;
    }

    pub fn request_di(&mut self) {
        self.di_requested = true;
    }
}