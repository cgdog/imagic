//! Show IBL

use changeable::Changeable;
use common::create_camera;
use imagic::prelude::*;
use imagic::window::WindowSize;
use std::f32::consts::FRAC_PI_4;

mod common;

pub struct App {
    skybox: Skybox,
    sphere: Sphere,
    camera_id: ID,
    window_size: WindowSize,
    camera_z: f32,
    camera_controller_option: Changeable<CameraControllerOptions>,
    sphere_use_textured_pbr: Changeable<bool>,
    red_pbr_material_index: ID,
    textured_pbr_material_index: ID,
}

impl Default for App {
    fn default() -> Self {
        Self {
            skybox: Skybox::default(),
            sphere: Sphere::new(1.0, 256, 256),
            camera_id: INVALID_ID,
            window_size: WindowSize::new(800.0, 500.0),
            camera_z: 8.0,
            camera_controller_option: Changeable::new(CameraControllerOptions::default()),
            sphere_use_textured_pbr: Changeable::new(false),
            red_pbr_material_index: INVALID_ID,
            textured_pbr_material_index: INVALID_ID,
        }
    }
}

impl App {
    fn prepare_lights(&mut self, imagic_context: &mut ImagicContext) {
        let transform_manager = imagic_context.transform_manager();
        let point_light_0 = PointLight::new(
            Vec3::new(-10.0, 10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );
        let point_light_1 = PointLight::new(
            Vec3::new(10.0, 10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );
        let point_light_2 = PointLight::new(
            Vec3::new(-10.0, -10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );
        let point_light_3 = PointLight::new(
            Vec3::new(10.0, -10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            &mut transform_manager.borrow_mut(),
        );

        let light_manager = imagic_context.light_manager_mut();
        light_manager.add_point_light(point_light_0);
        light_manager.add_point_light(point_light_1);
        light_manager.add_point_light(point_light_2);
        light_manager.add_point_light(point_light_3);
    }

    #[allow(unused)]
    fn prepare_rusted_pbr_material(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let graphics_context = imagic_context.graphics_context();
        let mut pbr_material = Box::new(PBRMaterial::new(
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            1.0,
            1.0,
            1.0,
        ));
        let albedo_texture = Texture::create_from_bytes(
            graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_basecolor.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let normal_texture = Texture::create_from_bytes(
            graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_normal.png"),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let metallic_texture = Texture::create_from_bytes(
            graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_metallic.png"),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let roughness_texture = Texture::create_from_bytes(
            graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_roughness.png"),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let ao_texture = Texture::create_from_bytes(
            graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/ao.png"),
            wgpu::TextureFormat::Rgba8Unorm,
        );

        let texture_manager = imagic_context.texture_manager_mut();

        let albedo_texture = texture_manager.add_texture(albedo_texture);
        pbr_material.set_albedo_texture(albedo_texture);
        let normal_texture = texture_manager.add_texture(normal_texture);
        pbr_material.set_normal_texture(normal_texture);
        let metallic_texture = texture_manager.add_texture(metallic_texture);
        pbr_material.set_metallic_texture(metallic_texture);
        let roughness_texture = texture_manager.add_texture(roughness_texture);
        pbr_material.set_roughness_texture(roughness_texture);
        let ao_texture = texture_manager.add_texture(ao_texture);
        pbr_material.set_ao_texture(ao_texture);

        let pbr_material_index = imagic_context.add_material(pbr_material);
        pbr_material_index
    }

    fn prepare_red_pbr_material(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let pbr_material = Box::new(PBRMaterial::new(
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            1.0,
            0.2,
            1.0,
        ));
        imagic_context.add_material(pbr_material)
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic_context: &mut ImagicContext) {
        let fov = FRAC_PI_4;
        let aspect = self.window_size.get_half_width() / self.window_size.get_height();
        let near = 0.01;
        let far = 500.0;
        // first camera
        let viewport = Vec4::new(0.0, 0.0, 1.0, 1.0);
        let clear_color = Color::new(0.1, 0.1, 0.1, 1.0);
        let camera_pos = Vec3::new(0.0, 0.0, self.camera_z);
        // TODO: 让 Cube 以天空盒的形式渲染
        let camera_layer_mask = LayerMask::new(Layer::Default.into());
        self.camera_id = create_camera(
            imagic_context,
            camera_pos,
            viewport,
            clear_color,
            fov,
            aspect,
            near,
            far,
            camera_layer_mask,
            Some(*self.camera_controller_option),
        );
        self.prepare_lights(imagic_context);

        self.textured_pbr_material_index = self.prepare_rusted_pbr_material(imagic_context);
        self.red_pbr_material_index = self.prepare_red_pbr_material(imagic_context);

        let pbr_material_index = if *self.sphere_use_textured_pbr {
            self.textured_pbr_material_index
        } else {
            self.red_pbr_material_index
        };

        self.sphere.init(imagic_context, pbr_material_index);
        let cube_texture_id = EquirectToCubeConverter::default().convert_by_bytes(
            include_bytes!("./assets/pbr/hdr/newport_loft.hdr"),
            imagic_context,
            512,
            wgpu::TextureFormat::Rgba32Float,
        );
        self.skybox
            .init_with_cube_texture(imagic_context, cube_texture_id);

        // self.skybox.init_ldr_bytes(imagic_context, [
        //     include_bytes!("./assets/skybox/right.jpg"),
        //     include_bytes!("./assets/skybox/left.jpg"),
        //     include_bytes!("./assets/skybox/top.jpg"),
        //     include_bytes!("./assets/skybox/bottom.jpg"),
        //     include_bytes!("./assets/skybox/front.jpg"),
        //     include_bytes!("./assets/skybox/back.jpg"),
        // ],);
    }

    fn on_update(&mut self, imagic_context: &mut ImagicContext) {
        if self.camera_controller_option.is_changed() {
            self.camera_controller_option.reset();
            imagic_context.change_camera_controller(self.camera_id, &self.camera_controller_option);
        }

        if self.sphere_use_textured_pbr.is_changed() {
            self.sphere_use_textured_pbr.reset();
            let pbr_material_index = if *self.sphere_use_textured_pbr {
                self.textured_pbr_material_index
            } else {
                self.red_pbr_material_index
            };

            imagic_context.pipeline_manager().borrow_mut().remove_render_pipeline(self.sphere.render_item_id());

            imagic_context
                .render_item_manager_mut()
                .get_render_item_mut(self.sphere.render_item_id())
                .set_material_id(pbr_material_index);
        }
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - IBL")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .default_size([100.0, 5.0])
            .show(&ctx, |ui| {
                let rotate_button_text = if self.camera_controller_option.is_auto_rotate {
                    "Stop Auto Rotate"
                } else {
                    "Auto Rotate"
                };
                if ui.button(rotate_button_text).clicked() {
                    self.camera_controller_option.is_auto_rotate =
                        !self.camera_controller_option.is_auto_rotate;
                }

                let sphere_material_text = if *self.sphere_use_textured_pbr {
                    "Use red pbr"
                } else {
                    "Use textured pbr"
                };
                if ui.button(sphere_material_text).clicked() {
                    *self.sphere_use_textured_pbr = !*self.sphere_use_textured_pbr;
                }
            });
    }

    fn get_imagic_option(&self) -> ImagicOption {
        ImagicOption::new(self.window_size, "IBL Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut imagic = Imagic::new(Box::new(App::default()));
    imagic.run();
}
