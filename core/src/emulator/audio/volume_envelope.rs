
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct VolumeEnvelope {
    pub volume: u8,
    pub delay: u8,
}

impl VolumeEnvelope {
    pub fn new() -> VolumeEnvelope {
        return VolumeEnvelope { 
            volume: 0,
            delay: 0,
        }
    }

    pub fn step(&mut self, period: u8, mode: bool) {
        if self.delay > 1 {
            self.delay -= 1;
        }
        else if self.delay == 1 {
            self.delay = period;
            if mode && self.volume < 15 { // Increasing
                self.volume += 1;
            }
            else if !mode && self.volume > 0 { // Decreasing
                self.volume -= 1;
            }
        }
    }
}