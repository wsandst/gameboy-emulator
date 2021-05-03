mod renderer;
mod sound;

extern crate emulator_core;
use emulator_core::{emulator, emulator::FrontendEvent};

const RENDERER_ENABLED : bool = true;

fn main() {
    // Create emulator and load ROM
    let mut emulator = emulator::Emulator::new();
    //emulator.memory.rom.read_from_file("roms/blargg/cpu_instrs.gb");
    emulator.memory.rom.load_from_file("roms/games/tetris.gb");
    //debugger::debug(&mut emulator);

    if RENDERER_ENABLED 
    {
        // Create an instance of Renderer, which starts a window
        let mut renderer = renderer::Renderer::new();
        // Main loop
        loop 
        {  
            // Cycle the emulator until a frontend event is requested
            match emulator.run_until_frontend_event() {
                // Render the emulator bitmap to the screen
                FrontendEvent::Render => {
                    renderer.set_screen_buffer(&mut emulator.screen.bitmap);
                    renderer.render();
                    // Handle input
                    let exit = renderer.input(&mut emulator);
                    if exit {
                        break;
                    }
                    renderer.sleep_to_sync_video();
                }
                // Handle sound event
                FrontendEvent::QueueSound => {
                    renderer.queue_sound(emulator.get_sound_queue());
                    renderer.sleep_to_sync_sound();
                }
            }
        }
    }
}