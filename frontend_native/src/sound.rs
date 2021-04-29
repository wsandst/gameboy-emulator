

use sdl2::audio::{AudioSpecDesired, AudioQueue};
use sdl2::AudioSubsystem;

pub struct SoundPlayer {
    pub device: AudioQueue<i16>,
}

impl SoundPlayer {
    pub fn new(audio_subsystem : AudioSubsystem) -> SoundPlayer {
        let desired_spec = AudioSpecDesired {
            freq: Some(48_000),
            channels: Some(2), // mono
            samples: None,     // default sample size
        };
        let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();

        SoundPlayer { device: device}
    }

}