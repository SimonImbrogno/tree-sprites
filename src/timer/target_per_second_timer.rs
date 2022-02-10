use std::time::Duration;

use super::{DurationTimer, Timer, TargetTimer, TimerState};

pub struct TargetPerSecondTimer {
    target_period: Duration,
    duration_timer: DurationTimer,
}

impl TargetPerSecondTimer {
    pub fn new(per_second_target: u64) -> Self {
        Self {
            target_period: Duration::from_micros(1_000_000 / per_second_target),
            duration_timer: DurationTimer::new(),
        }
    }

    pub fn target(&self) -> Duration {
        self.target_period
    }

    pub fn check(&mut self) -> TimerState {
        let elapsed = self.duration_timer.elapsed();

        let result = {
            if elapsed >= self.target_period {
                TimerState::Ready(elapsed)
            } else {
                TimerState::Pending(elapsed)
            }
        };

        result
    }
}

impl Timer for TargetPerSecondTimer {
    fn reset(&mut self) {
        self.duration_timer.reset();
    }

    fn elapsed(&self) -> Duration {
        self.duration_timer.elapsed()
    }
}
