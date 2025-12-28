use std::{collections::HashMap, rc::Rc};

use log::info;
use wgpu::{ColorTargetState, Device, VertexBufferLayout};

use crate::{assets::{Shader, materials::material::Material}, graphics::{graphics_context::GraphicsLimits, render_states::PolygonMode}};

pub type RenderPipeLine = wgpu::RenderPipeline;

pub type PipelineHashType = u64;
pub const INVALID_PIPELINE_HASH: PipelineHashType = PipelineHashType::MAX;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct PipelineKey {
    material_hash: PipelineHashType,
}

pub struct RenderPipelineManager {
    device: Rc<Device>,
    limits: Rc<GraphicsLimits>,
    pipelines: HashMap<PipelineHashType, RenderPipeLine>,
}

impl RenderPipelineManager {
    pub fn new(device: Rc<Device>, limits: Rc<GraphicsLimits>) -> Self {
        Self {
            device,
            limits,
            pipelines: HashMap::new(),
        }
    }

    pub fn get(&self, hash: PipelineHashType) -> Option<&RenderPipeLine> {
        self.pipelines.get(&hash)
    }

    pub fn insert(
        &mut self,
        hash: PipelineHashType,
        pipeline: RenderPipeLine,
    ) -> Option<&RenderPipeLine> {
        self.pipelines.insert(hash, pipeline);
        self.pipelines.get(&hash)
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
    }

    pub fn contains(&self, hash: PipelineHashType) -> bool {
        self.pipelines.contains_key(&hash)
    }

    /// Create render pipeline.
    pub(crate) fn create_render_pipeline(
        &mut self,
        pipeline_hash: PipelineHashType,
        material: &Material,
        shader: &Shader,
        vertex_buffer_layouts: &[VertexBufferLayout],
        targets: &[Option<ColorTargetState>],
        depth_format: wgpu::TextureFormat,
    ) {
        if cfg!(debug_assertions) {
            info!("Create render pipeline with hash: {}", pipeline_hash);
        }
        
        let pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &shader.get_bind_group_layouts(),
                    push_constant_ranges: &[],
                });
        let polygon_mode = if material.render_state.polygon_mode == PolygonMode::Fill {
            PolygonMode::Fill
        } else if material.render_state.polygon_mode == PolygonMode::Line {
            if !self.limits.is_support_polygon_mode_line {
                log::warn!("The current device does not support PolygonMode::Line, fallback to PolygonMode::Fill");
                PolygonMode::Fill
            } else {
                PolygonMode::Line
            }
        } else {
            if !self.limits.is_support_polygon_mode_point {
                log::warn!("The current device does not support PolygonMode::Point, fallback to PolygonMode::Fill");
                PolygonMode::Fill
            } else {
                PolygonMode::Point
            }
        };
        
        let cull_mode = match material.render_state.cull_mode {
            crate::graphics::render_states::CullMode::Front => Some(wgpu::Face::Front),
            crate::graphics::render_states::CullMode::Back => Some(wgpu::Face::Back),
            crate::graphics::render_states::CullMode::None => None,
        };
        
        let render_pipeline =
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("create pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: shader.get_shader_module(),
                        entry_point: Some("vs_main"),
                        buffers: vertex_buffer_layouts,
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: shader.get_shader_module(),
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: targets,
                    }),
                    primitive: wgpu::PrimitiveState {
                        cull_mode: cull_mode,
                        front_face: wgpu::FrontFace::Ccw,
                        polygon_mode: polygon_mode,
                        ..Default::default()
                    },
                    // depth_stencil: None,
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: depth_format,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });
        self.insert(pipeline_hash, render_pipeline);
    }
}
