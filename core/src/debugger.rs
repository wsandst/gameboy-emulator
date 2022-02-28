/// This file contains various functionality
/// for debugging this emulator. A commandline tool
/// for stepping through the emulator CPU is available, as
/// well as functionality for dumping GPU state to images.
use crate::emulator;
use std::error::Error;
use std::collections::HashSet;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use bmp::{Image, Pixel};

/// Represents various commands for commandline debugging 
/// of the emulator.
#[derive(PartialEq)]
enum CommandType {
    Step(u32),
    Run,
    PrintRegs,
    PrintMem(u16, u16),
    PrintSteps,
    PrintUniqueInstrs,
    PrintEmulatorState,
    ToggleVerbose,
    ToggleInstrTracking,
    Quit,
    Error(String),
    None,
}

struct DebugState {
    verbose : bool,
    instr_tracking : bool,
    step_counter : u32,
    unique_instr_set : HashSet<u8>
}

/// Prompt the debugging user for the next command
/// 
/// `rl` - readlines library object which keeps track of history
fn get_input(rl : &mut Editor::<()>) -> CommandType {
    // Parse the line using readlines to get history
    let readline = rl.readline(">> ");
    let cmd = match readline {
        Ok(line) => {
            rl.add_history_entry(line.as_str());
            line.as_str().to_string()
        },
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
            return CommandType::Quit;
        },
        Err(err) => {
            println!("Error: {:?}", err);
            return CommandType::Quit;
        }
    };

    // Parse into a command
    let split = cmd.split(" ");
    let words = split.collect::<Vec<&str>>();
    let arg_count = words.len();

    match words[0] {
        "quit" | "stop" | "exit" | "q" => { CommandType::Quit}
        "step" | "s" => { 
            if arg_count > 1 {
                CommandType::Step(words[1].parse::<u32>().unwrap())
            } else {
                CommandType::Step(1)
            }
        }
        "run" => { CommandType::Run }
        "steps" | "printsteps" | "stepcount" => { CommandType::PrintSteps}
        "regs" | "r" | "printregs" => { CommandType::PrintRegs}
        "mem" | "m" | "printmem" | "inspect" => { 
            if arg_count > 1 {
                let range = if arg_count > 2 {
                    // If a second argument is specified, print a range of memory
                    match parse_number(words[2]) {
                        Ok(range) => { range }
                        Err(error) => { 
                            return CommandType::Error(format!("Unable to parse the specified address, {}", error).to_string());
                        }
                    }
                }
                else {
                    1
                };
                match parse_number(words[1]) {
                    Ok(address) => { CommandType::PrintMem(address, range) }
                    Err(error) => { CommandType::Error(format!("Unable to parse the specified address, {}", error).to_string()) }
                }
            } else {
                CommandType::Error("Please specify a memory address to inspect".to_string())
            }
        } 
        "verbose" | "v" | "toggleverbose" => { CommandType::ToggleVerbose} 
        "instrtracking" | "it" | "trackinstr" | "trackunique" => {CommandType::ToggleInstrTracking}
        "unique" | "uniqueinstr" | "ui" | "listinstr" => {CommandType::PrintUniqueInstrs}
        "state" | "completestate" => {CommandType::PrintEmulatorState}
        _ => { CommandType::Error("Unknown command specified".to_string())}
    }
}

/// Commandline tool for debugging an emulator. Allows for
/// stepping through the emulator and inspecting memory.
pub fn debug(em : &mut emulator::Emulator) {
    let mut state = DebugState { 
        verbose: false, 
        instr_tracking: false, 
        step_counter: 0,
        unique_instr_set :  HashSet::new()
    };

    // Setup readlines history
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").unwrap();

    let mut cmd = CommandType::None;
    print!("\nDebugging Gameboy ROM {}\n", em.memory.rom.filename);

    while cmd != CommandType::Quit {
        cmd = get_input(&mut rl);
        execute_debug_command(&cmd, em, &mut state);
    }

    rl.save_history("history.txt").unwrap();
}

fn execute_debug_command(cmd: &CommandType, em : &mut emulator::Emulator, state: &mut DebugState) {
    match *cmd {
        CommandType::Step(step_size) => {
            step(em, step_size, state.step_counter, state.verbose, state.instr_tracking, &mut state.unique_instr_set); 
            state.step_counter += step_size;
        }
        CommandType::Run => {
            step(em, 100_000_000, state.step_counter, false, false, &mut state.unique_instr_set); 
        }
        CommandType::PrintRegs => {
            em.cpu.regs.debug_display();
        }
        CommandType::PrintMem(address, range) => {
            // Cut range if it exceeds memory bounds
            println!("{}", range);
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
            state.verbose = !state.verbose; println!("Verbose: {}", state.verbose);
        }
        CommandType::ToggleInstrTracking => {
            state.instr_tracking = !state.instr_tracking; 
            println!("Tracking unique instructions encountered: {}", state.instr_tracking);
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


fn step(em: &mut emulator::Emulator, step_size : u32, step_count : u32, verbose: bool, instr_tracking: bool, unique_instr_set : &mut HashSet<u8> ) {
    for i in 0..step_size {
        em.step();
        let next = em.memory.read_byte(em.cpu.regs.pc);
        if verbose {
            println!("Instr: {:#01x} @ pc = {1:#01x} ({1}), (step={2})", next, em.cpu.regs.pc, i+step_count);
        }
        if instr_tracking && !unique_instr_set.contains(&next){
            unique_instr_set.insert(next);
        }
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

fn parse_number(string : &str) -> Result<u16, Box<dyn Error>> {
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