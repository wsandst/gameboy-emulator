mod renderer;
mod emulator;
mod debugger;

use std::time::Duration;

const RENDERER_ENABLED : bool = false;

/*
Tests:
    Blargh:
        Passed:
            01-special.gb
            03-op sp,hl.gb
            04-op r,imm.gb
            05-op rp.gb
            06-ld r,r.gb
            07-jr,jp,call,ret,rst.gb
            08-misc instrs.gb
            09-op r,r.gb
            10-bit ops.g
            11-op a,(hl).gbb
        Missing instr:
            02-interrupts.gb (Interupts not really implemented)

*/

fn main() {
    let mut emulator = emulator::Emulator::new();
    //emulator.memory.rom.read_from_file("roms/cpu_instrs/daa.gb");
    emulator.memory.rom.read_from_file("roms/cpu_instrs/individual/02-interrupts.gb");
    //emulator.memory.rom.read_from_file("roms/cpu_instrs/cpu_instrs.gb");
    debugger::debug(&mut emulator);
    //emulator.run();

    if RENDERER_ENABLED 
    {
        // Create an instance of Renderer, which starts a window
        let mut renderer = renderer::Renderer::new();

        let mut buffer : [u8; 160*144*3] = [0; 160*144*3];

        for i in 0..(buffer.len()/3) {
            buffer[i*3+0] = 0;
            buffer[i*3+1] = 255;
            buffer[i*3+2] = 255;
        }

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
