use std::time::{Duration, Instant};

pub struct Timer{
    time_start: Instant,
    time_alloc: Duration,
}

impl Timer{
    pub fn new(duration: Duration) -> Timer{
        return Timer{time_start: Instant::now(), time_alloc: duration}
    }

    pub fn time_out(&self) -> bool {
        return self.time_start.elapsed() > self.time_alloc; 
    }
}