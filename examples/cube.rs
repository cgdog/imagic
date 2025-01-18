use std::{cell::RefCell, f32::consts, rc::Rc};

use log::info;
use imagic::{prelude::*, window::WindowSize};

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

    fn _prepare_hdr_texture(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut hdr_loader = HDRLoader{};
        let cwd = std::env::current_dir().unwrap();
        let hdr_path = cwd.join("examples/assets/pbr/hdr/newport_loft.hdr");
        let hdr_texture = hdr_loader.load(hdr_path.to_str().unwrap(), imagic_context.graphics_context());
        let hdr_texture_index = imagic_context.texture_manager_mut().add_texture(hdr_texture);
        hdr_texture_index
    }

    fn _prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let albedo_texture = Texture::create_from_bytes(imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let albedo_texture_index = imagic_context.texture_manager_mut().add_texture(albedo_texture);
        albedo_texture_index
    }

    fn prepare_material(&mut self, imagic: &mut Imagic) -> ID {
        let mut unlit_material = Box::new(UnlitMaterial::new());
        let albedo_index = self._prepare_albedo(imagic.context_mut());
        unlit_material.set_albedo_map(albedo_index);
        
        let material_index = imagic.context_mut().material_manager_mut().add_material(unlit_material);
        material_index
    }
    
    pub fn run(self) {
        let mut imagic = Imagic::new();

        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(app);
    }
}

impl ImagicAppTrait for App {

    fn init(&mut self, imagic: &mut Imagic) {
        let imagic_context = imagic.context_mut();
        // self.prepare_lights(imagic_context);
        self.camera = Camera::new(Vec3::new(0.0, 5.0, 10.0), consts::FRAC_PI_4
            , self.window_size.get_aspect(), 1.0, 100.0, Some(CameraControllerOptions::new(Vec3::ZERO, false)), imagic_context);

        let material_index = self.prepare_material(imagic);
        self.cube.init(imagic, material_index);
    }

    fn on_update(&mut self, _imagic_context: &mut ImagicContext) {
        // todo!()
    }

    fn on_render_ui(&mut self, _ctx: &egui::Context) {
        // todo!()
    }

    fn get_imagic_option(& self) -> ImagicOption {
        ImagicOption::new(self.window_size, "Cube Demo")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("cube main.");
    let app: App = Default::default();
    app.run();
}