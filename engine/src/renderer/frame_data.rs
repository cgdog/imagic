use crate::{
    assets::{TextureHandle, meshes::vertex_index::IndexFormat},
    core::node::NodeId,
    graphics::{bind_group::BindGroupID, buffer_view::BufferView, render_pipeline::PipelineHashType},
    math::{Mat4, Vec3, Vec4, color::Color},
};

/// Data used to render a item in the scene.
#[derive(Clone)]
pub(crate) struct ItemRenderData {
    pub bind_group: Vec<BindGroupID>,
    pub render_pipeline: PipelineHashType,//RR<RenderPipeLine>,
    pub vertex_buffer: BufferView,
    pub index_buffer: Option<BufferView>,
    pub index_format: IndexFormat,
    pub index_start: u32,
    pub index_count: u32,
    pub base_vertex: u32,
}

impl ItemRenderData {
    pub fn new(
        bind_group: Vec<BindGroupID>,
        render_pipeline: PipelineHashType,//RR<RenderPipeLine>,
        vertex_buffer: BufferView,
        index_buffer: Option<BufferView>,
        index_format: IndexFormat,
        index_start: u32,
        index_count: u32,
        base_vertex: u32,
    ) -> Self {
        Self {
            bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_format,
            index_start,
            index_count,
            base_vertex,
        }
    }
}

/// Data used to render a frame for a give camera.
pub(crate) struct CameraRenderData {
    pub(crate) _camera_id: NodeId,
    pub priority: u32,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub depth_attachment: TextureHandle,
    pub color_attchment: TextureHandle,
    /// Physical view port.
    pub view_port: Vec4,
    pub clear_color: Option<Color>,
    pub opaque_item_data: Vec<ItemRenderData>,
    pub skybox_item_data: Option<ItemRenderData>,
    pub transparent_item_data: Vec<ItemRenderData>,
    pub camera_position: Vec3,
}

impl CameraRenderData {
    pub fn new(
        camera_id: NodeId,
        priority: u32,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        // TODO: use TextureHandle to avoid clone TextureView every frame.
        camera_depth_attachment: TextureHandle,
        camera_color_attchment: TextureHandle,
        view_port: Vec4,
        clear_color: Option<Color>,
        camera_position: Vec3,
    ) -> Self {
        Self {
            _camera_id: camera_id,
            priority,
            view_matrix,
            projection_matrix,
            opaque_item_data: vec![],
            skybox_item_data: None,
            transparent_item_data: vec![],
            depth_attachment: camera_depth_attachment,
            color_attchment: camera_color_attchment,
            view_port,
            clear_color,
            camera_position,
            // builtin_uniforms: PerCameraBuiltinUniforms::default(),
        }
    }
}

/// Data used to render a frame.
pub(crate) struct FrameRenderData {
    pub camera_data: Vec<CameraRenderData>,
    pub(crate) time_data: Vec4,
}

impl Default for FrameRenderData {
    fn default() -> Self {
        Self {
            camera_data: vec![],
            time_data: Vec4::ZERO,
        }
    }
}

impl FrameRenderData {
    pub fn reset(&mut self) {
        self.camera_data.clear();
    }
}
