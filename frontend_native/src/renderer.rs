/// Implements an SDL2 frontend for the emulator

extern crate sdl2; 
use emulator_core::{emulator, debug_helper};
use super::debugger;
use super::sound;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use std::time::{Duration, Instant};
use std::fs;
use chrono::prelude;
use bmp::{Image, Pixel};

// Various settings

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;

const SCREEN_UPSCALE_FACTOR: usize = 4;

const SCREEN_WIDTH: usize = GB_SCREEN_WIDTH*SCREEN_UPSCALE_FACTOR;
const SCREEN_HEIGHT: usize = GB_SCREEN_HEIGHT*SCREEN_UPSCALE_FACTOR;

const PRINT_FRAMERATE : bool = false;
const PRINT_AUDIO_INFO: bool = false;

const SLEEP_TIME_60FPS_NS : i64 = 1_000_000_000 / 60;

/// Represents various audio syncing strategies
#[derive(Copy, Clone, PartialEq)]
pub enum AudioSyncStrategy {
    SkipFrames,
    ModulateFrequency,
    None
}

// Represents the emulator frontend render state and various render methods
pub struct Renderer
{
    // SDL2 related
    screen_texture: sdl2::render::Texture,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    sound_player: sound::SoundPlayer,
    // FPS counting
    frame_counter: u32,
    audio_counter: usize,
    frame_timer: Instant,
    audio_timer: Instant,
    speedup_timer: Instant,
    avg_frametime: u64,
    sleep_time_ns : i64,
    // Options
    pub speed_up: bool,
    pub paused: bool,
    pub sound_enabled: bool,
    pub audio_sync_strategy : AudioSyncStrategy,
}

impl Renderer
{
    /// Create a new SDL2 emulator frontend
    pub fn new() -> Renderer
    {
        
        let sdl_context = sdl2::init().unwrap();

        // Setup bitmap rendering and window
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("CorrodedBoy - Gameboy Emulator", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
     
        let mut canvas = window.into_canvas().build().unwrap();
        //println!("VSYNC:  {:?}", video_subsystem.gl_get_swap_interval());
        //video_subsystem.gl_set_swap_interval(-1).unwrap();
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
            paused: false,
            sound_enabled: true,
            screen_texture: texture, 
            canvas: canvas, 
            event_pump: event_pump, 
            sound_player: sound_player,
            frame_counter: 0,
            audio_counter: 0,
            frame_timer : Instant::now(),
            audio_timer: Instant::now(),
            speedup_timer: Instant::now(),
            avg_frametime: 0,
            sleep_time_ns: SLEEP_TIME_60FPS_NS,
            audio_sync_strategy: AudioSyncStrategy::ModulateFrequency,
        };
    }

