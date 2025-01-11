use log::info;

use crate::{
    math::{Mat4, Vec3, Vec4}, prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager,
        buffer::GPUBufferManager, render_item_manager::RenderItemManager,
        texture_manager::TextureManager, GraphicsContext, ImagicContext, SceneObject, Texture,
        Transform, TransformManager, INVALID_ID,
    }, types::ID, window::Window
};

use super::{Layer, LayerMask};

pub enum CameraMode {
    Perspective,
    Orthogonal,
}

pub struct Camera {
    camera_mode: CameraMode,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
    target_pos: Vec3,
    up: Vec3,

    view_port: Vec4,
    physical_view_port: Vec4,

    clear_color: Vec4,

    transform: ID,

    bind_group_id: ID,
    // bind_group_layout_id: ID,

    // TODO: merge buffers
    vertex_uniform_buffer_id: ID,
    fragment_uniform_buffer_id: ID,

    depth_texture: ID,

    layer: Layer,
    pub layer_mask: LayerMask,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            camera_mode: CameraMode::Perspective,
            fov: std::f32::consts::FRAC_PI_4,
            aspect: 1.0,
            near: 0.1,
            far: 500.0,
            target_pos: Vec3::ZERO,
            up: Vec3::Y,
            view_port: Vec4::new(0.0, 0.0, 1.0, 1.0),
            physical_view_port: Vec4::new(0.0, 0.0, 100.0, 100.0),
            clear_color: Vec4::new(0.1, 0.2, 0.3, 1.0),
            transform: INVALID_ID,
            bind_group_id: INVALID_ID,
            // bind_group_layout_id: INVALID_ID,
            vertex_uniform_buffer_id: INVALID_ID,
            fragment_uniform_buffer_id: INVALID_ID,

            depth_texture: INVALID_ID,

            layer: Layer::Default,
            layer_mask: LayerMask::default(),
        }
    }
}

impl SceneObject for Camera {
    fn transform(&self) -> &ID {
        &self.transform
    }

    fn get_layer(&self) -> Layer {
        self.layer
    }

    fn set_layer(&mut self, layer: Layer, _render_item_manager: &mut RenderItemManager) {
        self.layer = layer;
        // render_item_manager.get_render_item_mut(self.render_item_id).layer = layer;
    }
}

impl Camera {
    pub fn init_after_app(
        &mut self,
        window: &Window,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        transform_manager: &TransformManager,
        buffer_manager: &mut GPUBufferManager,
        texture_manager: &mut TextureManager,
    ) {
        self._compute_physical_viewport_from_window(window);
        self.create_depth_texture(graphics_context, texture_manager, window);
        self.create_bind_group(
            graphics_context,
            bind_group_manager,
            bind_group_layout_manager,
            transform_manager,
            buffer_manager,
        );
    }

    pub fn get_bind_group_id(&self) -> ID {
        self.bind_group_id
    }

