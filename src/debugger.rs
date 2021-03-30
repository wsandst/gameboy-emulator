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
        _ => { CommandType::None}
    }
}

pub fn debug(em : &mut emulator::Emulator) {
    let mut cmd = CommandType::None;
    let mut verbose : bool = true;
    let mut instr_tracking : bool = false;
    let mut step_counter : u32 = 0;
    let mut unique_instr_set : HashSet<u8> = HashSet::new();

    print!("Debugging Gameboy ROM {}\n", em.memory.rom.filename);
    while cmd != CommandType::Quit {
        cmd = get_input();
        match cmd {
            CommandType::Step(step_size) => {step(em, step_size, step_counter, verbose, instr_tracking, &mut unique_instr_set); step_counter += step_size;}
            CommandType::Run => { step(em, 10000000, step_counter, false, false, &mut unique_instr_set); }
            CommandType::PrintRegs => {em.cpu.regs.debug_display();}
            CommandType::PrintMem => {em.cpu.regs.debug_display();}
            CommandType::PrintSteps => {println!("Current step count: {}", step_counter);}
            CommandType::PrintUniqueInstrs => {display_unique_instructions(&unique_instr_set)}
            CommandType::ToggleVerbose => {verbose = !verbose; println!("Verbose: {}", verbose);}
            CommandType::ToggleInstrTracking => {instr_tracking = !instr_tracking; println!("Tracking unique instructions encountered: {}", instr_tracking);}
            CommandType::None => {println!("Unknown command. Try again")}
            CommandType::Quit => { println!("Exiting program")}
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
        if next == 0x10 { // HALT, exit
            println!("Program halted.");
            break;
        }
    }
}

pub fn display_unique_instructions(unique_instr_set : &HashSet<u8>) {
    println!("Displaying unique instructions which have been encountered: ");
    for instr in unique_instr_set {
        println!("{:#01x}", instr);
    }
}