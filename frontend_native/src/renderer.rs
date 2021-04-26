/// Implements rendering of a bitmap to the screen, using SDL2

extern crate sdl2; 
extern crate emulator_core;
use emulator_core::emulator;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;

const SCREEN_WIDTH: usize = GB_SCREEN_WIDTH*3;
const SCREEN_HEIGHT: usize = GB_SCREEN_WIDTH*3;

// Struct which contains the render state and various render methods
pub struct Renderer
{
    pub speed_up : bool,
    screen_texture : sdl2::render::Texture,
    canvas : sdl2::render::Canvas<sdl2::video::Window>,
    event_pump : sdl2::EventPump
}

impl Renderer
{
    pub fn new() -> Renderer
    {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
     
        let window = video_subsystem.window("Rust Gameboy Emulator", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();
     
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();

        let texture_creator =  canvas.texture_creator();
        let texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, GB_SCREEN_WIDTH as u32, GB_SCREEN_HEIGHT as u32).unwrap();
    
        let event_pump = sdl_context.event_pump().unwrap();
    
        return Renderer {speed_up: false, screen_texture: texture, canvas: canvas, event_pump: event_pump};
    }

    // Render a frame
    pub fn render(&mut self)
    {
        self.canvas.clear();
        self.canvas.copy(&self.screen_texture, None, Some(sdl2::rect::Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))).unwrap();
        self.canvas.present();
    }

    // Set the screen texture to a buffer array of size GB_HEIGHT*GB_WIDTH*3
    pub fn set_screen_buffer(&mut self, buffer : &mut [u8])
    {
        self.screen_texture.with_lock(None, |tbuffer: &mut [u8], _| {
            tbuffer.copy_from_slice(buffer);
        }).unwrap();
    }

    pub fn update_screen_buffer(&mut self, buffer: &mut [u8]) {
        self.screen_texture.update(None::<Rect>, buffer, 256*3).unwrap();
    }
    pub fn input(&mut self, emulator: &mut emulator::Emulator) -> bool
    {
        for event in self.event_pump.poll_iter() {
            match event {
                // Exit program
                Event::Quit {..} |
                Event::KeyDown { 
                    keycode: Some(Keycode::Escape), .. } => {
                        return true;
                },
                Event::KeyDown { keycode, .. } => { 
                    match keycode {
                        Some(Keycode::Return) => emulator.press_key(emulator::KeyPress::Start),
                        Some(Keycode::Backspace) => emulator.press_key(emulator::KeyPress::Select),
                        Some(Keycode::W) => emulator.press_key(emulator::KeyPress::Up),
                        Some(Keycode::S) => emulator.press_key(emulator::KeyPress::Down),
                        Some(Keycode::A) => emulator.press_key(emulator::KeyPress::Left),
                        Some(Keycode::D) => emulator.press_key(emulator::KeyPress::Right),
                        Some(Keycode::Up) => emulator.press_key(emulator::KeyPress::Up),
                        Some(Keycode::Down) => emulator.press_key(emulator::KeyPress::Down),
                        Some(Keycode::Z) => emulator.press_key(emulator::KeyPress::A),
                        Some(Keycode::X) => emulator.press_key(emulator::KeyPress::B),
                        Some(Keycode::Space) => emulator.press_key(emulator::KeyPress::A),
                        Some(Keycode::LShift) => emulator.press_key(emulator::KeyPress::B),

                        Some(Keycode::LCtrl) => self.speed_up = !self.speed_up,
                        _ => { }
                    }
                }
                _ => {}
            }
        }
        return false;
    }
}