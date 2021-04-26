[![Build Status](https://travis-ci.com/wsandst/gameboy-emulator.svg?branch=main)](https://travis-ci.com/wsandst/gameboy-emulator)
# gameboy-emulator
Gameboy Emulator written in Rust. The emulator has two available frontends: one native using SDL2, and a web frontend (through WASM) written in Javascript. The emulator is still in development.  
  
![Tetris example](https://i.ibb.co/C1MHRbf/tetris2.png)
## Functionality
Tetris is currently completely playable in both the native frontend and the web frontend. Other games have issues, though. The GPU is lacking in functionality.  
### Implemented parts:
* Mostly complete CPU implementation, lacking certain cycle accuracies. Passes the blargg cpu_instrs and instr_timing test ROMs, but not the mem_timing.
* Interrupts
* Timer
* Joypad input
* Buggy GPU, can run Tetris fine
* CPU debugging tool
* Native frontend
* Web frontend

## Todo:
- [ ] Improve tile graphics GPU drawing
- [ ] More advanced GPU sprite functionality
- [ ] GPU Optimizations
- [ ] Pass GPU tests
- [ ] CPU complete cycle accuracy
- [ ] Pass all blargg tests
- [ ] Sound core implementation
- [ ] Sound frontend integration
- [ ] Savestates using Serialization

## CPU Tests
Passing blargg cpu_instrs and instr_timing. Large refactor needed to pass mem_timing.

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