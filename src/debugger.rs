use crate::emulator;
use text_io::try_read;
use std::io::{self, Write};

#[derive(PartialEq)]
enum CommandType {
    Step(u32),
    PrintRegs,
    PrintMem,
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
    while cmd != CommandType::Quit {
        cmd = get_input();
        match cmd {
            CommandType::Step(step_size) => {step(em, step_size);}
            CommandType::PrintRegs => {em.cpu.regs.debug_display();}
            CommandType::PrintMem => {em.cpu.regs.debug_display();}
            CommandType::ToggleVerbose => {verbose = !verbose;}
            CommandType::None => {println!("Unknown command. Try again")}
            CommandType::Quit => { println!("Exiting program")}
        }
    }
}

pub fn step(em: &mut emulator::Emulator, step_size : u32) {
    for _i in 0..step_size {
        println!("Instr: {:#01x} @ pc = {:#01x} ({})", em.step(), em.cpu.regs.pc, em.cpu.regs.pc);
    }
}
