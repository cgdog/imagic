use log::info;
use wgpu::TextureAspect;

use crate::{
    prelude::{ComputeShader, ImagicContext, Texture, INVALID_ID},
    types::ID,
};

pub struct CubeMipmapsGenerator {
    cube_without_mipmaps: ID,
    cube_with_mipmaps: ID,
    face_size: u32,
    format: wgpu::TextureFormat,
}

impl ComputeShader for CubeMipmapsGenerator {
    fn execute(&mut self, imagic_context: &mut ImagicContext) {
        self.create_cube_texture_with_mipmaps(imagic_context);
    }
}

impl CubeMipmapsGenerator {
    pub fn new(cube_without_mipmaps: ID, face_size: u32, format: wgpu::TextureFormat) -> Self {
        Self {
            cube_without_mipmaps,
            face_size,
            cube_with_mipmaps: INVALID_ID,
            format,
        }
    }

    pub fn get_cube_with_mipmap(&self) -> ID {
        self.cube_with_mipmaps
    }

    fn create_cube_texture_with_mipmaps(&mut self, imagic_context: &mut ImagicContext) {
        let mip_level_count = self.face_size.ilog2() + 1;
        let cube_map_with_mipmap = Texture::create_cube_texture(
            imagic_context.graphics_context(),
            self.format,
            self.face_size,
            self.face_size,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            mip_level_count,
        );

        let original_cube_texture = imagic_context
            .texture_manager()
            .get_texture(self.cube_without_mipmaps);

        let mut encoder = imagic_context
            .graphics_context()
            .get_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mipmap Copy Encoder"),
            });
        for layer in 0..6 {
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: original_cube_texture.get(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: cube_map_with_mipmap.get(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
            );
        }

        imagic_context.graphics_context().get_queue().submit(Some(encoder.finish()));
        imagic_context.graphics_context().get_device().poll(wgpu::Maintain::Wait);

        self.cube_with_mipmaps = imagic_context
            .texture_manager_mut()
            .add_texture(cube_map_with_mipmap);
        info!("cube_with_mipmaps: {}", self.cube_with_mipmaps);
    }
}
