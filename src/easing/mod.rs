#[derive(Debug, Default)]
pub struct EasingAnimator {
    start: f32,
    end: f32,
    duration_ms: u64,
    elapsed_ms: u64,
}

impl EasingAnimator {
    pub fn new(start: f32, end: f32, duration_ms: u64) -> EasingAnimator {
        EasingAnimator {
            start,
            end,
            duration_ms,
            elapsed_ms: 0,
        }
    }

    pub fn end(&self) -> f32 {
        self.end
    }

    pub fn update(&mut self, delta_ms: u64) -> f32 {
        self.elapsed_ms += delta_ms;
        if self.elapsed_ms >= self.duration_ms {
            return self.end;
        }
        let mut progress = ease_out_cubic(self.elapsed_ms as f32 / self.duration_ms as f32);
        if progress > 0.96 {
            progress = 1.0;
        }
        self.start + (self.end - self.start) * progress
    }

    pub fn if_finished(&self) -> bool {
        self.elapsed_ms >= self.duration_ms
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
