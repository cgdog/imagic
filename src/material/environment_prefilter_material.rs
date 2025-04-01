use std::borrow::Cow;

use crate::{asset::{asset::Handle, asset_manager::AssetManager}, prelude::{bind_group_layout::BindGroupLayoutManager, GraphicsContext, Texture, INVALID_ID}, types::ID};

use super::MaterialTrait;


/// Use to generate prefiltered environment map.
pub struct EnvironmentPrefilterMaterial {
    input_cube_texture: Handle<Texture>,
    texture_cube_sampler: Option<wgpu::Sampler>,
    roughness: f32,
    max_mipmap_level: u32,
    bind_group_id: ID,
    cull_mode: wgpu::Face,
    uniform_buffer: Option<wgpu::Buffer>,
}

impl Default for EnvironmentPrefilterMaterial {
    fn default() -> Self {
        Self {
            input_cube_texture: Handle::INVALID,
            texture_cube_sampler: None,
            roughness: 0.0,
            max_mipmap_level: 5,
            bind_group_id: INVALID_ID,
            cull_mode: wgpu::Face::Front,
            uniform_buffer: None,
        }
    }
}

impl MaterialTrait for EnvironmentPrefilterMaterial {
    fn set_cull_mode(&mut self, cull_mode: wgpu::Face) {
        self.cull_mode = cull_mode;
    }

    fn get_cull_mode(&self) -> wgpu::Face {
        self.cull_mode
    }

    fn on_init(
        &mut self,
        graphics_context: &crate::prelude::GraphicsContext,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager,
    ) {
        self.create_texture_sampler(graphics_context);
        Self::try_create_bind_group_layout(graphics_context, bind_group_layout_manager);
    }

    fn create_shader_module(&self, graphics_context: &crate::prelude::GraphicsContext) -> wgpu::ShaderModule {
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("create env prefilter shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/environment_prefilter.wgsl"
            ))),
        });
        shader
    }

    fn on_update(&mut self, graphics_context: &GraphicsContext) {
        graphics_context.get_queue().write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.roughness]),
        );
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &crate::prelude::GraphicsContext,
        bind_group_manager: &mut crate::prelude::bind_group::BindGroupManager,
        bind_group_layout_manager: &mut crate::prelude::bind_group_layout::BindGroupLayoutManager,
        asset_manager: &AssetManager,
    ) -> ID {
        let uniform_buffer =
            graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("env prefilter Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.roughness]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("env prefilter bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.input_cube_texture).expect("failed to get environment prefilter input_cube_texture").get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.get_cube_texture_sampler()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        self.uniform_buffer = Some(uniform_buffer);

        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        self.bind_group_id
    }

    fn get_bind_group_layout_id(&self) -> ID {
        Self::internal_bind_group_layout_id(INVALID_ID)
    }

    fn get_bind_group_id(&self) -> ID {
        self.bind_group_id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn enable_lights(&self) -> bool {
        false
    }
}

impl EnvironmentPrefilterMaterial {
    pub fn new(input_cube_texture: Handle<Texture>, max_mipmap_level: u32) -> Self {
        Self {
            input_cube_texture,
            max_mipmap_level,
            ..Default::default()
        }
    }
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

    fn internal_bind_group_layout_id(new_id: usize) -> ID {
        // All material instances share the same bind group layout.
        static mut LAYOUT_ID: ID = INVALID_ID;
        if new_id != INVALID_ID {
            unsafe { LAYOUT_ID = new_id };
        }
        unsafe { LAYOUT_ID }
    }

    fn try_create_bind_group_layout(
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    ) {
        let layout_id = Self::internal_bind_group_layout_id(INVALID_ID);
        if layout_id == INVALID_ID {
            let bind_group_layout = Self::create_bind_group_layout(graphics_context);
            let layout_id = bind_group_layout_manager.add_bind_group_layout(bind_group_layout);
            Self::internal_bind_group_layout_id(layout_id);
        }
    }

    fn create_bind_group_layout(graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
            graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        // skybox map
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // sampler
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // lod
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(4),
                        },
                        count: None,
                    },
                ],
                label: Some("CustomSkyboxMaterial_bind_group_layout"),
            });
        bind_group_layout
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

    pub fn get_cube_texture_sampler(&self) -> &wgpu::Sampler {
        self.texture_cube_sampler.as_ref().unwrap()
    }
}