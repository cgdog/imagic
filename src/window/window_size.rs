#[derive(Clone, Copy)]
pub struct WindowSize {
    width: f32,
    height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0
        }
    }
}

impl WindowSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height
        }
    }

    pub fn set(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn get(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_aspect(&self) -> f32 {
        self.width / self.height
    }

    pub fn get_half_width(&self) -> f32 {
        self.width * 0.5
    }

    pub fn get_half_height(&self) -> f32 {
        self.height * 0.5
    }
}