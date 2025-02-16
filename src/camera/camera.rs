use wgpu::{TextureFormat, TextureView};

use crate::{
    math::{Mat4, Vec3, Vec4},
    prelude::{
        bind_group::BindGroupManager,
        bind_group_layout::BindGroupLayoutManager,
        buffer::{GPUBufferManager, SyncBuffer},
        render_item_manager::RenderItemManager,
        texture_manager::TextureManager,
        GraphicsContext, ImagicContext, RenderTexture, SceneObject, Texture, Transform,
        TransformManager, VertexOrIndexCount, INVALID_ID,
    },
    types::{Dirtyable, ID},
    window::WindowSize,
};

use super::{CameraControllerOptions, Layer, LayerMask};

pub enum CameraMode {
    Perspective,
    Orthogonal,
}

pub struct Camera {
    camera_mode: CameraMode,
    /// Field of view in radians
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
    target_pos: Vec3,
    up: Vec3,

    /// Normalized viewport. Each component is in [0.0, 1.0].
    view_port: Vec4,

    /// Logical viewport corresponding to window logical size.
    logical_view_port: Vec4,

    /// Physical viewport corresponding to window physical size.
    /// It is the real view port used by renderpass.
    physical_view_port: Vec4,

    clear_color: Vec4,

    render_texture: Option<Box<dyn RenderTexture>>,

    transform: ID,

    bind_group_id: ID,
    // bind_group_layout_id: ID,

    // TODO: merge buffers
    vertex_uniform_buffer_id: ID,
    fragment_uniform_buffer_id: ID,

    depth_texture: ID,

    layer: Layer,
    pub layer_mask: LayerMask,

    pub controller_options: Option<CameraControllerOptions>,
    pub(crate) controller_id: ID,

    /// User call render() manually, the Renderer will not take care of it.
    pub draw_manually: bool,
    pub is_viewport_auto_resizeable: bool,

    is_dirty: bool,
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
            logical_view_port: Vec4::new(0.0, 0.0, 100.0, 100.0),
            physical_view_port: Vec4::new(0.0, 0.0, 100.0, 100.0),
            clear_color: Vec4::new(0.1, 0.2, 0.3, 1.0),
            render_texture: None,
            transform: INVALID_ID,
            bind_group_id: INVALID_ID,
            // bind_group_layout_id: INVALID_ID,
            vertex_uniform_buffer_id: INVALID_ID,
            fragment_uniform_buffer_id: INVALID_ID,

            depth_texture: INVALID_ID,

            layer: Layer::Default,
            layer_mask: LayerMask::default(),
            controller_options: None,
            controller_id: INVALID_ID,

            draw_manually: false,
            is_viewport_auto_resizeable: true,

            is_dirty: false,
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

impl Dirtyable for Camera {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self) {
        self.is_dirty = true;
    }
}

impl Camera {
    /// Set render texture for this camera.
    pub fn set_render_texture(&mut self, render_texture: Box<dyn RenderTexture>) {
        self.render_texture = Some(render_texture);
    }

    pub fn get_render_texture(&mut self) -> &mut Option<Box<dyn RenderTexture>> {
        &mut self.render_texture
    }

    /// Render the scene by current Camera.
    pub fn render(&mut self, context: &ImagicContext, sync_buffer: Option<&SyncBuffer>) {
        // let mut attachment_views: &[TextureView];
        if let Some(rt) = &self.render_texture {
            // draw to rt
            self.aspect = self.physical_view_port.z / self.physical_view_port.w;

            let color_attachment_views = rt.get_color_attachment_views();
            let attachment_count = color_attachment_views.len();
            let color_attachment_format = rt.get_color_attachment_format();
            let depth_attachment_views = rt.get_depth_attachment_views();
            if attachment_count == 1 {
                self.render_to_attachments(
                    context,
                    &color_attachment_views[0],
                    0,
                    Some(color_attachment_format),
                    Some(&depth_attachment_views[0]),
                    sync_buffer,
                );
            } else {
                for (view_index, cur_rt_view) in color_attachment_views.iter().enumerate() {
                    // Update camera view matrix before rendering.
                    let (camera_pos, target_pos, up) = rt.get_per_face_camera_params(view_index);
                    context
                        .transform_manager()
                        .borrow_mut()
                        .get_transform_mut(self.transform)
                        .set_position(camera_pos);
                    self.target_pos = target_pos;
                    self.up = up;
                    // let depth_attachment = rt.get_depth_attachment_id();
                    // self.depth_texture = depth_attachment;
                    self.update_uniform_buffers(
                        context.graphics_context(),
                        &context.transform_manager().borrow(),
                        context.buffer_manager(),
                    );
                    self.render_to_attachments(
                        context,
                        cur_rt_view,
                        0,
                        Some(color_attachment_format),
                        Some(&depth_attachment_views[view_index]),
                        sync_buffer,
                    );
                }
            }
        } else {
            // draw to screen
            let surface_texture = context
                .graphics_context()
                .get_surface()
                .get_current_texture();
            let surface_texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.render_to_attachments(context, &surface_texture_view, 0, None, None, sync_buffer);
        }
    }

