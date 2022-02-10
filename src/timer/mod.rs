use std::time::Duration;

mod average_duration_timer;
mod duration_timer;
mod target_timer;
mod target_per_second_timer;

pub use average_duration_timer::AverageDurationTimer;
pub use duration_timer::DurationTimer;
pub use target_timer::TargetTimer;
pub use target_per_second_timer::TargetPerSecondTimer;

pub enum TimerState {
    Pending(Duration),
    Ready(Duration),
}

pub trait Timer {
    /// Set the start point for measurement to 'now'.
    fn reset(&mut self);

    /// Check how much time has elapsed since the timer's start point was last set.
    fn elapsed(&self) -> Duration;
}
