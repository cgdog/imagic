use std::{borrow::Cow, usize};

use crate::prelude::{bind_group_layout::BindGroupLayoutManager, GraphicsContext};

use super::MaterialTrait;

pub struct EquirectangularToCubeMaterial {
    equirectangular_map: usize,
    texture2d_sampler: Option<wgpu::Sampler>,
    bind_group_id: usize,
}

impl Default for EquirectangularToCubeMaterial {
    fn default() -> Self {
        Self {
            equirectangular_map: usize::MAX,
            texture2d_sampler: None,
            bind_group_id: usize::MAX,
        }
    }
}

impl MaterialTrait for EquirectangularToCubeMaterial {
    fn init(&mut self, graphics_context: &crate::prelude::GraphicsContext, bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager) {
        self.create_texture_sampler(graphics_context);
        EquirectangularToCubeMaterial::try_create_bind_group_layout(graphics_context, bind_group_layout_manager);
    }

    fn create_bind_group(&mut self, graphics_context: &crate::prelude::GraphicsContext, bind_group_manager: &mut crate::prelude::bind_group::BindGroupManager,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager, texture_manager: &crate::prelude::texture_manager::TextureManager) -> usize {
        let bind_group_layout = bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: bind_group_layout,
            label: Some("Equirectangular to cube bind group"),
            entries: &[
                wgpu::BindGroupEntry {// albedo map
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_manager.get_texture_view(self.equirectangular_map)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.get_2d_texture_sampler()),
                },
            ]
        });

        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        self.bind_group_id
    }

    fn get_bind_group_layout_id(&self) -> usize {
        EquirectangularToCubeMaterial::internal_bind_group_layout_id(usize::MAX)
    }

    fn get_bind_group_id(&self) -> usize {
        self.bind_group_id
    }

    fn create_shader_module(&self, graphics_context: &crate::prelude::GraphicsContext) -> wgpu::ShaderModule {
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crate equirectangular to cube shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/equirectangular_to_cube.wgsl"))),
        });
        shader
    }
}

impl EquirectangularToCubeMaterial {

    pub fn new() -> Self {
        Default::default()
    }

    fn create_texture_sampler(&mut self, graphics_context: &GraphicsContext) {
        if self.texture2d_sampler.is_none() {
            let texture_sampler = graphics_context.get_device().create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            self.texture2d_sampler = Some(texture_sampler);
        }
    }

    fn internal_bind_group_layout_id(new_id: usize) -> usize {
        // All EquirectangularToCubeMaterial instances share the same bind group layout.
        static mut LAYOUT_ID: usize = usize::MAX;
        if new_id != usize::MAX {
            unsafe { LAYOUT_ID = new_id };
        }
        unsafe { LAYOUT_ID }
    }

    fn try_create_bind_group_layout(graphics_context: &GraphicsContext, bind_group_layout_manager: &mut BindGroupLayoutManager) {
        let layout_id = EquirectangularToCubeMaterial::internal_bind_group_layout_id(usize::MAX);
        if layout_id == usize::MAX {
            let bind_group_layout = EquirectangularToCubeMaterial::create_bind_group_layout(graphics_context);
            let layout_id = bind_group_layout_manager.add_bind_group_layout(bind_group_layout);
            EquirectangularToCubeMaterial::internal_bind_group_layout_id(layout_id);
        }
    }

    fn create_bind_group_layout(graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
        graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {// equirectangular map
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {// sampler
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("EquirectangularToCubeMaterial_bind_group_layout"),
        });
        bind_group_layout
    }

    pub fn set_equirectangular_map(&mut self, equirectangular_map: usize) {
        self.equirectangular_map = equirectangular_map;
    }

    pub fn get_equirectangular_map(&self) -> usize {
        self.equirectangular_map
    }

    pub fn get_2d_texture_sampler(&self, ) -> &wgpu::Sampler {
        self.texture2d_sampler.as_ref().unwrap()
    }

}