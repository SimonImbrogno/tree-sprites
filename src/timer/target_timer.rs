use std::time::Duration;

use super::{DurationTimer, Timer, TimerState};

pub struct TargetTimer {
    target_duration: Duration,
    duration_timer: DurationTimer,
}

impl TargetTimer {
    pub fn new(target: Duration) -> Self {
        Self {
            target_duration: target,
            duration_timer: DurationTimer::new(),
        }
    }

    pub fn target(&self) -> Duration {
        self.target_duration
    }

    pub fn check(&mut self) -> TimerState {
        let elapsed = self.elapsed();

        let result = {
            if elapsed >= self.target_duration {
                TimerState::Ready(elapsed)
            } else {
                TimerState::Pending(elapsed)
            }
        };

        result
    }
}

impl Timer for TargetTimer {
    fn reset(&mut self) {
        self.duration_timer.reset();
    }

    fn elapsed(&self) -> Duration {
        self.duration_timer.elapsed()
    }
}
