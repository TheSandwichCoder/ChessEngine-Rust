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

// this is literally just a profiler that I made myself 
// cus stupid windows does not support samply

pub struct Profiler{
    timer: Instant,
    time_taken: Duration,
    sample_count: u128,
}

impl Profiler{
    pub fn new() -> Profiler{
        return Profiler{timer: Instant::now(), time_taken: Duration::new(0, 0), sample_count: 0};
    }

    pub fn timer_start(&mut self){
        self.timer = Instant::now();
    }

    pub fn timer_end(&mut self){
        self.time_taken += self.timer.elapsed();
        self.sample_count += 1;
    }

    pub fn show(&self){
        println!("total: {}ms", self.time_taken.as_millis());
        println!("average sample: {} microsecond", self.time_taken.as_micros() / self.sample_count);
    }
}