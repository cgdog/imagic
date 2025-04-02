use std::f32::consts;

use imagic::{asset::asset::Handle, ecs::world::World, prelude::*, window::WindowSize};
use log::info;

pub struct App {
    cube: Cube,
    camera: usize,
    window_size: WindowSize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cube: Cube::new(1.0, 1.0, 1.0, 1, 1, 1),
            camera: INVALID_ID,
            window_size: WindowSize::new(500.0, 500.0),
        }
    }
}

impl App {
    fn prepare_albedo(&mut self, world: &mut World) -> Handle<Texture> {
        let albedo_texture = Texture::create_from_bytes(
            world.context().graphics_context(),
            include_bytes!("./assets/lena.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            false,
            true,
        );

        world.asset_manager_mut().add(albedo_texture)
    }

    fn prepare_material(&mut self, world: &mut World) -> ID {
        let mut unlit_material = Box::new(UnlitMaterial::new());
        let albedo_index = self.prepare_albedo(world);
        unlit_material.set_albedo_map(albedo_index);

        let material_index = world.context_mut().add_material(unlit_material);
        material_index
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, world: &mut World) {
        // self.prepare_lights(imagic_context);
        self.camera = Camera::new(
            Vec3::new(0.0, 5.0, 5.0),
            consts::FRAC_PI_4,
            self.window_size.get_aspect(),
            1.0,
            100.0,
            Some(CameraControllerOptions::new(Vec3::ZERO, false)),
            world.context_mut(),
        );

        let material_index = self.prepare_material(world);
        self.cube.init(world.context_mut(), material_index);
    }

    fn get_imagic_option(&self) -> ImagicOption {
        ImagicOption::new(self.window_size, "Cube Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("cube main.");

    let mut imagic = Imagic::new(Box::new(App::default()));
    imagic.run();
}
