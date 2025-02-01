use crate::{
    camera::Camera,
    model::Cube,
    prelude::{CubeRenderTexture, EnvironmentPrefilterMaterial, ImagicContext},
    types::ID,
};

pub struct CubeTexturePrefilter {
    input_cube_texture: ID,
    max_mipmap_level: u32,
    face_size: u32,
}

impl CubeTexturePrefilter {
    pub fn new(input_cube_texture: ID, max_mipmap_level: u32, face_size: u32) -> Self {
        Self {
            input_cube_texture,
            max_mipmap_level,
            face_size,
        }
    }

    pub fn prefilter(
        &mut self,
        imagic_context: &mut ImagicContext,
        camera: &mut Camera,
        cube: &mut Cube,
        rt_format: wgpu::TextureFormat,
    ) {
        let prefilter_material =
            EnvironmentPrefilterMaterial::new(self.input_cube_texture, self.max_mipmap_level);
        let cube_rt = CubeRenderTexture::new(
            imagic_context,
            rt_format,
            self.face_size,
            self.face_size,
            self.max_mipmap_level,
        );

        
    }
}
