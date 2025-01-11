use crate::{graphics::GraphicsContext, types::ID};

pub struct BindGroupManager {
    bind_groups: Vec<wgpu::BindGroup>,
}

impl Default for BindGroupManager {
    fn default() -> Self {
        Self {
            bind_groups: Vec::new()
        }
    }
}

impl BindGroupManager {
    pub fn add_bind_group(&mut self, bind_group: wgpu::BindGroup) -> ID {
        let id = self.bind_groups.len();
        self.bind_groups.push(bind_group);
        id
    }

    pub fn get_bind_group(&self, index: usize) -> &wgpu::BindGroup {
        if index > self.bind_groups.len() {
            panic!("error: index out of bounds. index: {}, vec len: {} ", index, self.bind_groups.len());
        }
        &self.bind_groups[index]
    }
}

pub fn create_default_bind_group(graphics_context: &GraphicsContext, bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView, texture_sampler: &wgpu::Sampler) -> wgpu::BindGroup {
    let diffuse_bind_group = graphics_context.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                }
            ],
            label: Some("default_bind_group"),
        }
    );
    diffuse_bind_group
}