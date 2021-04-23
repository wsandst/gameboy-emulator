mod renderer;

extern crate emulator_core;
use emulator_core::emulator;
use emulator_core::debugger;

use std::time::{Duration, Instant};

const SLEEP_TIME_NS : u64 = 1_000_000_000 / 60;

const RENDERER_ENABLED : bool = true;
const PRINT_FRAMERATE : bool = true;
const KEEP_60_FPS : bool = true;

fn main() {
    // Create emulator and load ROM
    let mut emulator = emulator::Emulator::new();
    //emulator.memory.rom.read_from_file("roms/blargg/cpu_instrs.gb");
    emulator.memory.rom.load_from_file("roms/tetris.gb");
    //debugger::debug(&mut emulator);

    if RENDERER_ENABLED 
    {
        // Create an instance of Renderer, which starts a window
        let mut renderer = renderer::Renderer::new();
        let mut frame_count : u32 = 0;

        // Main game loop
        loop 
        {  
            let now = Instant::now();
            // Cycle the emulator until a draw is requested
            emulator.run_until_draw();

            // Render emulator bitmap
            renderer.set_screen_buffer(&mut emulator.screen.bitmap);
            renderer.render();

            // Handle input
            let exit = renderer.input(&mut emulator);
            if exit {
                break;
            }

            frame_count += 1;
            // Sleep to keep the proper framerate
            let frametime = now.elapsed().as_nanos() as u64;
            if KEEP_60_FPS && !renderer.speed_up && frametime < SLEEP_TIME_NS {
                std::thread::sleep(Duration::from_nanos(SLEEP_TIME_NS-frametime));
            }
            if PRINT_FRAMERATE && (frame_count % 10 == 0) {
                println!("Frame took {} ms", now.elapsed().as_millis());
            }
        }
    }
}