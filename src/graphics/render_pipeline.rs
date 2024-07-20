use std::borrow::Cow;
use wgpu::BindGroupLayout;
use crate::prelude::{MaterialTrait, Vertex};

use super::{bind_group_layout::BindGroupLayoutManager, graphics_context::GraphicsContext};

pub struct RenderPipelineManager {
    render_pipelines: Vec<wgpu::RenderPipeline>,
    default_pbr_pipeline: usize,
}

impl Default for RenderPipelineManager {
    fn default() -> Self {
        Self {
            render_pipelines: Vec::new(),
            default_pbr_pipeline: usize::MAX,
        }
    }
}

impl RenderPipelineManager {
    pub fn add_render_pipeline(&mut self, render_pipeline: wgpu::RenderPipeline) -> usize {
        let id = self.render_pipelines.len();
        self.render_pipelines.push(render_pipeline);
        id
    }

    pub fn get_render_pipeline(&self, index: usize) -> &wgpu::RenderPipeline {
        if index > self.render_pipelines.len() {
            panic!("error: index out of bounds. index: {}, vec len: {} ", index, self.render_pipelines.len());
        }
        &self.render_pipelines[index]
    }

    pub fn default_pbr_render_pipeline(&self) -> usize {
        self.default_pbr_pipeline
    }

    pub fn init(&mut self, _graphics_context: &GraphicsContext, _bind_group_layout_manager: &BindGroupLayoutManager) {
        // let default_pbr_render_pipeline = RenderPipelineManager::create_pbr_pipeline(graphics_context, bind_group_layout_manager);
        // let default_pbr = self.add_render_pipeline(default_pbr_render_pipeline);
        // self.default_pbr_pipeline = default_pbr;
    }

    pub fn create_pipeline(&mut self, graphics_context: &GraphicsContext, bind_group_layout_manager: &BindGroupLayoutManager, material: &Box<dyn MaterialTrait>) -> usize {
        // let bind_group_layout = bind_group_layout_manager.default_pbr_bind_group_layout();
        let bind_group_layouts = [
            bind_group_layout_manager.default_model_vertex_bind_group_layout(),
            bind_group_layout_manager.get_camera_bind_group_layout(),
            bind_group_layout_manager.get_bind_group_layout(material.get_bind_group_layout_id()),
            bind_group_layout_manager.get_lighting_bind_group_layout(),
        ];
        let vertex_buffer_layout = Vertex::default_vertex_buffer_layout();

        let pipeline_layout = graphics_context.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let swapchain_format = graphics_context.get_swapchian_format();

        let shader = material.create_shader_module(graphics_context);
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
        self.add_render_pipeline(render_pipeline)
    }
}


pub fn create_default_render_pipeline(graphics_context: &GraphicsContext, bind_group_layout: &BindGroupLayout) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/full_screen.wgsl"))),
    });

    let pipeline_layout = graphics_context.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_format = graphics_context.get_swapchian_format();

    let render_pipeline = graphics_context.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        // cache: None,
    });
    render_pipeline
}
