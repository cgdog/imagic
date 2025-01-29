use crate::{prelude::{GraphicsContext, INVALID_ID}, types::ID};

use super::MaterialTrait;

#[allow(unused)]
pub struct CubeTexturePrefilterMaterial {
    input_cube_texture: ID,
    texture_cube_sampler: Option<wgpu::Sampler>,
    roughness: f32,
    max_mipmap_level: u32,
    bind_group_id: ID,
    cull_mode: wgpu::Face,
}

impl Default for CubeTexturePrefilterMaterial {
    fn default() -> Self {
        Self {
            input_cube_texture: INVALID_ID,
            texture_cube_sampler: None,
            roughness: 0.0,
            max_mipmap_level: 5,
            bind_group_id: INVALID_ID,
            cull_mode: wgpu::Face::Back,
        }
    }
}

#[allow(unused)]
impl MaterialTrait for CubeTexturePrefilterMaterial {
    fn on_init(
        &mut self,
        graphics_context: &crate::prelude::GraphicsContext,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager,
    ) {
        self.create_texture_sampler(graphics_context);
    }

    fn create_shader_module(&self, graphics_context: &crate::prelude::GraphicsContext) -> wgpu::ShaderModule {
        todo!()
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &crate::prelude::GraphicsContext,
        bind_group_manager: &mut crate::prelude::bind_group::BindGroupManager,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager,
        texture_manager: &crate::prelude::texture_manager::TextureManager,
    ) -> ID {
        todo!()
    }

    fn get_bind_group_layout_id(&self) -> ID {
        todo!()
    }

    fn get_bind_group_id(&self) -> ID {
        todo!()
    }
}

impl CubeTexturePrefilterMaterial {
    fn create_texture_sampler(&mut self, graphics_context: &GraphicsContext) {
        if self.texture_cube_sampler.is_none() {
            let texture_sampler =
                graphics_context
                    .get_device()
                    .create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("CubeTexturePrefilterMaterial Sampler"),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        ..Default::default()
                    });
            self.texture_cube_sampler = Some(texture_sampler);
        }
    }

    pub fn set_roughness(&mut self, roughness: f32) {
        self.roughness = roughness;
    }

    pub fn get_roughness(&self) -> f32 {
        self.roughness
    }

    pub fn set_max_mipmap_level(&mut self, max_mipmap_level: u32) {
        self.max_mipmap_level = max_mipmap_level;
    }

    pub fn get_max_mipmap_level(&self) -> u32 {
        self.max_mipmap_level
    }

    pub fn set_cull_mode(&mut self, cull_mode: wgpu::Face) {
        self.cull_mode = cull_mode;
    }

    pub fn get_cull_mode(&self) -> wgpu::Face {
        self.cull_mode
    }
}