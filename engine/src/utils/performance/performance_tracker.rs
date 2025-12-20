use crate::utils::FPSCounter;

pub struct PerformanceTracker {
    pub fps_counter: FPSCounter,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            fps_counter: FPSCounter::new()
        }
    }

    pub(crate) fn on_update(&mut self, time: f32) {
        self.fps_counter.on_update(time);
    }
}