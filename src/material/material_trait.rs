use std::any::Any;

use crate::{
    asset::asset_manager::AssetManager, prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, GraphicsContext,
    }, types::ID
};

pub trait MaterialTrait : Any {

    /// Create BindGroupLayout and so on, e.g., create texture sampler.
    fn on_init(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
    );

    /// Create shader module. It will be called when create renderpipeline.
    fn create_shader_module(&self, graphics_context: &GraphicsContext) -> wgpu::ShaderModule;

    /// create bind group.
    fn create_bind_group(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        // texture_manager: &TextureManager,
        asset_manager: &AssetManager,
    ) -> ID;

    fn get_bind_group_layout_id(&self) -> ID;
    fn get_bind_group_id(&self) -> ID;

    fn get_cull_mode(&self) -> wgpu::Face {
        wgpu::Face::Back
    }

    #[allow(unused)]
    fn set_cull_mode(&mut self, cull_mode: wgpu::Face) {

    }

    fn get_front_face(&self) -> wgpu::FrontFace {
        wgpu::FrontFace::Ccw
    }

    #[allow(unused)]
    fn set_front_face(&mut self, front_face: wgpu::FrontFace) {
        
    }

    fn enable_lights(&self) -> bool {
        true
    }

    /// Update material parameters, e.g., update uniform buffers.
    /// 
    /// Do not call this method directly.
    /// 
    /// Users should call ImagicContext.update_material(material_id) instead,
    /// which will call this method and pass essential arguments. 
    #[allow(unused)]
    fn on_update(&mut self, graphics_context: &GraphicsContext) {
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
    }
}
