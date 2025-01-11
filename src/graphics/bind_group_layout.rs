use wgpu::BindGroupLayout;
use crate::{graphics::GraphicsContext, prelude::INVALID_ID, types::ID};

pub struct BindGroupLayoutManager {
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    default_pbr_bind_group_layout: usize,
    model_vertex_bind_group_layout: usize,
    camera_bind_group_layout: usize,
    lighting_bind_group_layout: usize,
}

impl Default for BindGroupLayoutManager {
    fn default() -> Self {
        Self {
            bind_group_layouts: Vec::new(),
            default_pbr_bind_group_layout: INVALID_ID,
            model_vertex_bind_group_layout: INVALID_ID,
            camera_bind_group_layout: INVALID_ID,
            lighting_bind_group_layout: INVALID_ID,
        }
    }
}

impl BindGroupLayoutManager {
    pub fn add_bind_group_layout(&mut self, bind_group_layout: wgpu::BindGroupLayout) -> ID {
        let index = self.bind_group_layouts.len();
        self.bind_group_layouts.push(bind_group_layout);
        index
    }

    pub fn get_bind_group_layout(&self, index: usize) -> &wgpu::BindGroupLayout {
        if index > self.bind_group_layouts.len() {
            panic!("error: index out of bounds. index: {}, vec len: {} ", index, self.bind_group_layouts.len());
        }
        &self.bind_group_layouts[index]
    }

    pub fn init(&mut self, graphics_context: &GraphicsContext) {
        let model_vertex_bind_group_layout = self.create_model_vertex_bind_group_layout(graphics_context);
        self.model_vertex_bind_group_layout = self.add_bind_group_layout(model_vertex_bind_group_layout);
        let camera_bind_group_layout = self.create_camera_bind_group_layout(graphics_context);
        self.camera_bind_group_layout = self.add_bind_group_layout(camera_bind_group_layout);
        let lighting_bind_group_layout = self.create_lighting_bind_group_layout(graphics_context);
        self.lighting_bind_group_layout = self.add_bind_group_layout(lighting_bind_group_layout);
    }

    pub fn get_camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layouts[self.camera_bind_group_layout]
    }

    pub fn get_lighting_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layouts[self.lighting_bind_group_layout]
    }

    pub fn default_model_vertex_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layouts[self.model_vertex_bind_group_layout]
    }

    pub fn default_pbr_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layouts[self.default_pbr_bind_group_layout]
    }

    fn create_model_vertex_bind_group_layout(&self, graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout = graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            entries: &[
                wgpu::BindGroupLayoutEntry {// MVP
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16 * 4),
                    },
                    count: None,
                },
            ],
            label: Some("Model Vertex bind group layout"),
        });
        bind_group_layout
    }

    fn create_camera_bind_group_layout(&mut self, graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
        graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {// view matrix, projection matrix
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16 * 2 * 4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {// camera pos
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
            ],
            label: Some("Camera_bind_group_layout"),
        });
        bind_group_layout
    }

    fn create_lighting_bind_group_layout(&mut self, graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
        graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {// Lighting
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Camera_bind_group_layout"),
        });
        bind_group_layout
    }

    // fn create_pbr_bind_group_layout(graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
    //     let bind_group_layout =
    //     graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //         entries: &[
    //             wgpu::BindGroupLayoutEntry {// MVP
    //                 binding: 0,
    //                 visibility: wgpu::ShaderStages::VERTEX,
    //                 ty: wgpu::BindingType::Buffer {
    //                     ty: wgpu::BufferBindingType::Uniform,
    //                     has_dynamic_offset: false,
    //                     min_binding_size: wgpu::BufferSize::new(16*3*4),
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// FragmentUniforms
    //                 binding: 7,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Buffer {
    //                     ty: wgpu::BufferBindingType::Uniform,
    //                     has_dynamic_offset: false,
    //                     min_binding_size: wgpu::BufferSize::new(16 * 3),
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// Lighting
    //                 binding: 8,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Buffer {
    //                     ty: wgpu::BufferBindingType::Storage { read_only: true },
    //                     has_dynamic_offset: false,
    //                     min_binding_size: None,
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// sampler
    //                 binding: 1,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 // This should match the filterable field of the
    //                 // corresponding Texture entry.
    //                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// albedo map
    //                 binding: 2,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Texture {
    //                     multisampled: false,
    //                     view_dimension: wgpu::TextureViewDimension::D2,
    //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// normal map
    //                 binding: 3,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Texture {
    //                     multisampled: false,
    //                     view_dimension: wgpu::TextureViewDimension::D2,
    //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// metallic map
    //                 binding: 4,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Texture {
    //                     multisampled: false,
    //                     view_dimension: wgpu::TextureViewDimension::D2,
    //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// roughness map
    //                 binding: 5,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Texture {
    //                     multisampled: false,
    //                     view_dimension: wgpu::TextureViewDimension::D2,
    //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {// ao map
    //                 binding: 6,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Texture {
    //                     multisampled: false,
    //                     view_dimension: wgpu::TextureViewDimension::D2,
    //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                 },
    //                 count: None,
    //             },
    //         ],
    //         label: Some("pbr_demo_bind_group_layout"),
    //     });
    //     bind_group_layout
    // }
    
}

pub fn create_default_bind_group_layout(graphics_context: &GraphicsContext) -> BindGroupLayout {
    let bind_group_layout =
        graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("default_bind_group_layout"),
        });
    bind_group_layout
}