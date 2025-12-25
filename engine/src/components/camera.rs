use std::cell::RefCell;

use crate::{
    assets::{TextureDimension, TextureFormat, TextureHandle, TextureSamplerManager},
    core::layer::LayerMask,
    graphics::uniform::BuiltinUniforms,
    impl_component,
    math::{Mat4, Vec3, Vec4, color::Color},
    window::window_size::WindowSize,
};

/// The camera projection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    /// Perspective projection.
    Perspective,
    /// Orthogonal projection.
    Orthogonal,
}

/// The camera component used to render the scene.
pub struct Camera {
    /// The projection mode.
    pub mode: CameraMode,
    /// Field of view in radians.
    pub fov: f32,
    /// Aspect ratio.
    pub aspect: f32,
    /// Near plane of frustum.
    pub near: f32,
    /// Far plane of frustum.
    pub far: f32,
    /// The target postion that the camera looks at.
    pub target_pos: Vec3,
    /// The up vector.
    pub up: Vec3,
    /// The clear color which is used to clear the color attachment.
    pub clear_color: Option<Color>,
    /// The layers visible to the camera.
    pub visible_layers: LayerMask,
    /// The render priority.
    /// The camera with lower priority will be rendered first.
    pub priority: u32,

    /// The size of orthogonal frustum.
    pub orthogonal_frustum_size: f32,

    /// The left of orthogonal frustum.
    pub left: f32,
    /// The right of orthogonal frustum.
    pub right: f32,
    /// The top of orthogonal frustum.
    pub top: f32,
    /// The bottom of orthogonal frustum.
    pub bottom: f32,

    /// The color attachment texture handle that the camera renders to.
    /// If it is `TextureHandle::INVALID`(default), the camera will render to screen.
    pub color_attachment: TextureHandle,
    /// The depth attachment texture handle. If `color_attachment` is `TextureHandle::INVALID`, `depth_attachment` will be set automatically.
    pub depth_attachment: TextureHandle,
    /// The depth attachment texture format.
    pub depth_format: TextureFormat,

    /// Normalized viewport. Each component is in [0.0, 1.0].
    pub(crate) view_port: Vec4,

    /// Logical viewport corresponding to window logical size.
    pub(crate) logical_view_port: Vec4,

    /// Physical viewport corresponding to window physical size.
    /// It is the real view port used by renderpass.
    pub(crate) physical_view_port: Vec4,

    pub(crate) per_camera_uniforms: RefCell<BuiltinUniforms>,

    physical_size: WindowSize,
    logical_size: WindowSize,
}

impl_component!(Camera);

impl Default for Camera {
    fn default() -> Self {
        Self {
            mode: CameraMode::Perspective,
            fov: std::f32::consts::FRAC_PI_4,
            aspect: 1.0,
            near: 0.1,
            far: 500.0,
            target_pos: Vec3::ZERO,
            up: Vec3::Y,
            physical_size: WindowSize::new(100.0, 100.0),
            logical_size: WindowSize::new(100.0, 100.0),
            view_port: Vec4::new(0.0, 0.0, 1.0, 1.0),
            logical_view_port: Vec4::new(0.0, 0.0, 100.0, 100.0),
            physical_view_port: Vec4::new(0.0, 0.0, 100.0, 100.0),
            clear_color: Some(Color::new(0.1, 0.2, 0.3, 1.0)),
            visible_layers: LayerMask::default(),
            priority: 0,
            color_attachment: TextureHandle::INVALID,
            depth_attachment: TextureHandle::INVALID,
            depth_format: TextureFormat::Depth24PlusStencil8,
            per_camera_uniforms: RefCell::new(BuiltinUniforms::new("Camera".to_owned())),
            orthogonal_frustum_size: 2.0,
            left: -1.0,
            right: 1.0,
            top: 1.0,
            bottom: -1.0,
        }
    }
}

