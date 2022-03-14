/// This file contains various helper functionality
/// for debugging this emulator. These are intended to be used
/// together with a frontend of some sort to enable advanced debugging.
/// There are also tools for dumping the GPU state as images.

use crate::emulator;
use std::error::Error;
use std::collections::HashSet;

use bmp::{Image, Pixel};

/// If true, the debugger will treat LD B, B as a breakpoint
const ROM_BREAKPOINTS_ENABLED: bool = true;

/// Represents various commands for commandline debugging 
/// of the emulator.
#[derive(PartialEq)]
pub enum CommandType {
    Step(u64),
    Run,
    Breakpoint(Option<u16>),
    PrintRegs,
    PrintMem(u16, u16),
    PrintSteps,
    PrintUniqueInstrs,
    PrintEmulatorState,
    ToggleVerbose,
    ToggleInstrTracking,
    ToggleBreakpoints,
    Quit,
    Error(String),
    None,
}

pub struct DebugState {
    verbose : bool,
    step_counter : u64,
    instr_tracking : bool,
    unique_instr_set : HashSet<u8>,
    use_breakpoints: bool,
    breakpoints : HashSet<u16>
}

impl DebugState {
    pub fn new() -> DebugState {
        DebugState {
            verbose: false, 
            step_counter: 0,
            instr_tracking: false, 
            unique_instr_set : HashSet::new(),
            use_breakpoints: true,
            breakpoints : HashSet::new()
        }
    }
}

/// Execute a debug command
/// `cmd` - the debug command to execute
pub fn execute_debug_command(cmd: &CommandType, em : &mut emulator::Emulator, state: &mut DebugState) {
    match *cmd {
        CommandType::Step(step_size) => {
            step(em, state, step_size); 
            state.step_counter += step_size;
        }
        CommandType::Run => {
            step(em, state, 100_000_000); 
        }
        CommandType::Breakpoint(addr_option) => {
            if let Some(address) = addr_option {
                state.breakpoints.insert(address);
                let temp = em.cpu.regs.pc;
                em.cpu.regs.pc = address;
                println!("Added breakpoint at {:#01x} ({}) ", address, get_instr_name(em));
                em.cpu.regs.pc = temp;
            }
            else {
                state.breakpoints.insert(em.cpu.regs.pc);
                println!("Added breakpoint at {:#01x} ({})", em.cpu.regs.pc, get_instr_name(em));
            }
        }
        CommandType::PrintRegs => {
            em.cpu.regs.debug_display();
            println!("Current instruction: {}", get_instr_name(em));
        }
        CommandType::PrintMem(address, range) => {
            // Cut range if it exceeds memory bounds
            let allowed_range = ((0xffff - address).saturating_add(1)).min(range);
            for i in 0..allowed_range {
                println!("{:#01x}: {:#01x}, {1:3}, {1:#010b}", address + i, em.memory.read_byte(address+i))
            }
        }
        CommandType::PrintSteps => {
            println!("Current step count: {}", state.step_counter);
        }
        CommandType::PrintUniqueInstrs => {
            display_unique_instructions(&state.unique_instr_set)
        }
        CommandType::ToggleVerbose => {
            state.verbose = !state.verbose; 
            println!("Verbose: {}", state.verbose);
        }
        CommandType::ToggleInstrTracking => {
            state.instr_tracking = !state.instr_tracking; 
            println!("Tracking unique instructions encountered: {}", state.instr_tracking);
        }
        CommandType::ToggleBreakpoints => {
            state.use_breakpoints = !state.use_breakpoints;
            println!("Using breakpoints: {}", state.use_breakpoints);
        }
        CommandType::Error(ref message) => {
            println!("Error: {}", message)
        }
        CommandType::Quit => { 
            println!("Exiting Debugger")
        }
        _ => {}
    }
}

