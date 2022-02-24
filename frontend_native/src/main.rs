mod renderer;
mod sound;

#[macro_use]
extern crate bmp;

extern crate emulator_core;
use emulator_core::{emulator, emulator::FrontendEvent, debugger};

use clap::{Arg};
use std::fs;

/// Run a SDL2 frontend for the Gameboy Emulator
fn main() {
    // Use clap to parse command line arguments
    let matches = clap::App::new("CorrodedBoy SDL2")
    .version("1.0")
    .author("William Sandström")
    .about("A Gameboy Emulator written in Rust")
    .arg(Arg::new("filename")
         .about("Select a ROM file to load")
         .required(true)
         .value_name("ROMFILE"))
    .arg(Arg::new("savefile")
         .about("Select a savefile (.save) to load")
         .short('s')
         .long("savefile")
         .takes_value(true)
         .value_name("SAVEFILE"))
    .arg(Arg::new("bootrom")
         .about("Select a bootrom to use. Not required.")
         .short('b')
         .long("bootrom")
         .takes_value(true)
         .value_name("BOOTROMFILE"))
    .arg(Arg::new("audiosync")
        .about("Select audio syncing strategy.")
        .long("audiosync")
        .takes_value(true)
        .value_name("STRATEGY")
        .possible_values(&["modfreq", "skipframes", "none"])
        .default_value("modfreq"))
    .arg(Arg::new("noaudio")
         .about("Disable audio")
         .short('a')
         .long("noaudio"))
    .arg(Arg::new("debugger")
         .about("Use CPU Debugger")
         .short('d')
         .long("debugger"))
    .get_matches();

    let mut emulator = emulator::Emulator::new();

    // Optionally load bootrom if flag is sent in
    if let Some(i) = matches.value_of("bootrom") {
        emulator.enable_bootrom();
        emulator.memory.rom.load_bootrom_from_file(i);
    }

    // Load ROM file
    if let Some(i) = matches.value_of("filename") {
        emulator.memory.rom.load_from_file(i);
    }

    // Load and deserialize emulator from provided file
    if let Some(i) = matches.value_of("savefile") {
        let bytes = fs::read(i).expect("Unable to read file");
        emulator = emulator::Emulator::deserialize(&bytes);
    }

    // Start debugger if requested
    if matches.is_present("debugger") {
        debugger::debug(&mut emulator);
    };

    // Create an instance of Renderer, which starts a window
    let mut renderer = renderer::Renderer::new();

    // Set renderer audio syncing strategy
    if let Some(i) = matches.value_of("audiosync") {
        renderer.audio_sync_strategy = match i {
            "modfreq" => renderer::AudioSyncStrategy::ModulateFrequency,
            "skipframes" => renderer::AudioSyncStrategy::SkipFrames,
            _ => renderer::AudioSyncStrategy::None,        
        }
    }

    renderer.sound_enabled = !matches.is_present("noaudio");

    run_emulator(&mut emulator, &mut renderer);
}

/// Run the SDL2 emulator frontend
fn run_emulator(emulator : &mut emulator::Emulator, renderer: &mut renderer::Renderer) {
    loop 
    {  
        // Cycle the emulator until a frontend event is requested
        match emulator.run_until_frontend_event() {
            // Render the emulator bitmap to the screen
            FrontendEvent::Render => {
                renderer.set_screen_buffer(&mut emulator.screen.bitmap);
                //renderer.set_screen_buffer(&mut debugger::gpu_state_dump(&mut emulator));
                renderer.render();
                // Handle input
                let exit = renderer.input(emulator);
                if exit {
                    break;
                }
                renderer.sleep_to_sync_video();
            }
            // Handle sound event
            FrontendEvent::QueueSound => {
                renderer.queue_sound(emulator);
            }
        }
    }
}