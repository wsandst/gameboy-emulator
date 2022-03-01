/// This file contains various functionality for
/// debugging this emulator. It integrates with the `debug_helper`
/// core functionality and allows for a smooth commandline debugging
/// experience.

use emulator_core::{emulator, debug_helper, debug_helper::CommandType};

// Use rustyline for a better commandline experience
// Allows for line history and more
use rustyline::Editor;
use rustyline::error::ReadlineError;

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
        // Stepping
        "step" | "s" => { 
            if arg_count > 1 {
                CommandType::Step(words[1].parse::<u64>().unwrap())
            } else {
                CommandType::Step(1)
            }
        }
        "run" => { CommandType::Run }
        // Adding breakpoints
        "breakpoint" | "br" => {
            if arg_count > 1 {
                let address = match debug_helper::parse_number(words[1]) {
                    Ok(range) => { range }
                    Err(error) => { 
                        return CommandType::Error(format!("Unable to parse the specified address, {}", error).to_string());
                    }
                };
                CommandType::Breakpoint(Some(address))
            } else {
                CommandType::Breakpoint(None)
            }
        }
        // Inspecting memory
        "regs" | "r" | "printregs" => { CommandType::PrintRegs}
        "mem" | "m" | "printmem" | "inspect" => { 
            if arg_count > 1 {
                let range = if arg_count > 2 {
                    // If a second argument is specified, print a range of memory
                    match debug_helper::parse_number(words[2]) {
                        Ok(range) => { range }
                        Err(error) => { 
                            return CommandType::Error(format!("Unable to parse the specified address, {}", error).to_string());
                        }
                    }
                }
                else {
                    1
                };
                match debug_helper::parse_number(words[1]) {
                    Ok(address) => { CommandType::PrintMem(address, range) }
                    Err(error) => { CommandType::Error(format!("Unable to parse the specified address, {}", error).to_string()) }
                }
            } else {
                CommandType::Error("Please specify a memory address to inspect".to_string())
            }
        } 
        // Print information
        "steps" | "printsteps" | "stepcount" => { CommandType::PrintSteps}
        // Toggle functionality
        "verbose" | "v" | "toggleverbose" => { CommandType::ToggleVerbose} 
        "instrtracking" | "it" | "trackinstr" | "trackunique" => {CommandType::ToggleInstrTracking}
        "unique" | "uniqueinstr" | "ui" | "listinstr" => {CommandType::PrintUniqueInstrs}
        "togglebreakpoints" | "tb" | "toggleb" | "tbreakpoints" | "tbreak" | "breakpoints" => {CommandType::ToggleBreakpoints}
        "state" | "completestate" => {CommandType::PrintEmulatorState}
        _ => { CommandType::Error("Unknown command specified".to_string())}
    }
}

/// Commandline tool for debugging an emulator. Allows for
/// stepping through the emulator and inspecting memory.
pub fn debug(em : &mut emulator::Emulator) {
    let mut state = debug_helper::DebugState::new();

    // Setup readlines history
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".emdebug.txt");

    let mut cmd = CommandType::None;
    print!("\nDebugging Gameboy ROM {}\n", em.memory.rom.filename);

    while cmd != CommandType::Quit {
        cmd = get_input(&mut rl);
        debug_helper::execute_debug_command(&cmd, em, &mut state);
    }

    rl.save_history(".emdebug.txt").unwrap();
}