use std::borrow::Cow;

use crate::{
    asset::{asset::Handle, asset_manager::AssetManager}, math::{color::Color, Vec4}, prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, GraphicsContext, Texture, INVALID_ID,
    }, types::ID
};

use super::material_trait::MaterialTrait;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PBRFragmentUniforms {
    // x: enabled features. y,z,w: reserved.
    features: [u32; 4],
    albedo: [f32; 4],
    metallic_roughness_ao: [f32; 4],
}

pub struct PBRMaterial {
    albedo_color: Color,
    metallic_roughness_ao: Vec4,

    albedo_texture: Handle<Texture>,
    normal_textue: Handle<Texture>,
    metallic_texture: Handle<Texture>,
    roughness_texture: Handle<Texture>,
    ao_texture: Handle<Texture>,

    irradiance_cube_texture: Handle<Texture>,
    prefiltered_cube_texture: Handle<Texture>,
    brdf_lut: Handle<Texture>,

    texture2d_sampler: Option<wgpu::Sampler>,
    texture_cube_sampler: Option<wgpu::Sampler>,

    bind_group_id: ID,

    fragment_uniform_buffer: Option<wgpu::Buffer>,
}

impl Default for PBRMaterial {
    fn default() -> Self {
        Self {
            albedo_color: Vec4::ONE,
            metallic_roughness_ao: Vec4::ONE,
            albedo_texture: Texture::white().clone(),
            normal_textue: Texture::white().clone(),
            metallic_texture: Texture::white().clone(),
            roughness_texture: Texture::white().clone(),
            ao_texture: Texture::white().clone(),
            irradiance_cube_texture: Texture::cube_texture_placeholder().clone(),
            prefiltered_cube_texture: Texture::cube_texture_placeholder().clone(),
            brdf_lut: Texture::white().clone(),
            texture2d_sampler: None,
            texture_cube_sampler: None,

            bind_group_id: INVALID_ID,
            fragment_uniform_buffer: None,
        }
    }
}

impl MaterialTrait for PBRMaterial {
    fn on_init(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    ) {
        self.create_texture_sampler(graphics_context);

        PBRMaterial::try_create_bind_group_layout(graphics_context, bind_group_layout_manager);
    }

    fn create_shader_module(&self, graphics_context: &GraphicsContext) -> wgpu::ShaderModule {
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("create PBR shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/pbr.wgsl"))),
        });
        shader
    }

    fn on_update(&mut self, graphics_context: &GraphicsContext) {
        let fragment_uniforms = PBRFragmentUniforms {
            features: self.get_enabled_features(),
            albedo: self.albedo_color.to_array(),
            metallic_roughness_ao: self.metallic_roughness_ao.to_array(),
        };

        graphics_context.get_queue().write_buffer(
            self.fragment_uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[fragment_uniforms]),
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        asset_manager: &AssetManager,
    ) -> ID {
        let fragment_uniforms = PBRFragmentUniforms {
            features: self.get_enabled_features(),
            albedo: self.albedo_color.to_array(),
            metallic_roughness_ao: self.metallic_roughness_ao.to_array(),
        };

        let fragment_uniform_buffer =
            graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PBR Fragment Uniform Buffer"),
                contents: bytemuck::cast_slice(&[fragment_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("PBR bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fragment_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.get_2d_texture_sampler()),
                },
                wgpu::BindGroupEntry {
                    // albedo map
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.albedo_texture).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    // normal map
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.normal_textue).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    // metallic map
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.metallic_texture).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    // roughness map
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.roughness_texture).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    // ao map
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.ao_texture).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    // irradiance map
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.irradiance_cube_texture).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(&self.get_cube_texture_sampler()),
                },
                wgpu::BindGroupEntry {
                    // prefiltered reflection map
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.prefiltered_cube_texture).unwrap().get_texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    // brdf lut
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(
                        asset_manager.get(&self.brdf_lut).unwrap().get_texture_view(),
                    ),
                },
            ],
        });
        self.fragment_uniform_buffer = Some(fragment_uniform_buffer);
        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        self.bind_group_id
    }

    fn get_bind_group_id(&self) -> ID {
        self.bind_group_id
    }

    fn get_bind_group_layout_id(&self) -> ID {
        PBRMaterial::internal_bind_group_layout_id(INVALID_ID)
    }
}

