use std::{borrow::Cow, cell::RefCell, f32::consts, rc::Rc};

use log::info;
use imagic::prelude::*;

pub struct App {
    cube_render_item_index: usize,
    window_size: (f64, f64),
}

impl Default for App {
    fn default() -> Self {
        Self {
            cube_render_item_index: usize::MAX,
            window_size: (500.0, 500.0),
        }
    }
}

impl App {
    fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(1.5f32, -5.0, 3.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );
        projection * view
    }
    
    fn create_bind_group_layout(&self, graphics_context: &GraphicsContext) -> wgpu::BindGroupLayout {
        let bind_group_layout =
        graphics_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("pbr_demo_bind_group_layout"),
        });
        bind_group_layout
    }

    fn create_bind_group(&mut self, imagic: &mut Imagic, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        let mx_total = Self::generate_matrix(self.window_size.0 as f32 / self.window_size.1 as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();

        let uniform_buf = imagic.context().graphics_context().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture = Texture::create_from_bytes(imagic.context().graphics_context(),
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let texture_view = texture.get_texture_view();

        let texture_sampler = imagic.context().graphics_context().get_device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = imagic.context().graphics_context().create_bind_group(&wgpu::BindGroupDescriptor{
            layout: bind_group_layout,
            label: Some("Cube bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                }
            ]
        });
        bind_group
    }

    fn create_render_pipeline(&self, imagic: &mut Imagic) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let graphics_context = imagic.context().graphics_context();
        let bind_group_layout = self.create_bind_group_layout(graphics_context);
        let vertex_buffer_layout = Cube::vertex_buffer_layout();

        // Load the shaders from disk
        let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../src/shaders/pbr.wgsl"))),
        });

        let pipeline_layout = graphics_context.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_format = graphics_context.get_swapchian_format();

        let render_pipeline = graphics_context.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            // cache: None,
        });
        (render_pipeline, bind_group_layout)
    }

    fn create_buffers(&mut self, imagic: &mut Imagic) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        Cube::create_buffer(imagic.context().graphics_context().get_device())
    }
    
    fn init(&mut self, imagic: &mut Imagic) {
        let (cube_vertex_buffer, cube_index_buffer, index_count) = self.create_buffers(imagic);
        let buffer_manager = imagic.context_mut().buffer_manager_mut();
        let cube_vertex_buffer_index = buffer_manager.add_buffer(cube_vertex_buffer);
        let cube_index_buffer_index = buffer_manager.add_buffer(cube_index_buffer);
        let (render_pipeline, bind_group_layout) = self.create_render_pipeline(imagic);
        let pipeline_id = imagic.context_mut().pipeline_manager_mut().add_render_pipeline(render_pipeline);
        let bind_group = self.create_bind_group(imagic, &bind_group_layout);
        let bind_group_id = imagic.context_mut().bind_group_manager_mut().add_bind_group(bind_group);

        let mut cube_item = RenderItem::new(
            VertexOrIndexCount::IndexCount { index_count, base_vertex: 0, instance_count: 1, index_format: Cube::index_buffer_format() },
            cube_vertex_buffer_index, cube_index_buffer_index, usize::MAX, true);
        cube_item.set_pipeline(pipeline_id);
        cube_item.set_bind_groups(vec![bind_group_id]);
        self.cube_render_item_index = imagic.context_mut().render_item_manager_mut().add_render_item(cube_item);
    }

    pub fn run(mut self) {
        let mut imagic = Imagic::new();
        let event_loop = imagic.init(ImagicOption::new(self.window_size.0, self.window_size.1, "PBR Demo"));

        self.init(&mut imagic);

        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(event_loop, app);
    }
}

impl ImagicAppTrait for App {
    fn on_update(&mut self, _imagic_context: &mut ImagicContext, _ui_renderer: &mut UIRenderer) {
        // todo!()
    }

    fn on_render_ui(&mut self, _ctx: &egui::Context) {
        // todo!()
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("pbr main.");
    let app: App = Default::default();
    app.run();
}