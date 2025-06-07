use instant::{Duration, Instant};

pub struct Fps {
    frame_count: u32,
    last_log_instant: Instant,
    interval: Duration,
    pub fps: f32,
}

impl Fps {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_log_instant: Instant::now(),
            interval: Duration::from_secs(1),
            fps: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now - self.last_log_instant;
        if elapsed >= self.interval {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_log_instant = now;
        }
    }
}
