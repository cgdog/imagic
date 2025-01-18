use std::f32::consts::FRAC_PI_4;

use imagic::{
    camera::{Camera, CameraControllerOptions, LayerMask},
    math::{Color, Vec3, Vec4},
    prelude::ImagicContext,
    types::ID,
};

pub fn create_camera(
    imagic_context: &mut ImagicContext,
    camera_pos: Vec3,
    viewport: Vec4,
    clear_color: Vec4,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
    layer_mask: LayerMask,
    controller_options: Option<CameraControllerOptions>,
) -> ID {
    let camera_id = Camera::new(
        camera_pos,
        fov,
        aspect,
        near,
        far,
        controller_options,
        imagic_context,
    );

    let camera = imagic_context.camera_manager_mut().get_camera(camera_id);
    camera.borrow_mut().set_viewport(viewport);
    camera.borrow_mut().set_clear_color(clear_color);
    camera.borrow_mut().layer_mask = layer_mask;
    camera_id
}

#[allow(unused)]
pub fn create_thin_camera(imagic_context: &mut ImagicContext, camera_pos: Vec3, aspect: f32) -> ID {
    create_camera(
        imagic_context,
        camera_pos,
        Vec4::new(0.0, 0.0, 1.0, 1.0),
        Color::new(0.0, 0.0, 0.0, 0.0),
        FRAC_PI_4,
        aspect,
        0.01,
        500.0,
        LayerMask::default(),
        Some(CameraControllerOptions::new(Vec3::ZERO, false)),
    )
}
