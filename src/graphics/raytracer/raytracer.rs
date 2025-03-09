use image::{ImageBuffer, Rgb};

use crate::math::ColorRGB;

/// Ray tracer
pub struct RayTracer {
    width: u32,
    height: u32,
}

impl RayTracer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn render(&mut self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut frame_buffer = vec![ColorRGB::ZERO; (self.width * self.height) as usize];

        for j in 0..self.height {
            for i in 0..self.width {
                let color = &mut frame_buffer[(i + j * self.width) as usize];
                
                color.x = j as f32 / self.height as f32;
                color.y = i as f32 / self.width as f32;
                color.z = 0.0;
            }
        }

        let mut imgbuf = image::ImageBuffer::new(self.width, self.height);
        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let index = (y * self.width + x) as usize;
            let color = &frame_buffer[index];
            let r = (255.0 * color.x.clamp(0.0, 1.0)) as u8;
            let g = (255.0 * color.y.clamp(0.0, 1.0)) as u8;
            let b = (255.0 * color.z.clamp(0.0, 1.0)) as u8;
            *pixel = image::Rgb([r, g, b]);
        }

        imgbuf
    }
}
