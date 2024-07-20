use std::io::Cursor;

use image::codecs::hdr::HdrDecoder;
use log::info;

use crate::prelude::{GraphicsContext, Texture};

pub struct HDRLoader {

}

impl HDRLoader {
    pub fn load(&mut self, path: &str, graphics_context: &GraphicsContext) -> Texture {
        info!("path: {}", path);
        let hdr_file_data = std::fs::read(path).expect("Failed to load hdr file.");
        let hdr_decoder = HdrDecoder::new(Cursor::new(hdr_file_data)).unwrap();
        // let bytes_size = hdr_decoder.total_bytes() as usize;
        // let (width, height) = hdr_decoder.dimensions();
        // let mut hdr_pixels = vec![0u8; bytes_size];
        // hdr_decoder.read_image(&mut hdr_pixels).expect("Failed to parse .hdr file");

        let meta = hdr_decoder.metadata();

        let mut hdr_pixels = vec![[0.0, 0.0, 0.0, 0.0]; meta.width as usize * meta.height as usize];
        hdr_decoder.read_image_transform(
            |pix| {
                let rgb = pix.to_hdr();
                [rgb.0[0], rgb.0[1], rgb.0[2], 1.0f32]
            },
            &mut hdr_pixels[..],
        ).expect("Failed to read hdr pixels");



        // info!("hdr_pixels: {:?}", hdr_pixels);

        let hdr_texture = Texture::create_hdr_texture(graphics_context, meta.width, meta.height, bytemuck::cast_slice(&hdr_pixels), wgpu::TextureFormat::Rgba32Float);
        hdr_texture
    }
}