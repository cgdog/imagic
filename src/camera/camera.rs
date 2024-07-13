use crate::prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, GraphicsContext, ImagicContext, SceneObject, Transform, TransformManager};

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
    target_pos: glam::Vec3,
    up: glam::Vec3,

    transform: usize,

    bind_group_id: usize,
    // bind_group_layout_id: usize,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            camera_mode: CameraMode::Perspective,
            fov: std::f32::consts::FRAC_PI_4,
            aspect: 1.0,
            near: 0.1,
            far: 500.0,
            target_pos: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            transform: usize::MAX,
            bind_group_id: usize::MAX,
            // bind_group_layout_id: usize::MAX,
        }
    }
}

impl SceneObject for Camera {
    fn transform(&self) -> &usize {
        &self.transform
    }
}

impl Camera {

    pub fn init_after_app(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager
        , bind_group_layout_manager: &mut BindGroupLayoutManager, transform_manager: &TransformManager) {
        self.create_bind_group(graphics_context, bind_group_manager, bind_group_layout_manager, transform_manager);
    }

    pub fn get_bind_group_id(&self) -> usize {
        self.bind_group_id
    }

    fn create_bind_group(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager
        , bind_group_layout_manager: &mut BindGroupLayoutManager, transform_manager: &TransformManager) -> usize {
        let projection_matrix = self.get_projection_matrix();
        let view_matrix = self.get_view_matrix(transform_manager);
        let mut mx_ref: [f32; 16 * 2] = [0.0; 16 * 2];
        mx_ref[..16].copy_from_slice(view_matrix.as_ref());
        mx_ref[16..32].copy_from_slice(projection_matrix.as_ref());

        let camera_vertex_uniform_buf = graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Vertex Uniform Buffer"),
            contents: bytemuck::cast_slice(&mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_pos = transform_manager.get_transform(self.transform).get_position();
        let camera_fragment_uniforms = [camera_pos[0], camera_pos[1], camera_pos[2], 1.0];

        let fragment_uniform_buffer = graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Fragment Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_fragment_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = bind_group_layout_manager.get_camera_bind_group_layout();
        let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor{
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
            ]
        });
        let bind_group_id = bind_group_manager.add_bind_group(bind_group);
        self.bind_group_id = bind_group_id;
        bind_group_id
    }

    pub fn new(pos: glam::Vec3, fov: f32, aspect: f32, near: f32, far: f32,
        imagic_context: &mut ImagicContext) -> usize {
        let transform_manager = imagic_context.transform_manager_mut();

        let mut transform = Transform::default();
        transform.set_position(pos);

        let transform_index = transform_manager.add_transform(transform);

        let camera =Self {
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

    pub fn get_projection_matrix(&self) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        projection
    }

    pub fn get_view_matrix(&self, transform_manager: &TransformManager) -> glam::Mat4 {
        let pos = transform_manager.get_transform(self.transform).get_position();
        let view = glam::Mat4::look_at_rh(
            *pos,
            self.target_pos,
            self.up,
        );
        view
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
}