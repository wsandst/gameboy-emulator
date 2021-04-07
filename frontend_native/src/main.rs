mod renderer;

extern crate emulator_core;
use emulator_core::emulator;
use emulator_core::debugger;

use std::time::Duration;

const RENDERER_ENABLED : bool = true;

fn main() {
    let mut emulator = emulator::Emulator::new();
    //println!("Test");
    emulator.memory.rom.read_from_file("roms/blargg/cpu_instrs.gb");
    //debugger::debug(&mut emulator);
    emulator.run();

    if RENDERER_ENABLED 
    {
        // Create an instance of Renderer, which starts a window
        let mut renderer = renderer::Renderer::new();

        let mut buffer = emulator.screen.bitmap; //[u8; 160*144*3] = [0; 160*144*3];

        /*for i in 0..(buffer.len()/3) {
            buffer[i*3+0] = 0;
            buffer[i*3+1] = 255;
            buffer[i*3+2] = 255;
        }*/

        renderer.set_screen_buffer(&mut buffer);

        let mut frame_count : u32 = 0;
        // Main game loop
        loop 
        {  
            renderer.render();
            let exit = renderer.input();
            if exit {
                break;
            }

            frame_count += 1;
            // Sleep to keep the proper framerate. Will be dependant on the emulator
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
