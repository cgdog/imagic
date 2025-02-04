//! Show Cubemap mipmaps

use changeable::Changeable;
use common::create_camera;
use common::materials::custom_skybox_material::CustomSkyboxMaterial;
use ibl::ibl_baker::IBLBaker;
use imagic::prelude::*;
use imagic::window::WindowSize;
use std::f32::consts::FRAC_PI_4;

mod common;

pub struct App {
    ibl_data: IBLData,
    skybox: Skybox,
    camera_id: ID,
    window_size: WindowSize,
    camera_z: f32,
    camera_controller_option: Changeable<CameraControllerOptions>,
    lod: Changeable<f32>,
    custom_skybox_material_id: ID,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ibl_data: IBLData::default(),
            skybox: Skybox::default(),
            camera_id: INVALID_ID,
            window_size: WindowSize::new(800.0, 500.0),
            camera_z: 8.0,
            camera_controller_option: Changeable::new(CameraControllerOptions::default()),
            lod: Changeable::new(0.0),
            custom_skybox_material_id: INVALID_ID,
        }
    }
}

impl App {
    fn init_ibl(&mut self, imagic_context: &mut ImagicContext) {
        let mut ibl_baker = IBLBaker::new(IBLBakerOptions {
            input_background_type: InputBackgroundType::HDRBytes(include_bytes!(
                "./assets/pbr/hdr/newport_loft.hdr"
            )),
            background_cube_map_size: 512,
            irradiance_cube_map_size: 32,
            ..Default::default()
        });
        self.ibl_data = ibl_baker.bake(imagic_context);
        let mut cube_mipmaps_generator = CubeMipmapsGenerator::new(
            self.ibl_data.background_cube_texture,
            512,
            wgpu::TextureFormat::Rgba32Float,
            MipmapGeneratorType::GaussianFilter4x4,
            // MipmapGeneratorType::BilinearFilter
        );
        cube_mipmaps_generator.execute(imagic_context);
        let mut custom_skybox_material = CustomSkyboxMaterial::new();
        custom_skybox_material.set_skybox_map(cube_mipmaps_generator.get_cube_with_mipmap());
        // custom_skybox_material.set_skybox_map(self.ibl_data.background_cube_texture);
        custom_skybox_material.set_cull_mode(wgpu::Face::Front);
        let custom_skybox_material_id =
            imagic_context.add_material(Box::new(custom_skybox_material));
        self.skybox
            .init_with_custom_material(imagic_context, custom_skybox_material_id);
        self.custom_skybox_material_id = custom_skybox_material_id;
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic_context: &mut ImagicContext) {
        self.init_ibl(imagic_context);

        let fov = FRAC_PI_4;
        let aspect = self.window_size.get_half_width() / self.window_size.get_height();
        let near = 0.01;
        let far = 500.0;
        // first camera
        let viewport = Vec4::new(0.0, 0.0, 1.0, 1.0);
        let clear_color = Color::new(0.1, 0.1, 0.1, 1.0);
        let camera_pos = Vec3::new(0.0, 0.0, self.camera_z);
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
    }

    fn on_update(&mut self, imagic_context: &mut ImagicContext) {
        if self.camera_controller_option.is_changed() {
            self.camera_controller_option.reset();
            imagic_context.change_camera_controller(self.camera_id, &self.camera_controller_option);
        }

        if self.lod.is_changed() {
            self.lod.reset();
            // info!("lod changed! to: {}", *self.lod);
            let custom_skybox_material = imagic_context.material_manager_mut().get_material_mut(self.custom_skybox_material_id);
            if let Some(custom_skybox_material) = custom_skybox_material.as_any_mut().downcast_mut::<CustomSkyboxMaterial>() {
                custom_skybox_material.set_lod(*self.lod);
                imagic_context.update_material(self.custom_skybox_material_id);
            }
        }
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - Mipmaps")
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
                    self.camera_controller_option.set();
                }

                if ui.add(egui::Slider::new(&mut *self.lod, 0.0..=10.0).text("lod")).changed() {
                    // info!("lod changed to: {}", *self.lod);
                    self.lod.set();
                }
            });
    }

    fn get_imagic_option(&self) -> ImagicOption {
        ImagicOption::new(self.window_size, "Mipmaps Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut imagic = Imagic::new(Box::new(App::default()));
    imagic.run();
}
