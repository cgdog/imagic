use std::{borrow::Cow, usize};

use imagic::{
    prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager,
        texture_manager::TextureManager, GraphicsContext, MaterialTrait, INVALID_ID,
    },
    types::ID,
    utils::changeable::Changeable,
};

pub struct CustomSkyboxMaterial {
    skybox_map: ID,
    texture_cube_sampler: Option<wgpu::Sampler>,
    bind_group_id: ID,
    cull_mode: wgpu::Face,
    lod: Changeable<f32>,
    uniform_buffer: Option<wgpu::Buffer>,
}

impl Default for CustomSkyboxMaterial {
    fn default() -> Self {
        Self {
            skybox_map: INVALID_ID,
            texture_cube_sampler: None,
            bind_group_id: INVALID_ID,
            cull_mode: wgpu::Face::Front,
            lod: Changeable::new(0.0),
            uniform_buffer: None,
        }
    }
}

impl MaterialTrait for CustomSkyboxMaterial {
    fn on_init(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    ) {
        self.create_texture_sampler(graphics_context);
        Self::try_create_bind_group_layout(graphics_context, bind_group_layout_manager);
    }

    fn on_update(&mut self, graphics_context: &GraphicsContext) {
        if self.lod.is_changed() {
            self.lod.reset();
            graphics_context.get_queue().write_buffer(
                self.uniform_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&[*self.lod]),
            );
        }
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        texture_manager: &TextureManager,
    ) -> ID {
        let uniform_buffer =
            graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("custom skybox Uniform Buffer"),
                contents: bytemuck::cast_slice(&[*self.lod]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("Skybox bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        texture_manager.get_texture_view(self.skybox_map),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.get_cube_texture_sampler()),
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

    fn create_shader_module(&self, graphics_context: &GraphicsContext) -> wgpu::ShaderModule {
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("create skybox shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/custom_skybox.wgsl"
            ))),
        });
        shader
    }

    fn set_cull_mode(&mut self, cull_mode: wgpu::Face) {
        self.cull_mode = cull_mode;
    }

    fn get_cull_mode(&self) -> wgpu::Face {
        self.cull_mode
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[allow(unused)]
impl CustomSkyboxMaterial {
    #[allow(unused)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_lod(&mut self, lod: f32) {
        *self.lod = lod;
        self.lod.set();
    }

    pub fn get_lod(&self) -> f32 {
        *self.lod
    }

    fn create_texture_sampler(&mut self, graphics_context: &GraphicsContext) {
        if self.texture_cube_sampler.is_none() {
            let texture_sampler =
                graphics_context
                    .get_device()
                    .create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("Custom Skybox CubeTexture Sampler"),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        ..Default::default()
                    });
            self.texture_cube_sampler = Some(texture_sampler);
        }
    }

    fn internal_bind_group_layout_id(new_id: usize) -> ID {
        // All SkyboxMaterial instances share the same bind group layout.
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
                        // This should match the filterable field of the
                        // corresponding Texture entry.
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

    #[allow(unused)]
    pub fn set_skybox_map(&mut self, skybox_map: usize) {
        self.skybox_map = skybox_map;
    }

    #[allow(unused)]
    pub fn get_skybox_map(&self) -> ID {
        self.skybox_map
    }

    pub fn get_cube_texture_sampler(&self) -> &wgpu::Sampler {
        self.texture_cube_sampler.as_ref().unwrap()
    }
}