impl Camera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let camera = Self {
            fov,
            aspect,
            near,
            far,
            ..Default::default()
        };
        camera
    }

    pub fn new_orthogonal(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let camera = Self {
            mode: CameraMode::Orthogonal,
            left,
            right,
            bottom,
            top,
            near,
            far,
            orthogonal_frustum_size: top,
            ..Default::default()
        };
        camera
    }

    pub fn new_orthogonal_by_size(orthogonal_frustum_size: f32, aspect: f32, near: f32, far: f32) -> Self {
        let camera = Self {
            mode: CameraMode::Orthogonal,
            orthogonal_frustum_size,
            left: -orthogonal_frustum_size * aspect,
            right: orthogonal_frustum_size * aspect,
            bottom: -orthogonal_frustum_size,
            top: orthogonal_frustum_size,
            near,
            far,
            ..Default::default()
        };
        camera
    }

    /// Set normalized viewport
    pub fn set_viewport(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.view_port.x = x;
        self.view_port.y = y;
        self.view_port.z = z;
        self.view_port.w = w;
        self.update_viewport();
    }

    pub fn set_viewport_by_vector(&mut self, view_port: Vec4) {
        self.view_port = view_port;
        self.update_viewport();
    }

    pub fn get_viewport(&self) -> &Vec4 {
        &self.view_port
    }

    pub fn get_viewport_mut(&mut self) -> &mut Vec4 {
        &mut self.view_port
    }

    pub(crate) fn on_resize(
        &mut self,
        texture_sampler_manager: &mut TextureSamplerManager,
        physical_size: &WindowSize,
        logical_size: &WindowSize,
    ) {
        if self.color_attachment != TextureHandle::INVALID {
            // This is a off screen camera. Do not need to resize attachments according to window size.
            return;
        }

        if *physical_size == self.physical_size && *logical_size == self.logical_size {
            // log::error!("useless resize: ({}, {})", physical_size.get_width(), physical_size.get_height());
            return;
        }
        self.physical_size = *physical_size;
        self.logical_size = *logical_size;
        self.update_viewport();

        let depth_texture_handle = texture_sampler_manager.create_attachment(
            physical_size.width as u32,
            physical_size.height as u32,
            1,
            TextureDimension::D2,
            1,
            self.depth_format,
        );
        // log::warn!("camera depth_texture_handle: {}", depth_texture_handle);
        texture_sampler_manager.remove_texture(&self.depth_attachment);
        self.depth_attachment = depth_texture_handle;
    }

    pub fn set_depth_format(&mut self, depth_format: TextureFormat) {
        self.depth_format = depth_format;
    }

    pub fn set_depth_attachment(&mut self, depth_attachment: TextureHandle) {
        self.depth_attachment = depth_attachment;
    }

    pub fn get_depth_attachment(&self) -> TextureHandle {
        self.depth_attachment
    }

    pub fn set_color_attachment(&mut self, color_attachment: TextureHandle) {
        self.color_attachment = color_attachment;
    }

    pub fn get_color_attachment(&self) -> TextureHandle {
        self.color_attachment
    }

    fn update_viewport(&mut self) {
        self.physical_view_port.x = self.view_port.x * self.physical_size.width;
        self.physical_view_port.y = self.view_port.y * self.physical_size.height;
        self.physical_view_port.z = self.view_port.z * self.physical_size.width;
        self.physical_view_port.w = self.view_port.w * self.physical_size.height;

        self.aspect = self.physical_view_port.z / self.physical_view_port.w;

        self.logical_view_port.x = self.view_port.x * self.logical_size.width;
        self.logical_view_port.y = self.view_port.y * self.logical_size.height;
        self.logical_view_port.z = self.view_port.z * self.logical_size.width;

        if self.mode == CameraMode::Orthogonal {
            self.left = -self.orthogonal_frustum_size * self.aspect;
            self.right = self.orthogonal_frustum_size * self.aspect;
        }
    }

    pub fn get_view_matrix(&self, camera_pos: &Vec3) -> Mat4 {
        let view = Mat4::look_at_rh(*camera_pos, self.target_pos, self.up);
        view
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let projection = if self.mode == CameraMode::Perspective {
            Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
        } else {
            let m = Mat4::orthographic_rh(self.left, self.right, self.bottom, self.top, self.near, self.far);
            // dbg!(m);
            m
        };
        projection
    }
}
