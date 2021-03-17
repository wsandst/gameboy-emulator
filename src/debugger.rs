use crate::emulator;
use text_io::try_read;
use std::io::{self, Write};

#[derive(PartialEq)]
enum CommandType {
    Step(u32),
    PrintRegs,
    PrintMem,
    PrintSteps,
    ToggleVerbose,
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
        "steps" | "printsteps" | "stepcount" => { CommandType::PrintSteps}
        "regs" | "r" | "printregs" => { CommandType::PrintRegs}
        "mem" | "m" | "printmem" => { CommandType::PrintMem} 
        "verbose" | "v" | "toggleverbose" => { CommandType::ToggleVerbose} 
        _ => { CommandType::None}
    }
}

pub fn debug(em : &mut emulator::Emulator) {
    print!("Debugging Gameboy ROM {}\n", em.memory.rom.filename);
    let mut cmd = CommandType::None;
    let mut verbose : bool = false;
    let mut step_counter : u32 = 0;
    while cmd != CommandType::Quit {
        cmd = get_input();
        match cmd {
            CommandType::Step(step_size) => {step(em, step_size); step_counter += step_size;}
            CommandType::PrintRegs => {em.cpu.regs.debug_display();}
            CommandType::PrintMem => {em.cpu.regs.debug_display();}
            CommandType::PrintSteps => {println!("Current step count: {}", step_counter);}
            CommandType::ToggleVerbose => {verbose = !verbose;}
            CommandType::None => {println!("Unknown command. Try again")}
            CommandType::Quit => { println!("Exiting program")}
        }
    }
}

pub fn step(em: &mut emulator::Emulator, step_size : u32) {
    for _i in 0..step_size {
        em.step();
        let next = em.memory.read_byte(em.cpu.regs.pc);
        println!("Instr: {:#01x} @ pc = {1:#01x} ({1})", next, em.cpu.regs.pc);
    }
}
