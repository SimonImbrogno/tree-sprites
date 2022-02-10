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

    pub fn measure<F>(&mut self, func: F) -> Duration where F: FnOnce() -> () {
        self.reset();
        func();
        self.elapsed()
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
