/// Implements rendering of a bitmap to the screen, using SDL2

extern crate sdl2; 
extern crate emulator_core;
use emulator_core::emulator;
use super::sound;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{Duration, Instant};

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;

const SCREEN_WIDTH: usize = GB_SCREEN_WIDTH*3;
const SCREEN_HEIGHT: usize = GB_SCREEN_WIDTH*3;

const SOUND_ENABLED : bool = true;
const PRINT_FRAMERATE : bool = true;

// VSYNC or sync to audio. Audio recommended to reduce gaps
const VIDEO_SYNC : bool = false;
const SOUND_SYNC : bool = true;

const SLEEP_TIME_NS : i64 = 1_000_000_000 / 60;
const SLEEP_TIME_SOUND_NS : i64 = (1_000_000_000 as f64 / (48000 as f64 / 2048 as f64)) as i64;

// Struct which contains the render state and various render methods
pub struct Renderer
{
    // SDL2 related
    screen_texture: sdl2::render::Texture,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    sound_player: sound::SoundPlayer,
    // FPS counting
    frame_counter: u32,
    frame_timer: Instant,
    avg_frametime: u64,
    sound_timer: Instant,
    // Options
    pub speed_up: bool,
}

impl Renderer
{
    pub fn new() -> Renderer
    {
        
        let sdl_context = sdl2::init().unwrap();

        // Setup bitmap rendering and window
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Rust Gameboy Emulator", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();
     
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        canvas.present();

        let texture_creator =  canvas.texture_creator();
        let texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, GB_SCREEN_WIDTH as u32, GB_SCREEN_HEIGHT as u32).unwrap();
    
        let event_pump = sdl_context.event_pump().unwrap();

        // Setup sound player
        let audio_subsystem = sdl_context.audio().unwrap();
        let sound_player = sound::SoundPlayer::new(audio_subsystem);
        sound_player.device.resume();
    
        return Renderer {
            speed_up: false, 
            screen_texture: texture, 
            canvas: canvas, 
            event_pump: event_pump, 
            sound_player: sound_player,
            frame_counter: 0,
            frame_timer : Instant::now(),
            avg_frametime: 0,
            sound_timer: Instant::now(),
        };
    }

    // Render a frame
    pub fn render(&mut self)
    {
        self.canvas.clear();
        self.canvas.copy(&self.screen_texture, None, Some(sdl2::rect::Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))).unwrap();
        self.canvas.present();
        self.frame_counter += 1;
    }
    
    pub fn sleep_to_sync_video(&mut self) {
        let frame_time = self.frame_timer.elapsed().as_nanos() as i64;
        if VIDEO_SYNC {
            // Sleep to keep the proper framerate
            let sleep_time: i64 = SLEEP_TIME_NS-frame_time;
            if !self.speed_up && sleep_time > 0 {
                std::thread::sleep(Duration::from_nanos(sleep_time as u64));
            }
        }
        if PRINT_FRAMERATE && (self.frame_counter % 10 == 0) {
            println!("Frame took {} ms", self.avg_frametime / 10);
            self.avg_frametime = 0;
        }
        self.avg_frametime += self.frame_timer.elapsed().as_millis() as u64;
        self.frame_timer = Instant::now();
    }

    pub fn sleep_to_sync_sound(&mut self) {
        if SOUND_SYNC {
            let sound_time = self.sound_timer.elapsed().as_nanos() as u64 as i64;
            let mut sleep_time: i64 = SLEEP_TIME_SOUND_NS-sound_time;
            if self.sound_player.device.size() < 12288 {
                sleep_time = -1;
            }
            if sleep_time > 0 {
                std::thread::sleep(Duration::from_nanos(sleep_time as u64));
            }
            //println!("Sound took {} ms", self.sound_timer.elapsed().as_millis());
            self.sound_timer = Instant::now();
        }
    }


    // Set the screen texture to a buffer array of size GB_HEIGHT*GB_WIDTH*3
    pub fn set_screen_buffer(&mut self, buffer : &mut [u8])
    {
        self.screen_texture.with_lock(None, |tbuffer: &mut [u8], _| {
            tbuffer.copy_from_slice(buffer);
        }).unwrap();
    }

    /*pub fn update_screen_buffer(&mut self, buffer: &mut [u8]) {
        self.screen_texture.update(None::<Rect>, buffer, 256*3).unwrap();
    }*/

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
                        Some(Keycode::Return) =>    emulator.press_key(emulator::KeyPress::Start),
                        Some(Keycode::Backspace) => emulator.press_key(emulator::KeyPress::Select),
                        Some(Keycode::W) =>         emulator.press_key(emulator::KeyPress::Up),
                        Some(Keycode::S) =>         emulator.press_key(emulator::KeyPress::Down),
                        Some(Keycode::A) =>         emulator.press_key(emulator::KeyPress::Left),
                        Some(Keycode::D) =>         emulator.press_key(emulator::KeyPress::Right),
                        Some(Keycode::Up) =>        emulator.press_key(emulator::KeyPress::Up),        
                        Some(Keycode::Down) =>      emulator.press_key(emulator::KeyPress::Down),
                        Some(Keycode::Z) =>         emulator.press_key(emulator::KeyPress::A),
                        Some(Keycode::X) =>         emulator.press_key(emulator::KeyPress::B),
                        Some(Keycode::Space) =>     emulator.press_key(emulator::KeyPress::A),
                        Some(Keycode::LShift) =>    emulator.press_key(emulator::KeyPress::B),
                        Some(Keycode::LCtrl) =>     self.speed_up = !self.speed_up,
                        _ => { }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Return) =>    emulator.clear_key(emulator::KeyPress::Start),
                        Some(Keycode::Backspace) => emulator.clear_key(emulator::KeyPress::Select),
                        Some(Keycode::W) =>         emulator.clear_key(emulator::KeyPress::Up),
                        Some(Keycode::S) =>         emulator.clear_key(emulator::KeyPress::Down),
                        Some(Keycode::A) =>         emulator.clear_key(emulator::KeyPress::Left),
                        Some(Keycode::D) =>         emulator.clear_key(emulator::KeyPress::Right),
                        Some(Keycode::Up) =>        emulator.clear_key(emulator::KeyPress::Up),        
                        Some(Keycode::Down) =>      emulator.clear_key(emulator::KeyPress::Down),
                        Some(Keycode::Z) =>         emulator.clear_key(emulator::KeyPress::A),
                        Some(Keycode::X) =>         emulator.clear_key(emulator::KeyPress::B),
                        Some(Keycode::Space) =>     emulator.clear_key(emulator::KeyPress::A),
                        Some(Keycode::LShift) =>    emulator.clear_key(emulator::KeyPress::B),
                        _ => { }
                    }
                }
                _ => {}
            }
        }
        return false;
    }

    pub fn queue_sound(&mut self, queue: &Vec<i16>) {
        if self.sound_player.device.size() == 0 {
            println!("Audio gap!");
        }
        if SOUND_ENABLED && !self.speed_up {
            self.sound_player.device.queue(queue);
        }
    }
}