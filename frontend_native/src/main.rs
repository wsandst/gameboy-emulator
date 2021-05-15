mod renderer;
mod sound;

extern crate emulator_core;
use emulator_core::{emulator, emulator::FrontendEvent};
use clap::{Arg};

fn main() {
    // Use clap to parse command line arguments
    let matches = clap::App::new("corrosive-boy")
    .version("0.1")
    .author("William Sandström")
    .about("A Gameboy Emulator written in Rust")
    .arg(Arg::new("filename")
         .about("Select a ROM file to load")
         .required(true))
    .arg(Arg::new("bootrom")
         .about("Select a bootrom to use. Not required.")
         .short('b')
         .long("bootrom")
         .takes_value(true))
    .arg(Arg::new("noaudio")
         .about("Disable audio")
         .short('a')
         .long("noaudio"))
    .get_matches();

    let use_bootrom = matches.is_present("bootrom");
    let mut emulator = emulator::Emulator::new(use_bootrom);

    // Optionally load bootrom if flag is sent in
    if let Some(i) = matches.value_of("bootrom") {
        emulator.memory.rom.load_bootrom_from_file(i);
    }

    // Load ROM file
    if let Some(i) = matches.value_of("filename") {
        emulator.memory.rom.load_from_file(i);
    }

    // Create an instance of Renderer, which starts a window
    let mut renderer = renderer::Renderer::new();
    run_emulator(&mut emulator, &mut renderer);
}

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
                renderer.queue_sound(emulator.get_sound_queue());
            }
        }
    }
}