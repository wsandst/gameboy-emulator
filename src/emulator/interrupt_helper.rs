use super::memory;

pub enum InterruptTypes {
    VBlank,
    Stat,
    Timer,
    Serial,
    Joypad,
    None,
}

pub struct InterruptHelper; 

impl InterruptHelper {
    pub fn is_interrupt_pending(memory: &memory::Memory) -> bool {
        return memory.interrupt_master_enable && (Self::get_combined_interrupt_flag(memory) > 0);
    }

    pub fn get_combined_interrupt_flag(memory: &memory::Memory) -> u8 {
        // Return IF & IE
        return memory.interrupt_enable & Self::get_if(memory) & 0b0001_1111;
    }

    pub fn get_highest_priority_interrupt(memory: &memory::Memory) -> InterruptTypes {
        let iflag = Self::get_combined_interrupt_flag(memory);
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

    pub fn clear_interrupt(memory: &mut memory::Memory, interrupt_type: InterruptTypes) {
        let mut interrupt_flag = Self::get_if(memory);
        match interrupt_type {
            InterruptTypes::VBlank => { interrupt_flag = interrupt_flag ^ 0b0000_0001}
            InterruptTypes::Stat   => { interrupt_flag = interrupt_flag ^ 0b0000_0010}
            InterruptTypes::Timer  => { interrupt_flag = interrupt_flag ^ 0b0000_0100}
            InterruptTypes::Serial => { interrupt_flag = interrupt_flag ^ 0b0000_1000}
            InterruptTypes::Joypad => { interrupt_flag = interrupt_flag ^ 0b0001_0000}
            _ => { }
        }
        Self::set_if(memory, interrupt_flag)
    }

    pub fn get_if(memory: &memory::Memory) -> u8 {
        return memory.read_byte(0xFF0F);
    }

    pub fn set_if(memory: &mut memory::Memory, val: u8) {
        memory.write_byte(0xFF0F, val);
    }

    pub fn trigger_interrupt(memory: &mut memory::Memory, interrupt_type : InterruptTypes) {
        let mut interrupt_flag = Self::get_if(memory);
        match interrupt_type {
            InterruptTypes::VBlank => { interrupt_flag = interrupt_flag | 0b0000_0001}
            InterruptTypes::Stat   => { interrupt_flag = interrupt_flag | 0b0000_0010}
            InterruptTypes::Timer  => { interrupt_flag = interrupt_flag | 0b0000_0100}
            InterruptTypes::Serial => { interrupt_flag = interrupt_flag | 0b0000_1000}
            InterruptTypes::Joypad => { interrupt_flag = interrupt_flag | 0b0001_0000}
            _ => { }
        }
        Self::set_if(memory, interrupt_flag)
    }
}