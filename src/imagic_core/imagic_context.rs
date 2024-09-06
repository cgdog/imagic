use crate::{graphics::{
    bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager,
    buffer::GPUBufferManager, render_pipeline::RenderPipelineManager, GraphicsContext,
}, prelude::{texture_manager::TextureManager, CameraManager, LightManager, MaterialManager, TransformManager}, window::Window};

use super::render_item::render_item_manager::RenderItemManager;

#[derive(Default)]
pub struct ImagicContext {
    graphics_context: GraphicsContext,
    bind_group_layout_manager: BindGroupLayoutManager,
    bind_group_manager: BindGroupManager,
    pipeline_manager: RenderPipelineManager,
    render_item_manager: RenderItemManager,
    buffer_manager: GPUBufferManager,
    light_manager: LightManager,
    material_manager: MaterialManager,
    texture_manager: TextureManager,
    transform_manager: TransformManager,
    camera_manager: CameraManager,
}

impl ImagicContext {
    pub fn init(&mut self) {
        self.bind_group_layout_manager.init(&self.graphics_context);
        self.pipeline_manager.init(&self.graphics_context, &self.bind_group_layout_manager);
    }

    pub fn init_after_app(&mut self, window: &Window) {
        self.camera_manager.init_after_app(window, &self.graphics_context, &mut self.bind_group_manager, &mut self.bind_group_layout_manager, &self.transform_manager, &mut self.buffer_manager);
        self.light_manager.init_after_app(&self.graphics_context, &mut self.bind_group_manager, &mut self.bind_group_layout_manager, &self.transform_manager);
        self.material_manager.init_after_app(&self.graphics_context, &mut self.bind_group_manager, &mut self.bind_group_layout_manager, &self.texture_manager);
        self.render_item_manager.init_after_app(&self.graphics_context, &mut self.bind_group_manager
            , &mut self.bind_group_layout_manager, &self.material_manager, &self.transform_manager, &mut self.pipeline_manager);
    }

    pub fn graphics_context_mut(&mut self) -> &mut GraphicsContext {
        &mut self.graphics_context
    }

    pub fn graphics_context(&self) -> &GraphicsContext {
        &self.graphics_context
    }

    pub fn bind_group_layout_manager_mut(&mut self) -> &mut BindGroupLayoutManager {
        &mut self.bind_group_layout_manager
    }

    pub fn bind_group_layout_manager(&self) -> &BindGroupLayoutManager {
        &self.bind_group_layout_manager
    }

    pub fn bind_group_manager_mut(&mut self) -> &mut BindGroupManager {
        &mut self.bind_group_manager
    }

    pub fn bind_group_manager(&self) -> &BindGroupManager {
        &self.bind_group_manager
    }

    pub fn pipeline_manager_mut(&mut self) -> &mut RenderPipelineManager {
        &mut self.pipeline_manager
    }

    pub fn pipeline_manager(&self) -> &RenderPipelineManager {
        &self.pipeline_manager
    }

    pub fn render_item_manager_mut(&mut self) -> &mut RenderItemManager {
        &mut self.render_item_manager
    }

    pub fn render_item_manager(&self) -> &RenderItemManager {
        &self.render_item_manager
    }

    pub fn buffer_manager(&self) -> &GPUBufferManager {
        &self.buffer_manager
    }

    pub fn buffer_manager_mut(&mut self) -> &mut GPUBufferManager {
        &mut self.buffer_manager
    }

    pub fn light_manager(&self) -> &LightManager {
        &self.light_manager
    }

    pub fn light_manager_mut(&mut self) -> &mut LightManager {
        &mut self.light_manager
    }

    pub fn material_manager(&self) -> &MaterialManager {
        &self.material_manager
    }

    pub fn material_manager_mut(&mut self) -> &mut MaterialManager {
        &mut self.material_manager
    }

    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }

    pub fn texture_manager_mut(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
    }

    pub fn transform_manager(&self) -> &TransformManager {
        &self.transform_manager
    }

    pub fn transform_manager_mut(&mut self) -> &mut TransformManager {
        &mut self.transform_manager
    }

    pub fn camera_manager(&self) -> &CameraManager {
        &self.camera_manager
    }

    pub fn camera_manager_mut(&mut self) -> &mut CameraManager {
        &mut self.camera_manager
    }
}
