use crate::{
    prelude::{MaterialTrait, Vertex, INVALID_ID},
    types::ID,
};
use std::{borrow::Cow, collections::HashMap};
use wgpu::{BindGroupLayout, TextureFormat};

use super::{bind_group_layout::BindGroupLayoutManager, graphics_context::GraphicsContext};

pub struct RenderPipelineManager {
    // render_pipelines: Vec<wgpu::RenderPipeline>,
    render_pipelines: HashMap<ID, wgpu::RenderPipeline>,
    default_pbr_pipeline: usize,
}

impl Default for RenderPipelineManager {
    fn default() -> Self {
        Self {
            render_pipelines: HashMap::new(),
            default_pbr_pipeline: INVALID_ID,
        }
    }
}

impl RenderPipelineManager {
    /// TODO: replace item_id with feature hash.
    pub fn add_render_pipeline(&mut self, item_id: ID, render_pipeline: wgpu::RenderPipeline) {
        self.render_pipelines.insert(item_id, render_pipeline);
    }

    pub fn get_render_pipeline(&self, index: usize) -> Option<&wgpu::RenderPipeline> {
        // if index > self.render_pipelines.len() {
        //     panic!(
        //         "error: index out of bounds. index: {}, vec len: {} ",
        //         index,
        //         self.render_pipelines.len()
        //     );
        // }
        self.render_pipelines.get(&index)
    }

    pub fn remove_render_pipeline(&mut self, index: usize) {
        self.render_pipelines.remove(&index);
    }

    pub fn default_pbr_render_pipeline(&self) -> ID {
        self.default_pbr_pipeline
    }

    pub fn init(
        &mut self,
        _graphics_context: &GraphicsContext,
        _bind_group_layout_manager: &BindGroupLayoutManager,
    ) {
        // let default_pbr_render_pipeline = RenderPipelineManager::create_pbr_pipeline(graphics_context, bind_group_layout_manager);
        // let default_pbr = self.add_render_pipeline(default_pbr_render_pipeline);
        // self.default_pbr_pipeline = default_pbr;
    }

    pub fn create_pipeline(
        &mut self,
        item_id: ID,
        color_attachment_format: Option<TextureFormat>,
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &BindGroupLayoutManager,
        material: &Box<dyn MaterialTrait>,
    ) {
        // let bind_group_layout = bind_group_layout_manager.default_pbr_bind_group_layout();
        const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
        let mut bind_group_layouts = vec![
            bind_group_layout_manager.default_model_vertex_bind_group_layout(),
            bind_group_layout_manager.get_camera_bind_group_layout(),
            bind_group_layout_manager.get_bind_group_layout(material.get_bind_group_layout_id()),
        ];
        if material.enable_lights() {
            bind_group_layouts.push(bind_group_layout_manager.get_lighting_bind_group_layout());
        }
        let vertex_buffer_layout = Vertex::default_vertex_buffer_layout();

        let pipeline_layout =
            graphics_context.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        let swapchain_format = match color_attachment_format {
            Some(color_attachment_format_) => color_attachment_format_,
            _ => graphics_context.get_swapchian_format(),
        };

        let shader = material.create_shader_module(graphics_context);
        let render_pipeline =
            graphics_context.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("create pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_buffer_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    // TODO: 能自定义 render target format
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(material.get_cull_mode()),
                    front_face: material.get_front_face(),
                    ..Default::default()
                },
                // depth_stencil: None,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        self.add_render_pipeline(item_id, render_pipeline);
    }
}

pub fn create_default_render_pipeline(
    graphics_context: &GraphicsContext,
    bind_group_layout: &BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = graphics_context.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../shaders/full_screen.wgsl"
        ))),
    });

    let pipeline_layout =
        graphics_context.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

    let swapchain_format = graphics_context.get_swapchian_format();

    let render_pipeline =
        graphics_context.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
    render_pipeline
}
