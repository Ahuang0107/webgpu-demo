use std::time::Duration;

#[derive(Debug, Default)]
pub struct EasingAnimator {
    start: f32,
    end: f32,
    duration: Duration,
    elapsed: Duration,
}

impl EasingAnimator {
    pub fn new(start: f32, end: f32, duration: Duration) -> EasingAnimator {
        EasingAnimator {
            start,
            end,
            duration,
            elapsed: Duration::ZERO,
        }
    }

    pub fn end(&self) -> f32 {
        self.end
    }

    pub fn update(&mut self, delta: Duration) -> f32 {
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            return self.end;
        }
        let mut progress =
            ease_out_cubic(self.elapsed.as_micros() as f32 / self.duration.as_micros() as f32);
        if progress > 0.96 {
            progress = 1.0;
        }
        self.start + (self.end - self.start) * progress
    }

    pub fn if_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// https://easings.net/#easeOutCubic
#[allow(dead_code)]
fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

#[allow(dead_code)]
fn ease_out_expo(x: f32) -> f32 {
    if x == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * x)
    }
}
