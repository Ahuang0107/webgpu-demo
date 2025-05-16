use std::time::{Duration, Instant};

pub struct Fps {
    frame_count: u32,
    last_log_instant: Instant,
    interval: Duration,
}

impl Fps {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_log_instant: Instant::now(),
            interval: Duration::from_secs(1),
        }
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now - self.last_log_instant;
        if elapsed >= self.interval {
            let fps = self.frame_count as f32 / elapsed.as_secs_f32();
            log::info!("FPS: {:.2}", fps);

            self.frame_count = 0;
            self.last_log_instant = now;
        }
    }
}
