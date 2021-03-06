use std::time::Duration;

use super::{DurationTimer, Timer};

pub struct AverageDurationTimer<const NUM_MEASUREMENTS: usize = 20> {
    duration_timer: DurationTimer,
    measurement_index: usize,
    measurements: Vec<Duration>,
    average: Duration,
}

impl<const NUM_MEASUREMENTS: usize> AverageDurationTimer<NUM_MEASUREMENTS> {
    pub fn new() -> Self {
        let mut measurements = Vec::new();
        measurements.resize_with(NUM_MEASUREMENTS, Default::default);

        Self {
            duration_timer: DurationTimer::new(),
            measurement_index: 0,
            measurements,
            average: Duration::default(),
        }
    }

    pub fn clear(&mut self) {
        self.measurement_index = 0;
        self.average = Duration::default();
    }

    pub fn average(&self) -> Duration {
        self.average
    }

    pub fn end(&mut self) -> Duration {
        let new_measurement = self.duration_timer.elapsed();
        self.duration_timer.reset();

        let index = self.measurement_index % self.measurements.len();
        let old_measurement = self.measurements[index];

        self.measurements[index] = new_measurement;
        self.measurement_index += 1;

        self.average = {
            let avg = self.average.as_nanos() as i128;
            let old = old_measurement.as_nanos() as i128;
            let new = new_measurement.as_nanos() as i128;

            let new_average = avg - ((old - new) / NUM_MEASUREMENTS as i128);

            Duration::from_nanos(new_average as u64)
        };

        new_measurement
    }

    pub fn begin_measure(&mut self) {
        self.reset();
    }

    pub fn end_measure(&mut self) -> Duration {
        self.end()
    }

    pub fn measure<F>(&mut self, func: F) -> Duration where F: FnOnce() -> (){
        self.begin_measure();
        func();
        self.end_measure()
    }

    pub fn measurements(&self) -> &[Duration]{
        &self.measurements
    }
}

impl<const NUM_MEASUREMENTS: usize> Timer for AverageDurationTimer<NUM_MEASUREMENTS> {
    fn reset(&mut self) {
        self.duration_timer.reset();
    }

    fn elapsed(&self) -> Duration {
        self.duration_timer.elapsed()
    }
}
