use std::borrow::Cow;

use crate::{prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, texture_manager::TextureManager, GraphicsContext, INVALID_ID}, types::ID};

use super::MaterialTrait;

/// This material can generate an irradiance map from a plain Cube Texture.
pub struct IrradianceMapGenMaterial {
    cube_texture: ID,
    cube_sampler: Option<wgpu::Sampler>,
    bind_group_id: ID,
    cull_mode: wgpu::Face,
    front_face: wgpu::FrontFace,
}

impl Default for IrradianceMapGenMaterial {
    fn default() -> Self {
        Self {
            cube_texture: INVALID_ID,
            cube_sampler: None,
            bind_group_id: INVALID_ID,
            cull_mode: wgpu::Face::Back,
            front_face: wgpu::FrontFace::Ccw,
        }
    }
}

impl MaterialTrait for IrradianceMapGenMaterial {
    fn on_init(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    ) {
        self.create_texture_sampler(graphics_context);
        Self::try_create_bind_group_layout(
            graphics_context,
            bind_group_layout_manager,
        );
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        texture_manager: &TextureManager,
    ) -> ID {
        let bind_group_layout =
            bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("Irradiance map gen bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        texture_manager.get_texture_view(self.cube_texture),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.get_cube_sampler()),
                },
            ],
        });

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
            label: Some("create irradiance map gen shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/irradiance_convolution.wgsl"
            ))),
        });
        shader
    }

    fn get_cull_mode(&self) -> wgpu::Face {
        self.cull_mode
    }

    fn set_cull_mode(&mut self, cull_mode: wgpu::Face) {
        self.cull_mode = cull_mode;
    }

    fn get_front_face(&self) -> wgpu::FrontFace {
        self.front_face
    }

    fn set_front_face(&mut self, front_face: wgpu::FrontFace) {
        self.front_face = front_face;
    }

    fn enable_lights(&self) -> bool {
        false
    }
}

impl IrradianceMapGenMaterial {
    pub fn new() -> Self {
        Default::default()
    }

    fn create_texture_sampler(&mut self, graphics_context: &GraphicsContext) {
        if self.cube_sampler.is_none() {
            let texture_sampler =
                graphics_context
                    .get_device()
                    .create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("irradiance map gen Sampler"),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        ..Default::default()
                    });
            self.cube_sampler = Some(texture_sampler);
        }
    }

    fn internal_bind_group_layout_id(new_id: usize) -> ID {
        static mut LAYOUT_ID: usize = INVALID_ID;
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
                        // equirectangular map
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
                ],
                label: Some("IrradicaneMapGenMaterial_bind_group_layout"),
            });
        bind_group_layout
    }

    pub fn set_input_cube_map(&mut self, cube_texture: ID) {
        self.cube_texture = cube_texture;
    }

    pub fn get_input_cube_map(&self) -> ID {
        self.cube_texture
    }

    pub fn get_cube_sampler(&self) -> &wgpu::Sampler {
        self.cube_sampler.as_ref().unwrap()
    }
}
