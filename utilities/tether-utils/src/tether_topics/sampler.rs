use std::time::{Duration, SystemTime};

use circular_buffer::CircularBuffer;

const WINDOW_SIZE: usize = 64;

type Sample = u128;
pub struct Sampler {
    buffer: CircularBuffer<WINDOW_SIZE, Sample>,
    interval: Duration,
    last_sample_time: SystemTime,
}

impl Sampler {
    pub fn new(interval_ms: u64) -> Self {
        Sampler {
            buffer: CircularBuffer::new(),
            last_sample_time: SystemTime::now(),
            interval: Duration::from_millis(interval_ms),
        }
    }

    pub fn add_sample(&mut self, absolute: u128) -> bool {
        if let Ok(elapsed) = self.last_sample_time.elapsed() {
            if elapsed > self.interval {
                self.last_sample_time = SystemTime::now();
                self.buffer.push_back(absolute);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn buffer(&self) -> &CircularBuffer<WINDOW_SIZE, Sample> {
        &self.buffer
    }

    pub fn delta_entries(&self) -> Vec<u128> {
        let mut result = Vec::with_capacity(WINDOW_SIZE);
        let mut previous = 0;
        for entry in &self.buffer {
            let delta = entry - previous;
            previous = *entry;
            result.push(delta);
        }
        result
    }
}
