

use sdl2::audio::{AudioSpecDesired, AudioQueue};
use sdl2::AudioSubsystem;

pub struct SoundPlayer {
    pub device: AudioQueue<f32>,
}

impl SoundPlayer {
    pub fn new(audio_subsystem : AudioSubsystem) -> SoundPlayer {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: Some(1024),     // default sample size
        };
        let device = audio_subsystem.open_queue::<f32, _>(None, &desired_spec).unwrap();
        // Start with buffer of no sound
        device.queue(&vec![0 as f32; 16384]);

        SoundPlayer { device: device}
    }

}