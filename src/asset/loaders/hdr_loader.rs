use std::io::Cursor;

use image::{DynamicImage, ImageReader};

use crate::prelude::{GraphicsContext, Texture};

pub struct HDRLoaderOptions {
    pub is_flip_y: bool,
}

impl Default for HDRLoaderOptions {
    fn default() -> Self {
        Self { is_flip_y: Default::default() }
    }
}

pub struct HDRLoader {
    options: HDRLoaderOptions,
}

impl Default for HDRLoader {
    fn default() -> Self {
        Self {
            options: Default::default(),
        }
    }
}

impl HDRLoader {
    pub fn new(options: HDRLoaderOptions) -> Self {
        Self {
            options,
        }
    }

    pub fn load(&mut self, path: &str, graphics_context: &GraphicsContext) -> Texture {
        // info!("path: {}", path);
        let hdr_file_data = std::fs::read(path).expect("Failed to load hdr file.");
        self.load_by_bytes(hdr_file_data.as_ref(), graphics_context)
    }

    pub fn load_by_bytes(
        &mut self,
        hdr_file_data: &[u8],
        graphics_context: &GraphicsContext,
    ) -> Texture {
        let mut img = ImageReader::new(Cursor::new(hdr_file_data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        if self.options.is_flip_y {
            img = img.flipv();
        }
        if let DynamicImage::ImageRgb32F(rgb_image) = img {
            // vec![0.0; pixels1.len() * 3];
            // for pixel in rgb_image.pixels() {
            //     let r = pixel[0]; // 红色通道
            //     let g = pixel[1]; // 绿色通道
            //     let b = pixel[2]; // 蓝色通道
            //     if r > 1.0 || g > 1.0 || b > 1.0 { // for debug
            //         println!("Pixel: ({}, {}, {})", r, g, b);
            //     }
            // }

            // 将像素数据展平为 Vec<f32>
            let pixels: Vec<f32> = rgb_image
                .pixels()
                .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 1.0])
                .collect();

            let flat_data: &[u8] = bytemuck::cast_slice(&pixels);

            let hdr_texture = Texture::create_hdr_texture(
                graphics_context,
                rgb_image.width(),
                rgb_image.height(),
                flat_data,
                wgpu::TextureFormat::Rgba32Float,
                1,
            );
            return hdr_texture;
        } else {
            panic!("Failed to load hdr image");
        }
    }
}
