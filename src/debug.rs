use std::time::Duration;
use crate::timer::{AverageDurationTimer, TargetTimer};

pub struct DebugTimers {
    //Main
    pub avg_update_timer: AverageDurationTimer<20>,
    pub avg_render_timer: AverageDurationTimer<20>,
    // TODO: configure how many samples recorded and how many used for average seperately.
    pub long_avg_update_timer: AverageDurationTimer<600>,
    pub long_avg_render_timer: AverageDurationTimer<600>,

    //Render
    pub debug_log_timer: TargetTimer,
    pub ground_render_timer: AverageDurationTimer<600>,
    pub tree_render_timer: AverageDurationTimer<600>,
}

impl DebugTimers {
    pub fn new() -> Self {
        Self {
            // Main
            avg_update_timer: AverageDurationTimer::new(),
            avg_render_timer: AverageDurationTimer::new(),
            long_avg_update_timer: AverageDurationTimer::new(),
            long_avg_render_timer: AverageDurationTimer::new(),

            //Render
            debug_log_timer: TargetTimer::new(Duration::from_secs(1)),
            ground_render_timer: AverageDurationTimer::new(),
            tree_render_timer: AverageDurationTimer::new(),
        }
    }
}