    /// Render the scene to the given color attachment view
    pub(crate) fn render_to_attachments(
        &self,
        context: &ImagicContext,
        color_attachment_view: &TextureView,
        camera_index: usize,
        color_attachment_format: Option<TextureFormat>,
        depth_attachment_view: Option<&TextureView>,
        sync_buffer: Option<&SyncBuffer>,
    ) {
        let mut encoder =
            context
                .graphics_context()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("imagic render command encoder desc"),
                });

        // Render scene
        {
            let camera_clear_color = self.get_clear_color();
            let camera_depth_textue = self.get_depth_texture();

            let dpeth_texture_view = if let Some(depth_view) = depth_attachment_view {
                depth_view
            } else {
                context
                    .texture_manager()
                    .get_texture_view(camera_depth_textue)
            };

            let camera_layer_mask = self.layer_mask;

            let mut load_op = wgpu::LoadOp::Load;
            if camera_index == 0 {
                let clear_color = wgpu::Color {
                    r: camera_clear_color.x as f64,
                    g: camera_clear_color.y as f64,
                    b: camera_clear_color.z as f64,
                    a: camera_clear_color.w as f64,
                };
                // let clear_color = wgpu::Color::BLUE;
                load_op = wgpu::LoadOp::Clear(clear_color);
            }

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("imagic render pass desc"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_attachment_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // load: wgpu::LoadOp::Clear(clear_color),
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: dpeth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let view_port = self.get_physical_viewport();
            rpass.set_viewport(view_port.x, view_port.y, view_port.z, view_port.w, 0.0, 1.0);

            let camera_bind_group_id = self.get_bind_group_id();

            let render_items = context.render_item_manager().render_items();
            for (item_id, item) in render_items.iter().enumerate() {
                if item.is_visible && camera_layer_mask.contains(item.layer) {
                    let pipeline_manager = context.pipeline_manager();
                    if pipeline_manager
                        .borrow()
                        .get_render_pipeline(item_id)
                        .is_none()
                    {
                        let material = context
                            .material_manager()
                            .get_material(item.get_material_id());
                        context.create_pipeline(item_id, color_attachment_format, material);
                    }

                    let material = context
                        .material_manager()
                        .get_material(item.get_material_id());
                    let material_bind_group_id = material.get_bind_group_id();
                    let lighting_bind_group_id = context.light_manager().get_bind_group_id();
                    let item_bind_groups = [
                        item.get_item_bind_group_id(), // Group 0
                        camera_bind_group_id,          // Group 1
                        material_bind_group_id, // Group 2, this contains material related unifroms
                        lighting_bind_group_id, // Group 3
                    ];

                    rpass.set_pipeline(
                        pipeline_manager
                            .borrow()
                            .get_render_pipeline(item_id)
                            .expect("item has no pipeline"),
                    );
                    for (index, bind_group_id) in item_bind_groups.iter().enumerate() {
                        if *bind_group_id != INVALID_ID {
                            let bind_group =
                                context.bind_group_manager().get_bind_group(*bind_group_id);
                            rpass.set_bind_group(index as u32, bind_group, &[]);
                        }
                    }

                    let vertex_or_index_count = item.get_vertex_or_index_count();
                    match vertex_or_index_count {
                        VertexOrIndexCount::VertexCount {
                            vertex_count,
                            instance_count,
                        } => {
                            // TODO: set vertex buffer?
                            rpass.draw(0..*vertex_count, 0..*instance_count);
                        }
                        VertexOrIndexCount::IndexCount {
                            index_count,
                            base_vertex,
                            instance_count,
                            index_format,
                        } => {
                            let vertex_buffer = context
                                .buffer_manager()
                                .get_buffer(item.get_vertex_buffer_id());
                            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                            let index_buffer = context
                                .buffer_manager()
                                .get_buffer(item.get_index_buffer_id());
                            rpass.set_index_buffer(index_buffer.slice(..), *index_format);
                            rpass.draw_indexed(0..*index_count, *base_vertex, 0..*instance_count);
                        }
                    }
                }
            }
        }

        if let Some(sync_buffer) = sync_buffer {
            sync_buffer.sync(&mut encoder);
        }
        context.graphics_context().submit(Some(encoder.finish()));
    }

    pub fn on_init(
        &mut self,
        logical_size: &WindowSize,
        physical_size: &WindowSize,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        transform_manager: &TransformManager,
        buffer_manager: &mut GPUBufferManager,
        texture_manager: &mut TextureManager,
    ) {
        let (logical_width, logical_height) = logical_size.get();
        self._compute_logical_viewport(logical_width, logical_height);
        let (physical_width, physical_height) = physical_size.get();
        self._compute_physical_viewport_aspect(physical_width, physical_height);
        self.create_depth_texture(
            graphics_context,
            texture_manager,
            physical_width as u32,
            physical_height as u32,
        );
        self.create_bind_group(
            graphics_context,
            bind_group_manager,
            bind_group_layout_manager,
            transform_manager,
            buffer_manager,
        );
    }

    pub fn on_update(
        &mut self,
        graphics_context: &GraphicsContext,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
    ) {
        if !self.is_dirty {
            return;
        }
        self.is_dirty = false;
        self.update_uniform_buffers(graphics_context, transform_manager, buffer_manager);
    }

    /// Called to update depth texture size, viewport and projection matrix when window resized.
    pub(crate) fn on_resize(
        &mut self,
        graphics_context: &GraphicsContext,
        texture_manager: &mut TextureManager,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
        physical_width: u32,
        physical_height: u32,
        logical_width: u32,
        logical_height: u32,
    ) {
        if !self.is_viewport_auto_resizeable {
            return;
        }
        // TODO: release outdated depth_texture: self.depth_texture, which requires to refactor TextureManager class.
        self.create_depth_texture(
            graphics_context,
            texture_manager,
            physical_width,
            physical_height,
        );
        self._compute_physical_viewport_aspect(physical_width as f32, physical_height as f32);
        self._compute_logical_viewport(logical_width as f32, logical_height as f32);
        self.update_uniform_buffers(graphics_context, transform_manager, buffer_manager);
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
        controller_options: Option<CameraControllerOptions>,
        imagic_context: &mut ImagicContext,
    ) -> ID {
        let transform_manager = imagic_context.transform_manager();

        let mut transform = Transform::default();
        transform.set_position(pos);

        let transform_index = transform_manager.borrow_mut().add_transform(transform);

        let camera = Self {
            fov,
            aspect,
            near,
            far,
            transform: transform_index,
            controller_options,
            ..Default::default()
        };

        let camera_index = imagic_context.add_camera(camera);

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

    /// Update camera matrices and other uniforms.
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

    /// Set normalized viewport, range in [0.0, 1.0].
    pub fn set_viewport(&mut self, view_port: Vec4) {
        self.view_port = view_port;
    }

    pub fn get_viewport(&self) -> &Vec4 {
        &self.view_port
    }

    /// Set logical viewport corresponding to window logical size.
    pub fn set_logical_viewport(&mut self, logical_view_port: Vec4) {
        self.logical_view_port = logical_view_port;
    }

    /// Logical viewport corresponding to window logical size.
    pub fn get_logical_viewport(&self) -> &Vec4 {
        &self.logical_view_port
    }

    pub fn set_physical_viewport(&mut self, physical_view_port: Vec4) {
        self.physical_view_port = physical_view_port;
    }

    /// Get the real view port (corresponding to window physical size) used by render pass.
    pub fn get_physical_viewport(&self) -> &Vec4 {
        &self.physical_view_port
    }

    /// Compute the logical viewport.
    fn _compute_logical_viewport(&mut self, logical_width: f32, logical_height: f32) {
        self.logical_view_port.x = self.view_port.x * logical_width;
        self.logical_view_port.y = self.view_port.y * logical_height;
        self.logical_view_port.z = self.view_port.z * logical_width;
        self.logical_view_port.w = self.view_port.w * logical_height;
    }

    /// Compute the real (physical) view port used by render pass, and compute aspect used to calculate projection matrix.
    fn _compute_physical_viewport_aspect(&mut self, physical_width: f32, physical_height: f32) {
        self.physical_view_port.x = self.view_port.x * physical_width;
        self.physical_view_port.y = self.view_port.y * physical_height;
        self.physical_view_port.z = self.view_port.z * physical_width;
        self.physical_view_port.w = self.view_port.w * physical_height;
        // info!("physical_view_port: {}", self.physical_view_port);

        self.aspect = self.physical_view_port.z / self.physical_view_port.w;
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
        width: u32,
        height: u32,
    ) {
        const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
        let dpeth_texture =
            Texture::create_depth_texture(graphics_context, width, height, DEPTH_FORMAT, true);
        self.depth_texture = texture_manager.add_texture(dpeth_texture);
    }

    pub fn get_depth_texture(&self) -> ID {
        self.depth_texture
    }
}
