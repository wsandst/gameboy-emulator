/// Implements rendering of a bitmap to the screen, using SDL2

extern crate sdl2; 

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
 
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 720;

// Struct which contains the render state and various render methods
pub struct Renderer
{
    canvas : sdl2::render::Canvas<sdl2::video::Window>,
    event_pump : sdl2::EventPump
}

impl Renderer
{
    pub fn new() -> Renderer
    {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
     
        let window = video_subsystem.window("Rust Gameboy Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();
     
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
    
        let event_pump = sdl_context.event_pump().unwrap();
    
        return Renderer {canvas: canvas, event_pump: event_pump};
    }

    pub fn render(&mut self)
    {
        self.canvas.set_draw_color(Color::RGB(255, 64, 255));
        self.canvas.clear();
        self.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    pub fn input(&mut self) -> bool
    {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { 
                    keycode: Some(Keycode::Escape), .. } => {
                        return true;
                },
                _ => {}
            }
        }
        return false;
    }
}