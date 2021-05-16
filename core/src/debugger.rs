use crate::emulator;
use text_io::try_read;
use std::io::{self, Write};
use std::collections::HashSet;

#[derive(PartialEq)]
enum CommandType {
    Step(u32),
    Run,
    PrintRegs,
    PrintMem,
    PrintSteps,
    PrintUniqueInstrs,
    PrintEmulatorState,
    ToggleVerbose,
    ToggleInstrTracking,
    Quit,
    None,
}

fn get_input() -> CommandType {
    print!("\n> ");
    io::stdout().flush().expect("Unable to flush stdout");
    let cmd : String = try_read!("{}\n").expect("Unable to read input line");

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
        "mem" | "m" | "printmem" => { CommandType::PrintMem} 
        "verbose" | "v" | "toggleverbose" => { CommandType::ToggleVerbose} 
        "instrtracking" | "it" | "trackinstr" | "trackunique" => {CommandType::ToggleInstrTracking}
        "unique" | "uniqueinstr" | "ui" | "listinstr" => {CommandType::PrintUniqueInstrs}
        "state" | "completestate" => {CommandType::PrintEmulatorState}
        _ => { CommandType::None}
    }
}

pub fn debug(em : &mut emulator::Emulator) {
    let mut cmd = CommandType::None;
    let mut verbose : bool = false;
    let mut instr_tracking : bool = false;
    let mut step_counter : u32 = 0;
    let mut unique_instr_set : HashSet<u8> = HashSet::new();

    print!("Debugging Gameboy ROM {}\n", em.memory.rom.filename);
    while cmd != CommandType::Quit {
        cmd = get_input();
        match cmd {
            CommandType::Step(step_size) => {step(em, step_size, step_counter, verbose, instr_tracking, &mut unique_instr_set); step_counter += step_size;}
            CommandType::Run => { step(em, 100_000_000, step_counter, false, false, &mut unique_instr_set); }
            CommandType::PrintRegs => {em.cpu.regs.debug_display();}
            CommandType::PrintMem => {em.cpu.regs.debug_display();}
            CommandType::PrintSteps => {println!("Current step count: {}", step_counter);}
            CommandType::PrintUniqueInstrs => {display_unique_instructions(&unique_instr_set)}
            CommandType::PrintEmulatorState => {}
            CommandType::ToggleVerbose => {verbose = !verbose; println!("Verbose: {}", verbose);}
            CommandType::ToggleInstrTracking => {instr_tracking = !instr_tracking; println!("Tracking unique instructions encountered: {}", instr_tracking);}
            CommandType::None => {println!("Unknown command. Try again")}
            CommandType::Quit => { println!("Exiting debugger")}
        }
    }
}

pub fn step(em: &mut emulator::Emulator, step_size : u32, step_count : u32, verbose: bool, instr_tracking: bool, unique_instr_set : &mut HashSet<u8> ) {
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

pub fn display_unique_instructions(unique_instr_set : &HashSet<u8>) {
    println!("Displaying unique instructions which have been encountered: ");
    for instr in unique_instr_set {
        println!("{:#01x}", instr);
    }
}

// GPU Debugging helpers
pub fn gpu_state_dump(em: &mut emulator::Emulator) -> Vec<u8> {
    let mut bitmap = vec![255; 256*256*3];
    // Render atlas
    draw_tiledata(em, &mut bitmap, false, 0, 0);
    draw_tiledata(em, &mut bitmap, true, 0, 128);
    outline_bitmap(&mut bitmap);

    println!("BG: x: {}, y: {} ", em.memory.gpu.scroll_x, em.memory.gpu.scroll_y);
    println!("Window: x: {}, y: {} ", em.memory.gpu.window_x, em.memory.gpu.window_y);
    return bitmap;
}

pub fn outline_bitmap(bitmap: &mut Vec<u8>) {
    let center_x = 128;
    let center_y = 128;
    for i in 0..256 {
        let bix = i*3*256+center_x*3;
        bitmap[bix+0] = 0;
        bitmap[bix+1] = 0;
        bitmap[bix+2] = 0;
        let biy = center_y*3*256+i*3;
        bitmap[biy+0] = 0;
        bitmap[biy+1] = 0;
        bitmap[biy+2] = 0;
    }
}

pub fn draw_tiledata(em: &mut emulator::Emulator, bitmap: &mut Vec<u8>, tiledata_select: bool, x_offset: usize, y_offset: usize) {
    for tile_id in 0..256 {
        for y in 0..8 {
            for x in 0..8 {
                let color = em.memory.gpu.draw_helper.get_bg_tile_pixel(tile_id as u8, x, y, tiledata_select);
                let tile_x = tile_id % 16;
                let tile_y = tile_id / 16;
                let i = tile_y*8*256 + y*256 + y_offset*256 +tile_x*8+x + x_offset;
                bitmap[i*3 + 0] = color.r;
                bitmap[i*3 + 1] = color.g;
                bitmap[i*3 + 2] = color.b;
            }
        }
    }
}