    /// Render a frame
    pub fn render(&mut self)
    {
        // Skip frames if we are sped up
        if self.speed_up {
            if self.speedup_timer.elapsed().as_millis() < 16 {
                return;
            }
            else {
                self.speedup_timer = Instant::now();
            }
        }
        
        self.canvas.clear();
        self.canvas.copy(&self.screen_texture, None, Some(sdl2::rect::Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))).unwrap();
        self.canvas.present();
        self.frame_counter += 1;
    }
    
    /// Sleep to keep a constant 60 FPS framerate
    pub fn sleep_to_sync_video(&mut self) {
        let frame_time = self.frame_timer.elapsed().as_nanos() as i64;
        // Sleep to keep the proper framerate
        let sleep_time: i64 = self.sleep_time_ns-frame_time;
        if (self.audio_sync_strategy != AudioSyncStrategy::SkipFrames || !self.sound_enabled)
             && !self.speed_up && sleep_time > 0 {
            spin_sleep::sleep(Duration::from_nanos(sleep_time as u64));
        }
        self.avg_frametime += self.frame_timer.elapsed().as_micros() as u64;
        self.frame_timer = Instant::now();
        if PRINT_FRAMERATE && (self.frame_counter % 10 == 0) {
            let frame_ms = self.avg_frametime / 10000;
            let fps = 1.0 / (self.avg_frametime as f64 / 10000000.0);
            println!("Frame took {} ms, {} fps", frame_ms, fps);
            self.avg_frametime = 0;
        }
    }

    /// Set the screen texture to a buffer array of size GB_HEIGHT*GB_WIDTH*3
    pub fn set_screen_buffer(&mut self, buffer : &mut [u8])
    {
        self.screen_texture.with_lock(None, |tbuffer: &mut [u8], _| {
            tbuffer.copy_from_slice(buffer);
        }).unwrap();
    }

    /// Check for user input and act accordingly
    /// 
    /// Returns true if the emulator should exit, otherwise false
    pub fn input(&mut self, emulator: &mut emulator::Emulator) -> bool
    {
        let mut take_screenshot = false;
        // Go through all input events
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
                        // Gameboy button presses
                        Some(Keycode::Return) =>    emulator.press_key(emulator::KeyPress::Start),
                        Some(Keycode::Backspace) => emulator.press_key(emulator::KeyPress::Select),
                        Some(Keycode::W) =>         emulator.press_key(emulator::KeyPress::Up),
                        Some(Keycode::S) =>         emulator.press_key(emulator::KeyPress::Down),
                        Some(Keycode::A) =>         emulator.press_key(emulator::KeyPress::Left),
                        Some(Keycode::D) =>         emulator.press_key(emulator::KeyPress::Right),
                        Some(Keycode::Up) =>        emulator.press_key(emulator::KeyPress::Up),        
                        Some(Keycode::Down) =>      emulator.press_key(emulator::KeyPress::Down),
                        Some(Keycode::Left) =>      emulator.press_key(emulator::KeyPress::Left),        
                        Some(Keycode::Right) =>     emulator.press_key(emulator::KeyPress::Right),
                        Some(Keycode::Z) =>         emulator.press_key(emulator::KeyPress::A),
                        Some(Keycode::X) =>         emulator.press_key(emulator::KeyPress::B),
                        Some(Keycode::Space) =>     emulator.press_key(emulator::KeyPress::A),
                        Some(Keycode::LShift) =>    emulator.press_key(emulator::KeyPress::B),
                        // Various emulator controls
                        Some(Keycode::P) =>         emulator.paused = !emulator.paused,
                        Some(Keycode::O) =>         self.sound_enabled = !self.sound_enabled,
                        Some(Keycode::LCtrl) =>     self.speed_up = !self.speed_up,
                        Some(Keycode::F1) =>        Renderer::save_emulator(emulator),
                        Some(Keycode::F2) =>        take_screenshot = true,
                        // Debugging controls
                        Some(Keycode::F3) =>        debug_helper::save_gpu_state_to_file(emulator, "debug.bmp"),
                        Some(Keycode::F4) =>        debugger::debug(emulator),
                        _ => { }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    match keycode {
                        // Gameboy button releases
                        Some(Keycode::Return) =>    emulator.clear_key(emulator::KeyPress::Start),
                        Some(Keycode::Backspace) => emulator.clear_key(emulator::KeyPress::Select),
                        Some(Keycode::W) =>         emulator.clear_key(emulator::KeyPress::Up),
                        Some(Keycode::S) =>         emulator.clear_key(emulator::KeyPress::Down),
                        Some(Keycode::A) =>         emulator.clear_key(emulator::KeyPress::Left),
                        Some(Keycode::D) =>         emulator.clear_key(emulator::KeyPress::Right),
                        Some(Keycode::Up) =>        emulator.clear_key(emulator::KeyPress::Up),        
                        Some(Keycode::Down) =>      emulator.clear_key(emulator::KeyPress::Down),
                        Some(Keycode::Left) =>      emulator.clear_key(emulator::KeyPress::Left),        
                        Some(Keycode::Right) =>     emulator.clear_key(emulator::KeyPress::Right),
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
        if take_screenshot {
            self.save_screenshot(emulator);
        }
        return false;
    }

    /// Queue sound from the emulator
    pub fn queue_sound(&mut self, emulator : &mut emulator::Emulator) {
        if self.sound_enabled && !self.speed_up {
            self.audio_counter += 1;
            if self.sound_player.device.size() == 0 {
                println!("Warning: Audio gap!");
                self.sound_player.device.queue(&vec![0 as f32; 6144]);
                self.sound_player.sound_syncer.skip_next_frame();
            }
            let sound_queue = emulator.get_sound_queue();
            self.sound_player.device.queue(sound_queue);
            // Debugging helper
            if PRINT_AUDIO_INFO && self.audio_counter % 60 == 0 {
                println!("Current sample rate: {}", self.sound_player.sound_syncer.current_output_rate);
                println!("Current queue size: {}",self.sound_player.device.size());
                println!("Audio Frame: {}",self.audio_counter);
            }
            match self.audio_sync_strategy {
                AudioSyncStrategy::ModulateFrequency => {
                    let new_samplerate = self.sound_player.get_new_samplerate();
                    emulator.set_sound_output_sample_rate(new_samplerate);
                }
                AudioSyncStrategy::SkipFrames => {
                    let sample_count : i64 = sound_queue.len() as i64;
                    let audio_time = self.audio_timer.elapsed().as_nanos() as i64;
                    // Remove the division by two to make mono
                    let sleep_time : i64 = ((1_000_000_000 * sample_count / 2) / 48000) - audio_time;
                    // Sleep to keep the proper audio rate
                    if !self.speed_up && sleep_time > 0 {
                        spin_sleep::sleep(Duration::from_nanos(sleep_time as u64));
                    }
                    self.audio_timer = Instant::now();
                }
                AudioSyncStrategy::None => {

                }
            }
        }
    }

    /// Save the emulator state as a savefile.
    /// The savefile will be named after the loaded ROM and the current time and
    /// placed in the current working directory.
    pub fn save_emulator(emulator : &mut emulator::Emulator) {
        let save_bincode = emulator.serialize();
        let filename = format!("{}-{}.save", emulator.get_rom_name(), prelude::Utc::now().format("%Y-%m-%dT%H:%M:%S"));
        fs::write(&filename, save_bincode).expect("Unable to write file");
        println!("Saved emulator state to savefile \"{}\"", &filename);
    }

    /// Save the emulator screen as a screenshot.
    /// The screenshot is placed in the current working directory.
    pub fn save_screenshot(&self, emulator : &mut emulator::Emulator) {
        let filename = format!("screenshot-{}-{}.bmp", emulator.get_rom_name(), prelude::Utc::now().format("%Y-%m-%dT%H:%M:%S"));
        let mut img = Image::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
        let pixels = self.canvas.read_pixels(None, PixelFormatEnum::RGB24).unwrap();
        for (x, y) in img.coordinates() {
            let i = (y as usize)*SCREEN_WIDTH + x as usize;
            img.set_pixel(x, y, px!(pixels[i*3+0], pixels[i*3+1], pixels[i*3+2]));
        }
        let _ = img.save(&filename);
        println!("Saved screenshot to file \"{}\"", &filename);
    }
}