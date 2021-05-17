[![Build Status](https://travis-ci.com/wsandst/gameboy-emulator.svg?branch=main)](https://travis-ci.com/wsandst/gameboy-emulator)
# Corrosive Boy, a Gameboy Emulator written in Rust
Gameboy Emulator written in Rust. The emulator has two available frontends: one native using SDL2, and a web frontend (through WASM) written in Javascript. The emulator is still in development.  
  
![Tetris example](https://i.ibb.co/C1MHRbf/tetris2.png)
## Functionality
**Tetris**, **Dr Mario** and **Super Mario Land**, **The Legend of Zelda: Links Awakening** have been tested and work quite well. Most parts of **Pokemon Red** work. The sound is currently lacking in functionality. 
### Implemented parts:
* Mostly complete CPU implementation, lacking certain cycle accuracies. Passes the blargg cpu_instrs and instr_timing test ROMs, but not the mem_timing.
* Interrupts
* Timer
* Joypad input
* Mostly complete GPU implementation, passes all parts of the acid2 test ROM except for one.
* Very rudimentary sound, in development. Currently only parts of the pulse channels are implemented.  
* Savestates using Serialization
* CPU debugging tool
* Native frontend
* Web frontend

## Todo:
- [X] Improve tile graphics GPU drawing
- [X] More advanced GPU sprite functionality
- [ ] GPU Optimizations
- [x] Partially pass acid2 GPU test
- [ ] Completely pass acid2 GPU test
- [ ] CPU complete cycle accuracy
- [ ] Pass all blargg tests
- [x] Partial pulse-wave sound implemention
- [ ] Sound core implementation
- [x] Sound frontend integration
- [ ] Improve sound/video syncing
- [X] Savestates using Serialization
- [X] Partial MBC3 support
- [ ] MBC3 Real-time clock support (for Pokemon Red)

## Test roms
Passing blargg cpu_instrs and instr_timing. Large refactor needed to pass mem_timing. 
Almost passes the acid2 GPU test, only fails internal window counter currently.

## Frontend options
### Native
Uses SDL2 in Rust for Graphics, Input and Sound.

### Web
Compiles the emulator core to WASM and uses wasm-pack to create a NPM module interface to the emulator.
This module is then used to create a Node.js frontend. Currently uses Canvas, but WebGL support is planned.

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

### Resources
https://www.youtube.com/watch?v=HyzD8pNlpwI  
https://gbdev.io/pandocs/  
https://gbdev.gg8.se/wiki/articles/Main_Page  
http://imrannazar.com/GameBoy-Emulation-in-JavaScript  
https://izik1.github.io/gbops/