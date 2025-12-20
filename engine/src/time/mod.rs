use std::time::Instant;

use crate::math::Vec4;

pub struct Time {
    start: Instant,
    last_frame_time: f32,
    scale: f32,
    enable_time_scale_flag: bool,
    /// x: time since started, y: delta time, z: scaled delta time, w: sin(time)
    pub(crate) time_data: Vec4,
}

impl Time {
    pub(crate) fn new() -> Self {
        let start = Instant::now();
        Self {
            start,
            last_frame_time: 0.0,
            scale: 1.0,
            enable_time_scale_flag: false,
            time_data: Vec4::ZERO,
        }
    }

    #[allow(unused)]
    pub(crate) fn reset(&mut self) {
        self.start = Instant::now();
        self.last_frame_time = 0.0;
        self.scale = 1.0;
        self.enable_time_scale_flag = false;
        self.time_data.x = 0.0;
        self.time_data.y = 0.0;
        self.time_data.z = 0.0;
        self.time_data.w = 0.0;
    }

    /// Time (in seconds) elapsed since game starts.
    pub fn elapsed(&self) -> f32 {
        self.time_data.x
    }

    /// Delta time inseconds.
    pub fn delta(&self) -> f32 {
        self.time_data.y
    }

    pub fn scaled_delta(&self) -> f32 {
        self.time_data.z
    }

    pub fn sin_time(&self) -> f32 {
        self.time_data.w
    }

    pub fn on_update(&mut self) {
        self.time_data.x = self.start.elapsed().as_secs_f32();
        self.time_data.y = self.time_data.x - self.last_frame_time;
        if self.enable_time_scale_flag {
            self.time_data.z = self.time_data.y * self.scale;
        }
        self.last_frame_time = self.time_data.x;
        self.time_data.w = self.time_data.x.sin();
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn enable_time_scale(&mut self) {
        self.enable_time_scale_flag = true;
    }

    pub fn disable_time_scale(&mut self) {
        self.enable_time_scale_flag = false;
    }
}