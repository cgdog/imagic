use std::borrow::Cow;

use crate::prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, texture_manager::TextureManager, GraphicsContext};

use super::material::MaterialTrait;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PBRFragmentUniforms {
    albedo: [f32; 4],
    metallic_roughness_ao: [f32; 4],
}

pub struct PBRMaterial {
    albedo_color: glam::Vec4,
    metallic_roughness_ao: glam::Vec4,

    albedo_texture: usize,
    normal_textue: usize,
    metallic_texture: usize,
    roughness_texture: usize,
    ao_texture: usize,

    texture2d_sampler: Option<wgpu::Sampler>,

    bind_group_id: usize,
}

impl Default for PBRMaterial {

    fn default() -> Self {
        Self {
            albedo_color: glam::Vec4::ONE,
            metallic_roughness_ao: glam::Vec4::ONE,
            albedo_texture: usize::MAX,
            normal_textue: usize::MAX,
            metallic_texture: usize::MAX,
            roughness_texture: usize::MAX,
            ao_texture: usize::MAX,
            texture2d_sampler: None,

            bind_group_id: usize::MAX,
        }
    }
}

impl MaterialTrait for PBRMaterial {
    fn init(&mut self, graphics_context: &GraphicsContext, bind_group_layout_manager: &mut BindGroupLayoutManager) {
        self.create_texture_sampler(graphics_context);

        PBRMaterial::try_create_bind_group_layout_id(graphics_context, bind_group_layout_manager);
    }

    fn create_shader_module(&self, graphics_context: &GraphicsContext) -> wgpu::ShaderModule {
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/pbr.wgsl"))),
        });
        shader
    }

    fn create_bind_group(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager
        , bind_group_layout_manager: &mut BindGroupLayoutManager, texture_manager: &TextureManager) -> usize {

        let fragment_uniforms = PBRFragmentUniforms {
            albedo: self.albedo_color.to_array(),
            metallic_roughness_ao: self.metallic_roughness_ao.to_array(),
        };

        let fragment_uniform_buffer = graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PBR Fragment Uniform Buffer"),
            contents: bytemuck::cast_slice(&[fragment_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = bind_group_layout_manager.get_bind_group_layout(self.get_bind_group_layout_id());
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor{
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
                wgpu::BindGroupEntry {// albedo map
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(texture_manager.get_texture_view(self.albedo_texture)),
                },
                wgpu::BindGroupEntry {// normal map
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(texture_manager.get_texture_view(self.normal_textue)),
                },
                wgpu::BindGroupEntry {// metallic map
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(texture_manager.get_texture_view(self.metallic_texture)),
                },
                wgpu::BindGroupEntry {// roughness map
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(texture_manager.get_texture_view(self.roughness_texture)),
                },
                wgpu::BindGroupEntry {// ao map
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(texture_manager.get_texture_view(self.ao_texture)),
                },
            ]
        });
        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        self.bind_group_id
    }

    fn get_bind_group_id(&self) -> usize {
        self.bind_group_id
    }

    fn get_bind_group_layout_id(&self) -> usize {
        PBRMaterial::internal_bind_group_layout_id(usize::MAX)
    }
}

impl PBRMaterial {
    pub fn new(albedo_color: glam::Vec4, metallic: f32, roughness: f32, ao: f32) -> Self {
        Self {
            albedo_color,
            metallic_roughness_ao: glam::Vec4::new(metallic, roughness, ao, 1.0),
            ..Default::default()
        }
    }

    fn internal_bind_group_layout_id(new_id: usize) -> usize {
        // All PBRMaterial instances share the same bind group layout.
        static mut LAYOUT_ID: usize = usize::MAX;
        if new_id != usize::MAX {
            unsafe { LAYOUT_ID = new_id };
        }
        unsafe { LAYOUT_ID }
    }

    fn try_create_bind_group_layout_id(graphics_context: &GraphicsContext, bind_group_layout_manager: &mut BindGroupLayoutManager) {
        let layout_id = PBRMaterial::internal_bind_group_layout_id(usize::MAX);
        if layout_id == usize::MAX {
            let bind_group_layout = PBRMaterial::create_bind_group_layout(graphics_context);
            let layout_id = bind_group_layout_manager.add_bind_group_layout(bind_group_layout);
            PBRMaterial::internal_bind_group_layout_id(layout_id);
        }
    }

    fn create_bind_group_layout(graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
        graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {// FragmentUniforms: albedo color, metallic_roughness_ao
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16 * 2),
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
                wgpu::BindGroupLayoutEntry {// albedo map
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {// normal map
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {// metallic map
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {// roughness map
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {// ao map
                    binding: 6,
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

    pub fn set_albedo_color(&mut self, albedo_color: glam::Vec4) {
        self.albedo_color = albedo_color;
    }

    pub fn get_albedo_color(&self) -> glam::Vec4 {
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

    pub fn get_metallic_roughness_ao(&self) -> glam::Vec4 {
        self.metallic_roughness_ao
    }

    pub fn set_albedo_texture(&mut self, albedo_texture: usize) {
        self.albedo_texture = albedo_texture;
    }

    pub fn get_albedo_texture(&self) -> usize {
        self.albedo_texture
    }

    pub fn set_normal_texture(&mut self, normal_texture: usize) {
        self.normal_textue = normal_texture;
    }

    pub fn get_normal_texture(&self) -> usize {
        self.normal_textue
    }

    pub fn set_metallic_texture(&mut self, metallic_texture: usize) {
        self.metallic_texture = metallic_texture;
    }

    pub fn get_metallic_texture(&self) -> usize {
        self.metallic_texture
    }

    pub fn set_roughness_texture(&mut self, roughness_texture: usize) {
        self.roughness_texture = roughness_texture;
    }

    pub fn get_roughness_texture(&self) -> usize {
        self.roughness_texture
    }

    pub fn set_ao_texture(&mut self, ao_texture: usize) {
        self.ao_texture = ao_texture;
    }

    pub fn get_ao_texture(&self) -> usize {
        self.ao_texture
    }

    pub fn get_2d_texture_sampler(&self, ) -> &wgpu::Sampler {
        self.texture2d_sampler.as_ref().unwrap()
    }

}