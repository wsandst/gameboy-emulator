/// Implements rendering of a bitmap to the screen, using SDL2

extern crate sdl2; 

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
 
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 720;

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;

// Struct which contains the render state and various render methods
pub struct Renderer
{
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
     
        let window = video_subsystem.window("Rust Gameboy Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
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
    
        return Renderer {screen_texture: texture, canvas: canvas, event_pump: event_pump};
    }

    // Render a frame
    pub fn render(&mut self)
    {
        self.canvas.clear();
        self.canvas.copy(&self.screen_texture, None, Some(sdl2::rect::Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT))).unwrap();
        self.canvas.present();
    }

    // Set the screen texture to a buffer array of size GB_HEIGHT*GB_WIDTH*3
    pub fn set_screen_buffer(&mut self, buffer : &mut [u8])
    {
        self.screen_texture.with_lock(None, |tbuffer: &mut [u8], _| {
            println!("T {}", tbuffer.len());
            tbuffer.copy_from_slice(buffer);
        }).unwrap();
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