

use sdl2::audio::{AudioSpecDesired, AudioQueue};
use sdl2::AudioSubsystem;

pub struct SoundPlayer {
    pub device: AudioQueue<f32>,
    pub sound_syncer: SoundSyncer,
}

impl SoundPlayer {
    pub fn new(audio_subsystem : AudioSubsystem) -> SoundPlayer {
        let desired_spec = AudioSpecDesired {
            freq: Some(48000),
            channels: Some(1), // mono
            samples: Some(1024),     // default sample size
        };
        let device = audio_subsystem.open_queue::<f32, _>(None, &desired_spec).unwrap();
        // Start with buffer of no sound
        device.queue(&vec![0 as f32; 6144]);

        SoundPlayer { device: device, 
            sound_syncer: SoundSyncer::new()}
    }

    pub fn get_new_samplerate(&mut self) -> usize {
        self.sound_syncer.update(self.device.size() as usize);
        return self.sound_syncer.current_output_rate;
    }

}

const SAMPLES_PER_AVERAGE : usize = 50;

pub struct SoundSyncer {
    pub current_output_rate: usize,
    prev_buffer_sizes: [usize; SAMPLES_PER_AVERAGE],
    cur_buffer_size_index: usize,
    previous_buffer_avg: usize,
}

impl SoundSyncer {
    pub fn new() -> SoundSyncer {
        SoundSyncer {
            current_output_rate: 48000,
            prev_buffer_sizes: [0; SAMPLES_PER_AVERAGE],
            cur_buffer_size_index: 0,
            previous_buffer_avg: 0,

        }
    }
    pub fn update(&mut self, buffer_size: usize) {
        // Do this every SAMPLES_PER_AVERAGE times
        if self.cur_buffer_size_index == SAMPLES_PER_AVERAGE {
            self.cur_buffer_size_index = 0;
            let avg = self.get_average_buffer_size();
            // Only do this if previous_buffer has already been set once
            if self.previous_buffer_avg != 0 {
                let slope = (((self.previous_buffer_avg as f64 - avg as f64) / self.previous_buffer_avg as f64) as f64) / SAMPLES_PER_AVERAGE as f64;
                self.current_output_rate = (self.current_output_rate as f64 * (1.0 + slope)) as usize;
                //println!("prev_avg: {}, avg: {}", self.previous_buffer_avg, avg);
                //println!("Slope: {}, new rate: {}", slope, self.current_output_rate)
            }
            self.previous_buffer_avg = avg;
        }
        // Store the buffer size for every step, used to calculate average
        self.prev_buffer_sizes[self.cur_buffer_size_index] = buffer_size;
        self.cur_buffer_size_index += 1;
    }

    fn get_average_buffer_size(&self) -> usize {
        let mut avg = 0;
        for size in self.prev_buffer_sizes {
            avg += size;
        }
        return avg / SAMPLES_PER_AVERAGE;
    }

    pub fn skip_next_frame(&mut self) {
        self.previous_buffer_avg = 0;
    }

}