/// Step the emulator
/// 
/// `step_size` - the amount of steps/instructions to run.
fn step(em: &mut emulator::Emulator, state : &mut DebugState, step_size: u64) {
    for i in 0..step_size {
        em.step();
        let next = em.memory.read_byte(em.cpu.regs.pc);
        if state.verbose {
            println!("Instr: {} @ pc = {1:#01x} ({1}), (step={2})", get_instr_name(em), em.cpu.regs.pc, i+state.step_counter);
        }
        if state.instr_tracking && !state.unique_instr_set.contains(&next){
            state.unique_instr_set.insert(next);
        }
        if state.use_breakpoints && 
            ((ROM_BREAKPOINTS_ENABLED && next == 0x40) || state.breakpoints.contains(&em.cpu.regs.pc)) {
            println!("Breakpoint triggered at {:#01x} ({})", em.cpu.regs.pc, get_instr_name(em));
            break;
        }
    }
}

/// Get the mnemonic of the current instruction pointed to by the program counter
fn get_instr_name(em: &emulator::Emulator) -> &str {
    let instr = em.memory.read_byte(em.cpu.regs.pc);
    if instr == 0xCB {
        let next = em.memory.read_byte(em.cpu.regs.pc+1);
        return EXTENDED_INSTR_MAP[next as usize];
    }
    else {
        return INSTR_MAP[instr as usize];
    }
}

fn display_unique_instructions(unique_instr_set : &HashSet<u8>) {
    println!("Displaying unique instructions which have been encountered: ");
    for instr in unique_instr_set {
        println!("{:#01x}", instr);
    }
}

// GPU Debugging helpers

const BITMAP_WIDTH : usize = 768;
const BITMAP_HEIGHT: usize = 512;

/// Dump the emulator GPU state to a 768x512 bitmap image
pub fn gpu_state_dump(em: &mut emulator::Emulator) -> Vec<u8> {
    let mut bitmap = vec![255; BITMAP_WIDTH*BITMAP_HEIGHT*3];
    // Render atlas
    draw_tiledata(em, &mut bitmap, false, 0, 0);
    draw_tiledata(em, &mut bitmap, true, 0, 127);
    draw_tilemap(em, &mut bitmap, true, true, 256, 0);
    draw_tilemap(em, &mut bitmap, false, true, 256, 256);
    draw_tilemap(em, &mut bitmap, true, false, 512, 0);
    draw_tilemap(em, &mut bitmap, false, false, 512, 256);
    outline_cross_bitmap(&mut bitmap, 512, 256, 256, 0, 0);
    outline_cross_bitmap(&mut bitmap, 512, 256, 256, 256, 0);
    outline_cross_bitmap(&mut bitmap, 256, 128, 128, 0, 0);

    
    println!("BG: x: {}, y: {} ", em.memory.gpu.scroll_x, em.memory.gpu.scroll_y);
    println!("Window: x: {}, y: {} ", em.memory.gpu.window_x, em.memory.gpu.window_y);
    return bitmap;
}

pub fn parse_number(string : &str) -> Result<u16, Box<dyn Error>> {
    let val =
    if string.starts_with("0x") {
        u16::from_str_radix(&string[2..], 16)?
    }
    else if string.starts_with("$") {
        u16::from_str_radix(&string[1..], 16)?
    }
    else {
        string.parse::<u16>()?
    };
    Ok(val)
}

/// Save the current emulator GPU state as a 768x512 .bmp image.
/// The image is saved in the working directory of the emulator.
pub fn save_gpu_state_to_file(em: &mut emulator::Emulator, filename: &str) {
    let mut img = Image::new(BITMAP_WIDTH as u32, BITMAP_HEIGHT as u32);
    let bitmap = gpu_state_dump(em);
 
    for (x, y) in img.coordinates() {
        let i = (y as usize)*BITMAP_WIDTH + x as usize;
        img.set_pixel(x, y, px!(bitmap[i*3+0], bitmap[i*3+1], bitmap[i*3+2]));
    }
    let _ = img.save(filename);
    println!("Dumped GPU Atlas image to file 'debug.bmp'");
}

/// Outline a cross in the specified bitmap
fn outline_cross_bitmap(bitmap: &mut Vec<u8>, width: usize, center_x: usize, center_y: usize, x_offset : usize, y_offset: usize) {
    for i in 0..width {
        let bix = i*3*BITMAP_WIDTH+center_x*3 + x_offset*3 + y_offset*BITMAP_WIDTH*3;
        bitmap[bix+0] = 0;
        bitmap[bix+1] = 0;
        bitmap[bix+2] = 0;
        let biy = center_y*3*BITMAP_WIDTH+i*3 + x_offset*3 + y_offset*BITMAP_WIDTH*3;
        bitmap[biy+0] = 0;
        bitmap[biy+1] = 0;
        bitmap[biy+2] = 0;
    }
}

