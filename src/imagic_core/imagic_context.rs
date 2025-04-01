use std::cell::RefCell;

use wgpu::TextureFormat;
use winit::dpi::{LogicalSize, PhysicalSize};

use crate::{
    asset::asset_manager::AssetManager, camera::{Camera, CameraControllerOptions}, graphics::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager,
        buffer::GPUBufferManager, render_pipeline::RenderPipelineManager, GraphicsContext,
    }, input::InputManager, prelude::{
        texture_manager::TextureManager, CameraManager, LightManager, MaterialManager, MaterialTrait, RenderItem, Texture, TransformManager
    }, types::{ID, RR}, window::WindowSize
};

use super::render_item::render_item_manager::RenderItemManager;

#[derive(Default)]
pub struct ImagicContext {
    graphics_context: GraphicsContext,
    bind_group_layout_manager: BindGroupLayoutManager,
    bind_group_manager: BindGroupManager,
    pipeline_manager: RefCell<RenderPipelineManager>,
    render_item_manager: RenderItemManager,
    buffer_manager: GPUBufferManager,
    light_manager: LightManager,
    material_manager: MaterialManager,
    texture_manager: TextureManager,
    transform_manager: RR<TransformManager>,
    camera_manager: CameraManager,
    input_manager: InputManager,
    asset_manager: AssetManager,

    logical_size: WindowSize,
    physical_size: WindowSize,
}

impl ImagicContext {
    pub fn init(&mut self, logical_size: WindowSize, physical_size: WindowSize) {
        self.logical_size = logical_size;
        self.physical_size = physical_size;
        self.bind_group_layout_manager.init(&self.graphics_context);
        self.pipeline_manager
            .borrow_mut().init(&self.graphics_context, &self.bind_group_layout_manager);
        self.init_default_assets();
    }

    fn init_default_assets(&mut self) {
        Texture::_internal_create_default_textures(Some(&self.graphics_context), &mut self.asset_manager);
    }

    pub(crate) fn asset_manager(&self) -> &AssetManager {
        &self.asset_manager
    }

    pub(crate) fn asset_manager_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_manager
    }

    /// Called after App.init()
    pub(crate) fn init_after_app(&mut self) {
        // self.camera_manager.init_after_app(
        //     window,
        //     &self.graphics_context,
        //     &mut self.bind_group_manager,
        //     &mut self.bind_group_layout_manager,
        //     self.transform_manager.clone(),
        //     &mut self.buffer_manager,
        //     &mut self.texture_manager,
        //     &mut self.input_manager,
        // );
        self.light_manager.init_after_app(
            &self.graphics_context,
            &mut self.bind_group_manager,
            &mut self.bind_group_layout_manager,
            &self.transform_manager.borrow(),
        );
        // self.material_manager.init_after_app(
        //     &self.graphics_context,
        //     &mut self.bind_group_manager,
        //     &mut self.bind_group_layout_manager,
        //     &self.texture_manager,
        // );
        // self.render_item_manager.init_after_app(
        //     &self.graphics_context,
        //     &mut self.bind_group_manager,
        //     &mut self.bind_group_layout_manager,
        //     &self.material_manager,
        //     &self.transform_manager.borrow(),
        //     &mut self.pipeline_manager,
        // );
    }

    /// Update imageic context after App.on_update().
    /// Dirty uniform buffers will be updated and uploaded to GPU here.
    pub fn on_update(&mut self) {
        self.camera_manager.on_update(
            &self.graphics_context,
            &self.transform_manager.borrow(),
            &self.buffer_manager,
        );

        self.input_manager.on_update();

        // self.material_manager.on_update(&self.graphics_context);
    }

    pub fn update_material(&mut self, material_id: ID) {
        self.material_manager.get_material_mut(material_id).on_update(&self.graphics_context);
    }

    pub fn on_resize(
        &mut self,
        new_physical_size: PhysicalSize<u32>,
        new_logical_size: LogicalSize<u32>,
    ) {
        self.physical_size.set(
            new_physical_size.width as f32,
            new_physical_size.height as f32,
        );
        self.logical_size.set(
            new_logical_size.width as f32,
            new_logical_size.height as f32,
        );

        self.graphics_context.on_resize(new_physical_size);
        self.camera_manager.on_resize(
            &self.graphics_context,
            &mut self.texture_manager,
            &self.transform_manager.borrow(),
            &self.buffer_manager,
            new_physical_size.width,
            new_physical_size.height,
            new_logical_size.width,
            new_logical_size.height,
        );
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

    // pub fn pipeline_manager_mut(&mut self) -> &mut RenderPipelineManager {
    //     &mut self.pipeline_manager
    // }

    // pub fn pipeline_manager(&self) -> &RenderPipelineManager {
    //     &self.pipeline_manager
    // }

    pub fn pipeline_manager(&self) -> &RefCell<RenderPipelineManager> {
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

    pub fn transform_manager(&self) -> RR<TransformManager> {
        self.transform_manager.clone()
    }

    // pub fn transform_manager_mut(&mut self) -> RR<TransformManager> {
    //     self.transform_manager.clone()
    // }

    pub fn camera_manager(&self) -> &CameraManager {
        &self.camera_manager
    }

    pub fn camera_manager_mut(&mut self) -> &mut CameraManager {
        &mut self.camera_manager
    }

    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    pub fn input_manager_mut(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }

    pub fn change_camera_controller(
        &mut self,
        camera_id: ID,
        camera_controller_options: &CameraControllerOptions,
    ) {
        self.camera_manager.change_camera_controller(
            &mut self.input_manager,
            camera_id,
            camera_controller_options,
        );
    }

    pub fn add_camera(&mut self, camera: Camera) -> ID {
        self.camera_manager.add_camera(
            camera,
            &self.logical_size,
            &self.physical_size,
            &self.graphics_context,
            &mut self.bind_group_manager,
            &mut self.bind_group_layout_manager,
            self.transform_manager.clone(),
            &mut self.buffer_manager,
            &mut self.texture_manager,
            &mut self.input_manager,
        )
    }

    pub fn add_material(&mut self, material: Box<dyn MaterialTrait>) -> ID {
        self.material_manager.add_material(
            material,
            &self.graphics_context,
            &mut self.bind_group_manager,
            &mut self.bind_group_layout_manager,
            &self.asset_manager,
        )
    }

    pub fn add_render_item(&mut self, render_item: RenderItem) -> ID {
        self.render_item_manager.add_render_item(
            render_item,
            &self.graphics_context,
            &mut self.bind_group_manager,
            &mut self.bind_group_layout_manager,
            &self.transform_manager.borrow(),
        )
    }

    /// TODO: replace item_id with feature hash
    pub fn create_pipeline(&self, item_id: ID, color_attachment_format: Option<TextureFormat>, material: &Box<dyn MaterialTrait>) {
        self.pipeline_manager.borrow_mut().create_pipeline(
            item_id,
            color_attachment_format,
            &self.graphics_context,
            &self.bind_group_layout_manager,
            material,
        );
    }

    // pub fn add_point_light(&mut self, light: PointLight) -> ID {
    // }
}
