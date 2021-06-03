
/// Simple sample buffer which can convert between two bitrates
/// Uses nearest neighbour sampling
/// Drop-in replacement for BlipBuf to allow for WASM compilation
/// which is needed for the web frontend
/// It only support sequential deltas unlike BlipBuf
pub struct SampleBuf {
    samples: Vec<i16>,
    clock_rate: f64,
    sample_rate: f64,
    clocks_per_sample: usize,
    samples_available: usize,
    current_value: i16,
    current_time: usize,
}

const SCALE_FACTOR: usize = 1000;

impl SampleBuf {
    /// Creates new buffer that can hold at most sample_count samples. Sets rates
    /// so that there are `MAX_RATIO` clocks per sample. Returns pointer to new
    /// buffer, or panics if insufficient memory.
    pub fn new(sample_count: u32) -> SampleBuf {
        SampleBuf { 
            samples: vec![0; sample_count as usize],
            clock_rate: 0.0, 
            sample_rate: 0.0, 
            clocks_per_sample: 0,
            samples_available: 0,
            current_value: 0,
            current_time: 0,
        }
    }

    /// Sets approximate input clock rate and output sample rate. For every
    /// `clock_rate` input clocks, approximately `sample_rate` samples are generated.
    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        self.clock_rate = clock_rate;
        self.sample_rate = sample_rate;
        self.clocks_per_sample = ((clock_rate * SCALE_FACTOR as f64) / sample_rate).ceil() as usize;
    }

    /// Clears entire buffer
    pub fn clear(&mut self) {
        self.samples_available = 0;
        self.current_value = 0;
    }

    /// Adds positive/negative delta into buffer at specified clock time.
    pub fn add_delta(&mut self, clock_time: u32, delta: i32) {
        while self.current_time < (clock_time as usize * SCALE_FACTOR) {
            self.current_time += self.clocks_per_sample;
            self.samples[self.samples_available] = self.current_value;
            self.samples_available += 1;
        }
        self.current_value = (self.current_value as i32 + delta) as i16;
    }

    /// Generate all samples up to this point and reset the time
    pub fn end_frame(&mut self, clock_duration: u32) {
        self.add_delta(clock_duration, 0);
        self.current_time = 0;
    }

    /// Read samples into buffer
    pub fn read_samples(&mut self, buf: &mut [i16], _stereo: bool) -> usize {
        buf[..self.samples_available].copy_from_slice(&self.samples[..self.samples_available]);
        let samples = self.samples_available;
        self.samples_available = 0;
        return samples;
    }
}