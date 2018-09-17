use std::time::Instant;

#[derive(Debug)]
pub struct PerfCounter {
    last_measure: Instant,
    count: f64,
    last_rate: f64,
}

impl PerfCounter {
    pub fn new() -> Self {
        Self {
            last_measure: Instant::now(),
            count: 0.0,
            last_rate: 0.0,
        }
    }

    pub fn log(&mut self, value: f64) {
        self.count += value;

        let dt = self.last_measure.elapsed();
        let dt = dt.subsec_nanos() as f64 * 1.0e-9 + dt.as_secs() as f64;
        if dt >= 0.2 {
            self.last_rate = self.count / dt;
            self.count = 0.0;
            self.last_measure = Instant::now();
        }
    }

    pub fn rate(&self) -> f64 {
        self.last_rate
    }
}
