#![macro_use]

use std::time::Duration;

mod average_duration_timer;
mod duration_timer;
mod target_timer;

pub use average_duration_timer::AverageDurationTimer;
pub use duration_timer::DurationTimer;
pub use target_timer::TargetTimer;

pub enum TimerState {
    Pending(Duration),
    Ready(Duration),
}

// For some reason can't import this directly... need to export it at crate level :|
macro_rules! measure {
    ($timer:expr, $code:block) => {
        $timer.begin_measure();
        $code
        $timer.end_measure()
    }
}

pub(crate) use measure;

pub trait Timer {
    /// Set the start point for measurement to 'now'.
    fn reset(&mut self);

    /// Check how much time has elapsed since the timer's start point was last set.
    fn elapsed(&self) -> Duration;
}