/// Draw the Emulator GPU Tiledata on the specified bitmap
fn draw_tiledata(em: &mut emulator::Emulator, bitmap: &mut Vec<u8>, tiledata_select: bool, x_offset: usize, y_offset: usize) {
    for tile_id in 0..256 {
        for y in 0..8 {
            for x in 0..8 {
                let color = em.memory.gpu.draw_helper.get_bg_tile_pixel(tile_id as u8, x, y, tiledata_select);
                let tile_x = tile_id % 16;
                let tile_y = tile_id / 16;
                let i = tile_y*8*BITMAP_WIDTH + y*BITMAP_WIDTH + y_offset*BITMAP_WIDTH +tile_x*8+x + x_offset;
                bitmap[i*3 + 0] = color.r;
                bitmap[i*3 + 1] = color.g;
                bitmap[i*3 + 2] = color.b;
            }
        }
    }
}

/// Draw the Emulator GPU Tilemap on the specified bitmap
fn draw_tilemap(em: &mut emulator::Emulator, bitmap: &mut Vec<u8>, 
                tilemap_select: bool, tiledata_select: bool, x_offset: usize, y_offset: usize) {
    for y in 0..32 {
        for x in 0..32 {
            let tile_id = em.memory.gpu.get_tilemap_id(x, y, tilemap_select);
            for ty in 0..8 {
                for tx in 0..8 {
                    let color = em.memory.gpu.draw_helper.get_bg_tile_pixel(tile_id as u8, tx, ty, tiledata_select);
                    let i = y*8*BITMAP_WIDTH + ty*BITMAP_WIDTH + y_offset*BITMAP_WIDTH +x*8+tx + x_offset;
                    bitmap[i*3 + 0] = color.r;
                    bitmap[i*3 + 1] = color.g;
                    bitmap[i*3 + 2] = color.b;
                }
            }
        }
    }
}