    fn create_bind_group(
        &mut self,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        transform_manager: &TransformManager,
        buffer_manager: &mut GPUBufferManager,
    ) -> ID {
        let projection_matrix = self.get_projection_matrix();
        let view_matrix = self.get_view_matrix(transform_manager);
        let mut mx_ref: [f32; 16 * 2] = [0.0; 16 * 2];
        mx_ref[..16].copy_from_slice(view_matrix.as_ref());
        mx_ref[16..32].copy_from_slice(projection_matrix.as_ref());

        let camera_vertex_uniform_buf =
            graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Vertex Uniform Buffer"),
                contents: bytemuck::cast_slice(&mx_ref),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_pos = transform_manager
            .get_transform(self.transform)
            .get_position();
        let camera_fragment_uniforms = [camera_pos[0], camera_pos[1], camera_pos[2], 1.0];

        let fragment_uniform_buffer =
            graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Fragment Uniform Buffer"),
                contents: bytemuck::cast_slice(&[camera_fragment_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = bind_group_layout_manager.get_camera_bind_group_layout();
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            label: Some("Camera bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_vertex_uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fragment_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        self.vertex_uniform_buffer_id = buffer_manager.add_buffer(camera_vertex_uniform_buf);
        self.fragment_uniform_buffer_id = buffer_manager.add_buffer(fragment_uniform_buffer);
        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        bind_group_id
    }

    pub fn new(
        pos: Vec3,
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
        imagic_context: &mut ImagicContext,
    ) -> ID {
        let transform_manager = imagic_context.transform_manager_mut();

        let mut transform = Transform::default();
        transform.set_position(pos);

        let transform_index = transform_manager.add_transform(transform);

        let camera = Self {
            fov,
            aspect,
            near,
            far,
            transform: transform_index,
            ..Default::default()
        };

        let camera_manager = imagic_context.camera_manager_mut();
        let camera_index = camera_manager.add_camera(camera);

        camera_index
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let projection = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        projection
    }

    pub fn get_view_matrix(&self, transform_manager: &TransformManager) -> Mat4 {
        let pos = transform_manager
            .get_transform(self.transform)
            .get_position();
        let view = Mat4::look_at_rh(*pos, self.target_pos, self.up);
        view
    }

    pub fn update_uniform_buffers(
        &self,
        graphics_context: &GraphicsContext,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
    ) {
        let projection_matrix = self.get_projection_matrix();
        let view_matrix = self.get_view_matrix(transform_manager);
        let mut mx_ref: [f32; 16 * 2] = [0.0; 16 * 2];
        mx_ref[..16].copy_from_slice(view_matrix.as_ref());
        mx_ref[16..32].copy_from_slice(projection_matrix.as_ref());

        let camera_vertex_uniform_buf = buffer_manager.get_buffer(self.vertex_uniform_buffer_id);
        graphics_context.get_queue().write_buffer(
            camera_vertex_uniform_buf,
            0,
            bytemuck::cast_slice(&mx_ref),
        );

        let camera_pos = transform_manager
            .get_transform(self.transform)
            .get_position();
        let camera_fragment_uniforms = [camera_pos[0], camera_pos[1], camera_pos[2], 1.0];

        let fragment_uniform_buffer = buffer_manager.get_buffer(self.fragment_uniform_buffer_id);
        graphics_context.get_queue().write_buffer(
            fragment_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_fragment_uniforms]),
        );
    }

    pub fn get_camera_mode(&self) -> &CameraMode {
        &self.camera_mode
    }

    pub fn set_camera_mode(&mut self, camera_mode: CameraMode) {
        self.camera_mode = camera_mode;
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, new_fov: f32) {
        self.fov = new_fov;
    }

    pub fn get_aspect(&self) -> f32 {
        self.aspect
    }

    pub fn set_aspect(&mut self, new_aspect: f32) {
        self.aspect = new_aspect;
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn set_near(&mut self, new_near: f32) {
        self.near = new_near;
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn set_far(&mut self, new_far: f32) {
        self.far = new_far;
    }

    pub fn set_viewport(&mut self, view_port: Vec4) {
        self.view_port = view_port;
    }

    pub fn get_viewport(&self) -> &Vec4 {
        &self.view_port
    }

    pub fn set_physical_viewport(&mut self, physical_view_port: Vec4) {
        self.physical_view_port = physical_view_port;
    }

    /// get the real view port used by render pass
    pub fn get_physical_viewport(&self) -> &Vec4 {
        &self.physical_view_port
    }

    /// compute the real view port used by render pass
    fn _compute_physical_viewport_from_window(&mut self, window: &Window) {
        let (physical_widht, physical_heigt) = window.get_physical_size().get();
        self.physical_view_port.x = self.view_port.x * physical_widht;
        self.physical_view_port.y = self.view_port.y * physical_heigt;
        self.physical_view_port.z = self.view_port.z * physical_widht;
        self.physical_view_port.w = self.view_port.w * physical_heigt;

        info!("physical_view_port: {}", self.physical_view_port);
    }

    pub fn get_clear_color(&self) -> &Vec4 {
        &self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.clear_color = clear_color;
    }

    fn create_depth_texture(
        &mut self,
        graphics_context: &GraphicsContext,
        texture_manager: &mut TextureManager,
        window: &Window,
    ) {
        const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
        let window_physical_size = window.get_physical_size();
        // let width = self.physical_view_port.z as u32;
        // let height = self.physical_view_port.w as u32;
        let width = window_physical_size.get_width() as u32;
        let height = window_physical_size.get_height() as u32;

        let dpeth_texture =
            Texture::create_depth_texture(graphics_context, width, height, DEPTH_FORMAT);
        self.depth_texture = texture_manager.add_texture(dpeth_texture);
    }

    pub fn get_depth_texture(&self) -> ID {
        self.depth_texture
    }
}
