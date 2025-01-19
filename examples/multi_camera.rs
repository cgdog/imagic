use std::usize;
use std::f32::consts;

use common::create_camera;
use imagic::prelude::*;
use imagic::window::WindowSize;
use log::info;

mod common;

pub struct App {
    cube: Cube,
    first_camera_id: usize,
    second_camera_id: usize,
    window_size: WindowSize,
    camera_z: f32,
    camera_controller_option_1: CameraControllerOptions,
    camera_controller_option_2: CameraControllerOptions,
    need_update_camera_controller: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cube: Cube::new(1.0, 1.0, 1.0, 1, 1, 1),
            first_camera_id: INVALID_ID,
            second_camera_id: INVALID_ID,
            window_size: WindowSize::new(800.0, 500.0),
            camera_z: 8.0,
            camera_controller_option_1: CameraControllerOptions::new(Vec3::ZERO, true),
            camera_controller_option_2: CameraControllerOptions::default(),
            need_update_camera_controller: false,
        }
    }
}

impl App {
    fn prepare_hdr_texture(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut hdr_loader = HDRLoader {};
        let cwd = std::env::current_dir().unwrap();
        let hdr_path = cwd.join("examples/assets/pbr/hdr/newport_loft.hdr");
        let hdr_texture = hdr_loader.load(
            hdr_path.to_str().unwrap(),
            imagic_context.graphics_context(),
        );
        let hdr_texture_index = imagic_context
            .texture_manager_mut()
            .add_texture(hdr_texture);
        hdr_texture_index
    }

    fn _prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let albedo_texture = Texture::create_from_bytes(
            imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let albedo_texture_index = imagic_context
            .texture_manager_mut()
            .add_texture(albedo_texture);
        albedo_texture_index
    }

    fn prepare_material(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut equirectangular_to_cube_material = Box::new(EquirectangularToCubeMaterial::new());
        let hdr_texture = self.prepare_hdr_texture(imagic_context);
        equirectangular_to_cube_material.set_equirectangular_map(hdr_texture);
        // let albedo_index = self._prepare_albedo(imagic.context_mut());
        // equirectangular_to_cube_material.set_equirectangular_map(albedo_index);

        let material_index = imagic_context
            .material_manager_mut()
            .add_material(equirectangular_to_cube_material);
        material_index
    }

    pub fn run(self) {
        let app:Box<dyn ImagicAppTrait> = Box::new(self);
        let mut imagic = Imagic::new(app);
        imagic.run();
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic_context: &mut ImagicContext) {
        let fov = consts::FRAC_PI_4;
        let aspect = self.window_size.get_half_width() / self.window_size.get_height();
        let near = 0.01;
        let far = 500.0;

        // first camera
        let first_viewport = Vec4::new(0.0, 0.0, 0.5, 1.0);
        let first_clear_color = Vec4::new(0.1, 0.1, 0.1, 1.0);
        let first_camera_pos = Vec3::new(0.0, 0.0, self.camera_z);
        self.first_camera_id = create_camera(
            imagic_context,
            first_camera_pos,
            first_viewport,
            first_clear_color,
            fov,
            aspect,
            near,
            far,
            LayerMask::default(),
            Some(self.camera_controller_option_1),
        );

        let second_viewport = Vec4::new(0.5, 0.0, 0.5, 1.0);
        let second_clear_color = Vec4::new(0.1, 0.2, 0.3, 1.0);
        let second_camera_pos = Vec3::new(0.0, 0.0, self.camera_z);
        self.second_camera_id = create_camera(
            imagic_context,
            second_camera_pos,
            second_viewport,
            second_clear_color,
            fov,
            aspect,
            near,
            far,
            LayerMask::default(),
            Some(self.camera_controller_option_2),
        );

        let material_index = self.prepare_material(imagic_context);
        self.cube.init(imagic_context, material_index);
    }

    fn on_update(&mut self, imagic_context: &mut ImagicContext) {
        if self.need_update_camera_controller {
            self.need_update_camera_controller = false;
            imagic_context.change_camera_controller(self.first_camera_id, &self.camera_controller_option_1);
        }
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - Multi Cameras")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .default_size([100.0, 10.0])
            .show(&ctx, |ui| {
                let rotate_button_text = 
                if self.camera_controller_option_1.is_auto_rotate {
                    "Stop Auto Rotate"
                } else {
                    "Auto Rotate"
                };
                if ui.button(rotate_button_text).clicked() {
                    self.camera_controller_option_1.is_auto_rotate = !self.camera_controller_option_1.is_auto_rotate;
                    self.need_update_camera_controller = true;
                }
            });
    }

    fn get_imagic_option(&self) -> ImagicOption {
        ImagicOption::new(self.window_size, "multi_camera Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("cube main.");
    let app: App = Default::default();
    app.run();
}
