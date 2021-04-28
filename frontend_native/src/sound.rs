

use sdl2::audio::{AudioSpecDesired, AudioQueue};
use sdl2::AudioSubsystem;

/*pub fn gen_wave(bytes_to_write: i32) -> Vec<i16> {
    // Generate a square wave
    let tone_volume = 1_000i16;
    let period = 48_000 / 256;
    let sample_count = bytes_to_write;
    let mut result = Vec::new();

    for x in 0..sample_count {
        result.push(if (x / period) % 2 == 0 {
            tone_volume
        } else {
            -tone_volume
        });
    }
    result
}*/

pub struct SoundPlayer {
    audio_subsystem : AudioSubsystem,
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

        SoundPlayer { audio_subsystem: audio_subsystem, device: device}
    }

}