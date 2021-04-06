# gameboy-emulator
Gameboy Emulator written in Rust. Uses SDL2 for graphics and input. 
Currently only the CPU is implemented.
## Todo:
- [x] General CPU Instructions
- [X] Automatic blargg testing
- [X] CPU Interrupts
- [X] General CPU Cycle Timings
- [ ] Memory cycle accuracy
- [ ] Pass all blargg tests
- [ ] GPU General
- [ ] GPU Background
- [ ] GPU Sprites
- [ ] GPU Optimizations
- [ ] Native Screen Rendering
- [ ] Javascript Screen Rendering
- [ ] I/O

## CPU Tests
Passing blargg cpu_instrs and instr_timing. Large refactor needed to pass mem_timing.

## Frontend options
### Native
Uses SDL2 in Rust for Graphics, Input and Sound.

### Web
Compiles the emulator core to WASM and uses wasm-pack to create a NPM module interface to the emulator
This module is then used to create a Node.js frontend.

## Build instructions
### Native
`cargo build --release --package gb-emulator-native`  
Run:  
`./target/release/gb-emulator-native`

### Web
`cd frontend_web`  
`wasm-pack build`  
Run:  
`cd frontend_web/site`   
`npm install`  
`npm run serve`


