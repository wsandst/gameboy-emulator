
/// Represents the Gameboy Joypad
/// 
/// The gameboy has 8 keys: 4 arrow keys, A, B, Select and Start
/// Which button is depressed is stored in 0xFF00 (JOYP),
/// according to the bit layout below:
///        Bit 5   Bit 4
/// Bit 3: START   DOWN  
/// Bit 2: SELECT  UP 
/// Bit 1: B       LEFT 
/// Bit 0: A       RIGHT 
/// A depressed key has value 0 for the bit
/// 
/// The systems asks for a keypress to be read by writing either 
/// 0x10 (bit 4) or 0x20 (bit 5) to JOYPAD

use super::super::emulator::KeyPress;

pub struct Joypad {
    // These together represent JOYP
    key_column_select: u8, // Bit 4/5
    // 0: Right, left, up, down, 1: A, b, select, start
    key_columns: [u8; 2],
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad { key_column_select: 0, key_columns: [0xF, 0xF]}
    }

    pub fn write_byte(&mut self, joyp: u8) {
        self.key_column_select = joyp & 0x30;
    }

    pub fn read_byte(&self) -> u8 {
        //println!("r1: {0:#010b}, r2: {1:#010b}", self.key_columns[0], self.key_columns[1]);
        return match self.key_column_select {
            0x10 => self.key_columns[0],
            0x20 => self.key_columns[1],
            _ => 0,
        }
    }

    pub fn press_key(&mut self, key: KeyPress) {
        match key {
            KeyPress::Right =>    { self.key_columns[1] &= !(1 << 0) } // Bit 0
            KeyPress::Left =>     { self.key_columns[1] &= !(1 << 1) } // Bit 1
            KeyPress::Up =>       { self.key_columns[1] &= !(1 << 2) } // Bit 2
            KeyPress::Down =>     { self.key_columns[1] &= !(1 << 3) } // Bit 3
            KeyPress::A =>        { self.key_columns[0] &= !(1 << 0) } // Bit 0
            KeyPress::B =>        { self.key_columns[0] &= !(1 << 1) } // Bit 1
            KeyPress::Select =>   { self.key_columns[0] &= !(1 << 2) } // Bit 2
            KeyPress::Start =>    { self.key_columns[0] &= !(1 << 3) } // Bit 3
        }
    }

    /// Set key bit to 0
    pub fn clear_key(&mut self, key: KeyPress) {
        match key {
            KeyPress::Right =>    { self.key_columns[1] |= 1 << 0 } // Bit 0
            KeyPress::Left =>     { self.key_columns[1] |= 1 << 1 } // Bit 1
            KeyPress::Up =>       { self.key_columns[1] |= 1 << 2 } // Bit 2
            KeyPress::Down =>     { self.key_columns[1] |= 1 << 3 } // Bit 3
            KeyPress::A =>        { self.key_columns[0] |= 1 << 0 } // Bit 0
            KeyPress::B =>        { self.key_columns[0] |= 1 << 1 } // Bit 1
            KeyPress::Select =>   { self.key_columns[0] |= 1 << 2 } // Bit 2
            KeyPress::Start =>    { self.key_columns[0] |= 1 << 3 } // Bit 3
        }
    }

    pub fn clear_all_keys(&mut self) {
        self.key_columns[0] = 0x0F;
        self.key_columns[1] = 0x0F;
    }
}

#[cfg(test)]
mod test
{
    use super::Joypad;
    use super::super::KeyPress;

    #[test]
    fn joypad_test()
    {
        let mut joypad = Joypad::new();
        joypad.write_byte(0x10);
        // Test START, A, B, Select
        joypad.press_key(KeyPress::Start);
        assert_eq!(joypad.read_byte(), 0b0000_0111);
        joypad.press_key(KeyPress::A);
        joypad.press_key(KeyPress::B);
        joypad.press_key(KeyPress::Select);
        assert_eq!(joypad.read_byte(), 0b0000_0000);
        // Test switching
        joypad.write_byte(0x20);
        assert_eq!(joypad.read_byte(), 0b0000_1111);
        // Test Down, Up, Left, Right
        joypad.press_key(KeyPress::Down);
        assert_eq!(joypad.read_byte(), 0b0000_0111);
        joypad.press_key(KeyPress::Up);
        joypad.press_key(KeyPress::Left);
        joypad.press_key(KeyPress::Right);
        assert_eq!(joypad.read_byte(), 0b0000_0000);
        // Test clearing
        joypad.clear_all_keys();
        assert_eq!(joypad.read_byte(), 0b0000_1111);
        joypad.write_byte(0x10);
        assert_eq!(joypad.read_byte(), 0b0000_1111);
        // Test edge case, no column selected
        // Think this should be 0, but could be 0x0F?
        joypad.write_byte(0x00);
        assert_eq!(joypad.read_byte(), 0b0000_0000);
    }
}