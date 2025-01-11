use crate::{prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, texture_manager::TextureManager, GraphicsContext}, types::ID};

pub trait MaterialTrait {
    fn init(&mut self, graphics_context: &GraphicsContext, bind_group_layout_manager: &mut BindGroupLayoutManager);

    fn create_bind_group(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager, texture_manager: &TextureManager) -> ID;

    fn get_bind_group_layout_id(&self) -> ID;
    fn get_bind_group_id(&self) -> ID;

    fn create_shader_module(&self, graphics_context: &GraphicsContext) -> wgpu::ShaderModule;
}