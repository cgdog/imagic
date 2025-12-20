pub struct FPSCounter {
    frame_count: u32,
    last_count_time: f32,
    pub fps: f32,
}

impl FPSCounter {
    pub fn new() -> Self {
        Self { frame_count: 0, last_count_time: 0.0, fps: 0.0 }
    }

    pub(crate) fn on_update(&mut self, new_time: f32) {
        self.frame_count += 1;
        let elapsed_seconds = new_time - self.last_count_time;
        if elapsed_seconds >= 1.0 {
            self.fps = self.frame_count as f32 / elapsed_seconds;
            self.frame_count = 0;
            self.last_count_time = new_time;
        }
    }
}