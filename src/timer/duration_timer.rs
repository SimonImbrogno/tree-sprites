use std::time::{Duration, Instant};

use super::Timer;

pub struct DurationTimer {
    last_instant: Instant,
}

impl DurationTimer {
    pub fn new() -> Self {
        Self {
            last_instant: Instant::now(),
        }
    }

    pub fn begin_measure(&mut self) {
        self.reset();
    }

    pub fn end_measure(&mut self) -> Duration {
        self.elapsed()
    }

    pub fn measure<F>(&mut self, func: F) -> Duration where F: FnOnce() -> () {
        self.begin_measure();
        func();
        self.end_measure()
    }
}

impl Timer for DurationTimer {
    fn reset(&mut self) {
        self.last_instant = Instant::now();
    }

    fn elapsed(&self) -> Duration {
        (Instant::now() - self.last_instant)
    }
}