// Instruction Opcode to string maps
const INSTR_MAP : &'static [&str] = &[
    "NOP",              "LD BC",      "LD [BC]",         "INC BC",     "INC B",        "DEC B",      "LD B",         "RLCA",       "LD u16, SP",   "ADD HL, BC", "LD A, [BC]",  "DEC BC",    "INC C",       "DEC C",    "LD C, u8",    "RRCA",
    "STOP",             "LD DE, u16", "LD (DE), A",      "INC DE",     "INC D",        "DEC D",      "LD D, u8",     "RLA",        "JR i8",        "ADD HL, DE", "LD A, [DE]",  "DEC DE",    "INC E",       "DEC E",    "LD E, u8",    "RRA",
    "JR NZ, i8",        "LD HL, u16", "LD [HL+], A",     "INC HL",     "INC H",        "DEC H",      "LD H, u8",     "DAA",        "JR Z, i8",     "ADD HL, HL", "LD A, [HL+]", "DEC HL",    "INC L",       "DEC L",    "LD L, u8",    "CPL",
    "JR NC, i8",        "LD SP, u16", "LD [HL-], A",     "INC SP",     "INC [HL]",     "DEC [HL]",   "LD [HL], u8",  "SCF",        "JR C, i8",     "ADD HL, SP", "LD A, [HL-]", "DEC SP",    "INC A",       "DEC A",    "LD A, u8",    "CCF",
    "LD B, B",          "LD B, C",    "LD B, D",         "LD B, E",    "LD B, H",      "LD B, L",    "LD B, [HL]",   "LD B, A",    "LD C, B",      "LD C, C",    "LD C, D",     "LD C, E",   "LD C, H",     "LD C, L",  "LD C, [HL]",  "LD C, A",
    "LD D, B",          "LD D, C",    "LD D, D",         "LD D, E",    "LD D, H",      "LD D, L",    "LD D, [HL]",   "LD D, A",    "LD E, B",      "LD E, C",    "LD E, D",     "LD E, E",   "LD E, H",     "LD E, L",  "LD E, [HL]",  "LD E, A",
    "LD H, B",          "LD H, C",    "LD H, D",         "LD H, E",    "LD H, H",      "LD H, L",    "LD H, [HL]",   "LD H, A",    "LD L, B",      "LD L, C",    "LD L, D",     "LD L, E",   "LD L, H",     "LD L, L",  "LD L, [HL]",  "LD L, A",
    "LD [HL], B",       "LD [HL], C", "LD [HL], D",      "LD [HL], E", "LD [HL], H",   "LD [HL], L", "HALT",         "LD [HL], A", "LD A, B",      "LD A, C",    "LD A, D",     "LD A, E",   "LD A, H",     "LD A, L",  "LD A, [HL]",  "LD A, A",
    "ADD A, B",         "ADD A, C",   "ADD A, D",        "ADD A, E",   "ADD A, H",     "ADD A, L",   "ADD A, [HL]",  "ADD A, A",   "ADC A, B",     "ADC A, C",   "ADC A, D",    "ADC A, E",  "ADC A, H",    "ADC A, L", "ADC A, [HL]", "ADC A, A",
    "SUB A, B",         "SUB A, C",   "SUB A, D",        "SUB A, E",   "SUB A, H",     "SUB A, L",   "SUB A, [HL]",  "SUB A, A",   "SBC A, B",     "SBC A, C",   "SBC A, D",    "SBC A, E",  "SBC A, H",    "SBC A, L", "SBC A, [HL]", "SBC A, A",
    "AND A, B",         "AND A, C",   "AND A, D",        "AND A, E",   "AND A, H",     "AND A, L",   "AND A, [HL]",  "AND A, A",   "XOR A, B",     "XOR A, C",   "XOR A, D",    "XOR A, E",  "XOR A, H",    "XOR A, L", "XOR A, [HL]", "XOR A, A",
    "OR A, B" ,         "OR A, C" ,   "OR A, D",         "OR A, E",    "OR A, H",      "OR A, L",    "OR A, [HL]",   "OR A, A",    "CP A, B",      "CP A, C",    "CP A, D",     "CP A, E",   "CP A, H",     "CP A, L",  "CP A, [HL]",  "CP A, A",
    "RET NZ",           "POP BC",     "JP NZ, u16",      "JP u16",     "CALL NZ, u16", "PUSH BC",    "ADD A, u8",    "RST $00",    "RET Z",        "RET",        "JP Z, u16",   "PREFIX CB", "CALL Z, u16", "CALL u16", "ADC A, u8",   "RST $08",
    "RET NC",           "POP DE",     "JP NC, u16",      "NOP",        "CALL NZ, u16", "PUSH DE",    "SUB A, u8",    "RST $10",    "RET C",        "RETI",       "JP C, u16",   "NOP",       "CALL C, u16", "NOP",      "SBC A, u8",   "RST $08",
    "LD [$ff00+u8], A", "POP HL",     "LD [$ff00+C], A", "NOP",        "NOP",          "PUSH HL",    "AND A, u8",    "RST $20",    "ADD SP, i8",   "JP HL",      "LD [u16], A", "NOP",       "NOP",         "NOP",      "XOR A, u8",   "RST $08", 
    "LD A, [$ff00+u8]", "POP AF",     "LD A, [$ff00+C]", "DI",         "NOP",          "PUSH AF",    "OR A, u8",     "RST $30",    "LD HL, SP+i8", "LD SP, HL",  "LD A, [u16]", "EI",        "NOP",         "NOP",      "CP A, u8",    "RST $08",
];