impl PBRMaterial {
    pub fn new(albedo_color: Color, metallic: f32, roughness: f32, ao: f32) -> Self {
        Self {
            albedo_color,
            metallic_roughness_ao: Vec4::new(metallic, roughness, ao, 1.0),
            ..Default::default()
        }
    }

    fn internal_bind_group_layout_id(new_id: ID) -> ID {
        // All PBRMaterial instances share the same bind group layout.
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
        let layout_id = PBRMaterial::internal_bind_group_layout_id(INVALID_ID);
        if layout_id == INVALID_ID {
            let bind_group_layout = PBRMaterial::create_bind_group_layout(graphics_context);
            let layout_id = bind_group_layout_manager.add_bind_group_layout(bind_group_layout);
            PBRMaterial::internal_bind_group_layout_id(layout_id);
        }
    }

    fn create_bind_group_layout(graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
            graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        // FragmentUniforms: albedo color, metallic_roughness_ao
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16 * 3),
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
                        // albedo map
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // normal map
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // metallic map
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // roughness map
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // ao map
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // irradiance cube map
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // cube sampler
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // reflection cube map
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        // brdflut map
                        binding: 10,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
                label: Some("PBRMaterial_bind_group_layout"),
            });
        bind_group_layout
    }

    fn create_texture_sampler(&mut self, graphics_context: &GraphicsContext) {
        if self.texture2d_sampler.is_none() {
            let texture_sampler =
                graphics_context
                    .get_device()
                    .create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        ..Default::default()
                    });
            self.texture2d_sampler = Some(texture_sampler);
        }

        if self.texture_cube_sampler.is_none() {
            let texture_sampler =
                graphics_context
                    .get_device()
                    .create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("PBR CubeTexture Sampler"),
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

    pub fn set_albedo_color(&mut self, albedo_color: Vec4) {
        self.albedo_color = albedo_color;
    }

    pub fn set_albedo_color_rgb(&mut self, rgb: &[f32; 3]) {
        self.albedo_color.x = rgb [0];
        self.albedo_color.y = rgb [1];
        self.albedo_color.z = rgb [2];
        self.albedo_color.w = 1.0;
    }

    pub fn get_albedo_color(&self) -> Vec4 {
        self.albedo_color
    }

    pub fn set_metallic(&mut self, metallic: f32) {
        self.metallic_roughness_ao.x = metallic;
    }

    pub fn get_metallic(&self) -> f32 {
        self.metallic_roughness_ao.x
    }

    pub fn set_roughness(&mut self, roughness: f32) {
        self.metallic_roughness_ao.y = roughness;
    }

    pub fn get_roughness(&self) -> f32 {
        self.metallic_roughness_ao.y
    }

    pub fn set_ao(&mut self, ao: f32) {
        self.metallic_roughness_ao.z = ao;
    }

    pub fn get_ao(&self) -> f32 {
        self.metallic_roughness_ao.z
    }

    pub fn get_metallic_roughness_ao(&self) -> Vec4 {
        self.metallic_roughness_ao
    }

    pub fn set_albedo_texture(&mut self, albedo_texture: Handle<Texture>) {
        self.albedo_texture = albedo_texture;
    }

    pub fn get_albedo_texture(&self) -> &Handle<Texture> {
        &self.albedo_texture
    }

    pub fn set_normal_texture(&mut self, normal_texture: Handle<Texture>) {
        self.normal_textue = normal_texture;
    }

    pub fn get_normal_texture(&self) -> &Handle<Texture> {
        &self.normal_textue
    }

    pub fn set_metallic_texture(&mut self, metallic_texture: Handle<Texture>) {
        self.metallic_texture = metallic_texture;
    }

    pub fn get_metallic_texture(&self) -> &Handle<Texture> {
        &self.metallic_texture
    }

    pub fn set_roughness_texture(&mut self, roughness_texture: Handle<Texture>) {
        self.roughness_texture = roughness_texture;
    }

    pub fn get_roughness_texture(&self) -> &Handle<Texture> {
        &self.roughness_texture
    }

    pub fn set_ao_texture(&mut self, ao_texture: Handle<Texture>) {
        self.ao_texture = ao_texture;
    }

    pub fn get_ao_texture(&self) -> &Handle<Texture> {
        &self.ao_texture
    }

    pub fn set_irradiance_cube_texture(&mut self, irradiance_cube_texture: Handle<Texture>) {
        self.irradiance_cube_texture = irradiance_cube_texture;
    }

    pub fn get_irradiance_cube_texture(&self) -> &Handle<Texture> {
        &self.irradiance_cube_texture
    }

    pub fn set_prefiltered_cube_texture(&mut self, prefiltered_cube_texture: Handle<Texture>) {
        self.prefiltered_cube_texture = prefiltered_cube_texture;
    }

    pub fn get_prefiltered_cube_texture(&self) -> &Handle<Texture> {
        &self.prefiltered_cube_texture
    }

    pub fn set_brdf_lut(&mut self, brdf_lut: Handle<Texture>) {
        self.brdf_lut = brdf_lut;
    }

    pub fn get_brdf_lut(&self) -> &Handle<Texture> {
        &self.brdf_lut
    }

    pub fn get_2d_texture_sampler(&self) -> &wgpu::Sampler {
        self.texture2d_sampler.as_ref().unwrap()
    }

    pub fn get_cube_texture_sampler(&self) -> &wgpu::Sampler {
        self.texture_cube_sampler.as_ref().unwrap()
    }

    // PBR related features, corresponding to WGSL feature flags.
    const FEATURE_FLAG_ALBEDO_MAP: u32 = 1 << 0;
    const FEATURE_FLAG_NORMAL_MAP: u32 = 1 << 1;
    const FEATURE_FLAG_METALLIC_MAP: u32 = 1 << 2;
    const FEATURE_FLAG_ROUGHNESS_MAP: u32 = 1 << 3;
    const FEATURE_FLAG_AO_MAP: u32 = 1 << 4;
    const FEATURE_FLAG_IBL: u32 = 1 << 5;

    pub fn get_enabled_features(&self) -> [u32; 4] {
        let mut features: u32 = 0;
        if self.albedo_texture != Handle::INVALID && self.albedo_texture != *Texture::white() {
            features |= Self::FEATURE_FLAG_ALBEDO_MAP; // 1 << 0
        }
        if self.normal_textue != Handle::INVALID && self.normal_textue != *Texture::white() {
            // TODO: determine the default normal map.
            features |= Self::FEATURE_FLAG_NORMAL_MAP; // 1 << 1
        }
        if self.metallic_texture != Handle::INVALID && self.metallic_texture != *Texture::white() {
            features |= Self::FEATURE_FLAG_METALLIC_MAP; // 1 << 2
        }
        if self.roughness_texture != Handle::INVALID && self.roughness_texture != *Texture::white() {
            features |= Self::FEATURE_FLAG_ROUGHNESS_MAP;
        }
        if self.ao_texture != Handle::INVALID && self.ao_texture != *Texture::white() {
            features |= Self::FEATURE_FLAG_AO_MAP;
        }
        if self.irradiance_cube_texture != Handle::INVALID
            && self.irradiance_cube_texture != *Texture::cube_texture_placeholder()
            && self.prefiltered_cube_texture != Handle::INVALID
            && self.prefiltered_cube_texture != *Texture::cube_texture_placeholder()
            && self.brdf_lut != Handle::INVALID
            && self.brdf_lut != *Texture::white()
        {
            features |= Self::FEATURE_FLAG_IBL;
        }

        [features, 0, 0, 0]
    }
}
