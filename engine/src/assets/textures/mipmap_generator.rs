use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use ahash::{AHashMap, AHasher};
use wgpu::{Device, Queue};

use crate::assets::{Texture, TextureViewDimension};

/// Generate mipmaps for given gpu texture.
pub(crate) struct MipmapGenerator {
    render_pipelines: AHashMap<u64, wgpu::RenderPipeline>,
    sampler: wgpu::Sampler,
    device: Rc<Device>,
    queue: Rc<Queue>,
}

impl MipmapGenerator {
    pub(crate) fn new(device: Rc<Device>, queue: Rc<Queue>) -> Self {
        let render_pipelines = AHashMap::<u64, wgpu::RenderPipeline>::new();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mipmap generator"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            render_pipelines,
            sampler,
            device,
            queue,
        }
    }

    fn ensure_create_pipeline(&mut self, texture: &mut Texture) -> u64 {
        let shader = self
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/wgsl/blit.wgsl"));
        let mut hasher = AHasher::default();
        texture.format.hash(&mut hasher);
        let pipeline_hash = hasher.finish();
        if !self.render_pipelines.contains_key(&pipeline_hash) {
            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("blit"),
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        // note: texture format may be different.
                        targets: &[Some(texture.format.into())],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });
            self.render_pipelines.insert(pipeline_hash, pipeline);
        }
        pipeline_hash
    }

    pub(crate) fn generate_mipmap(&mut self, texture: &mut Texture) {
        self.generate_mipmap_core(texture);
    }

    pub(crate) fn generate_mipmap_core(&mut self, texture: &mut Texture) {
        let pipeline_hash = self.ensure_create_pipeline(texture);
        let render_pipeline = self.render_pipelines.get(&pipeline_hash);
        if let Some(pipeline) = render_pipeline
            && let Some(gpu_texture) = &texture.gpu_texture
        {
            let bind_group_layout = pipeline.get_bind_group_layout(0);
            let mip_count = texture.mip_level_count;
            let face_count = texture.size.depth_or_array_layers;
            let all_face_views = (0..face_count).map(|face_index| {
                let cur_face_views = 
                (0..mip_count)
                .map(|mip| {
                    gpu_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("mip"),
                        format: None,//Some(TextureFormat::Rgba32Float),
                        dimension: Some(TextureViewDimension::D2),
                        usage: None,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: mip,
                        mip_level_count: Some(1),
                        base_array_layer: face_index,
                        array_layer_count: Some(1),
                    })
                })
                .collect::<Vec<_>>();
            cur_face_views
            }).collect::<Vec<_>>();
            

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mipmap generator command encoder"),
                });

            for views in &all_face_views {
                for target_mip in 1..mip_count as usize {
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                // resource: wgpu::BindingResource::TextureView(&views[target_mip - 1]),
                                // TODO: fix bug here, should use previous mip level as source. But currently it has issue when generating mipmaps for cube map texture.
                                resource: wgpu::BindingResource::TextureView(&views[0]),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                        label: None,
                    });
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Mipmap generator pass."),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &views[target_mip],
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.draw(0..3, 0..1);
                }
            }
            self.queue.submit(Some(encoder.finish()));
        } else {
            log::error!("Failed to generate mipmaps. No GPU texture or failed to create render pipeline");
        }
    }
}