// CB extended instructions
const EXTENDED_INSTR_MAP : &'static [&str] = &[
    "RLC B",    "RLC C",    "RLC D",    "RLC H",    "RLC L",    "RLC [HL]",    "RLC A",    "RRC B",    "RRC C",    "RRC D",    "RRC H",    "RRC L",    "RRC [HL]",    "RRC A",
    "RL B",     "RL C",     "RL D",     "RL H",     "RL L",     "RL [HL]",     "RL A",     "RR B",     "RR C",     "RR D",     "RR H",     "RR L",     "RR [HL]",     "RR A",
    "SLA B",    "SLA C",    "SLA D",    "SLA H",    "SLA L",    "SLA [HL]",    "SLA A",    "SRA B",    "SRA C",    "SRA D",    "SRA H",    "SRA L",    "SRA [HL]",    "SRA A",
    "SWAP B",   "SWAP C",   "SWAP D",   "SWAP H",   "SWAP L",   "SWAP [HL]",   "SWAP A",   "SRL B",    "SRL C",    "SRL D",    "SRL H",    "SRL L",    "SRL [HL]",    "SRL A",
    "BIT 0, B", "BIT 0, C", "BIT 0, D", "BIT 0, H", "BIT 0, L", "BIT 0, [HL]", "BIT 0, A", "BIT 1, B", "BIT 1, C", "BIT 1, D", "BIT 1, H", "BIT 1, L", "BIT 1, [HL]", "BIT 1, A",
    "BIT 2, B", "BIT 2, C", "BIT 2, D", "BIT 2, H", "BIT 2, L", "BIT 2, [HL]", "BIT 2, A", "BIT 3, B", "BIT 3, C", "BIT 3, D", "BIT 3, H", "BIT 3, L", "BIT 3, [HL]", "BIT 3, A",
    "BIT 4, B", "BIT 4, C", "BIT 4, D", "BIT 4, H", "BIT 4, L", "BIT 4, [HL]", "BIT 4, A", "BIT 5, B", "BIT 5, C", "BIT 5, D", "BIT 5, H", "BIT 5, L", "BIT 5, [HL]", "BIT 5, A",
    "BIT 6, B", "BIT 6, C", "BIT 6, D", "BIT 6, H", "BIT 6, L", "BIT 6, [HL]", "BIT 6, A", "BIT 7, B", "BIT 7, C", "BIT 7, D", "BIT 7, H", "BIT 7, L", "BIT 7, [HL]", "BIT 7, A",
    "RES 0, B", "RES 0, C", "RES 0, D", "RES 0, H", "RES 0, L", "RES 0, [HL]", "RES 0, A", "RES 1, B", "RES 1, C", "RES 1, D", "RES 1, H", "RES 1, L", "RES 1, [HL]", "RES 1, A",
    "RES 2, B", "RES 2, C", "RES 2, D", "RES 2, H", "RES 2, L", "RES 2, [HL]", "RES 2, A", "RES 3, B", "RES 3, C", "RES 3, D", "RES 3, H", "RES 3, L", "RES 3, [HL]", "RES 3, A",
    "RES 4, B", "RES 4, C", "RES 4, D", "RES 4, H", "RES 4, L", "RES 4, [HL]", "RES 4, A", "RES 5, B", "RES 5, C", "RES 5, D", "RES 5, H", "RES 5, L", "RES 5, [HL]", "RES 5, A",
    "RES 6, B", "RES 6, C", "RES 6, D", "RES 6, H", "RES 6, L", "RES 6, [HL]", "RES 6, A", "RES 7, B", "RES 7, C", "RES 7, D", "RES 7, H", "RES 7, L", "RES 7, [HL]", "RES 7, A",
    "SET 0, B", "SET 0, C", "SET 0, D", "SET 0, H", "SET 0, L", "SET 0, [HL]", "SET 0, A", "SET 1, B", "SET 1, C", "SET 1, D", "SET 1, H", "SET 1, L", "SET 1, [HL]", "SET 1, A",
    "SET 2, B", "SET 2, C", "SET 2, D", "SET 2, H", "SET 2, L", "SET 2, [HL]", "SET 2, A", "SET 3, B", "SET 3, C", "SET 3, D", "SET 3, H", "SET 3, L", "SET 3, [HL]", "SET 3, A",
    "SET 4, B", "SET 4, C", "SET 4, D", "SET 4, H", "SET 4, L", "SET 4, [HL]", "SET 4, A", "SET 5, B", "SET 5, C", "SET 5, D", "SET 5, H", "SET 5, L", "SET 5, [HL]", "SET 5, A",
    "SET 6, B", "SET 6, C", "SET 6, D", "SET 6, H", "SET 6, L", "SET 6, [HL]", "SET 6, A", "SET 7, B", "SET 7, C", "SET 7, D", "SET 7, H", "SET 7, L", "SET 7, [HL]", "SET 7, A",
];
