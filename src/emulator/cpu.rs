mod registers;

pub struct CPU
{
    pub regs : registers::Registers
}

impl CPU {
    pub fn new() -> CPU
    {
        CPU { regs : registers::Registers::new()}
    }
    
    pub fn execute(&mut self, opcode : u8)
    {
        match opcode {
            0x80 => { self.regs.a = self.add(self.regs.b); // ADD B
            }
            _ => { /* TODO: support more instructions */ }
          }
    }

    // ADD Instruction
    fn add(&mut self, value: u8) -> u8
    {
        let (new_value, did_overflow) = self.regs.a.overflowing_add(value);
        self.set_flags(new_value, did_overflow, false, CPU::calculate_half_carry(self.regs.a, value));
        new_value
    }

    // Various helpers
    fn calculate_half_carry(register : u8 , result : u8) -> bool
    {
        (register & 0xF) + (result & 0xF) > 0xF
    }

    fn set_flags(&mut self, new_value : u8, carry : bool, subtract: bool, half_carry : bool)
    {
        self.regs.set_zero_flag(new_value == 0);
        self.regs.set_subtract_flag(subtract);
        self.regs.set_carry_flag(carry);
        self.regs.set_halfcarry_flag(half_carry);

    }
}