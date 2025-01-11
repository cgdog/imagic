use std::{cell::RefCell, f32::consts, rc::Rc};

use log::info;
use imagic::{prelude::*, window::WindowSize};
use math::Vec3;

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

    fn _prepare_skybox(&mut self, imagic_context: &mut ImagicContext) -> usize {
        let mut hdr_loader = HDRLoader{};
        let cwd = std::env::current_dir().unwrap();
        let hdr_path = cwd.join("examples/assets/pbr/hdr/newport_loft.hdr");
        let hdr_texture = hdr_loader.load(hdr_path.to_str().unwrap(), imagic_context.graphics_context());
        let hdr_texture_index = imagic_context.texture_manager_mut().add_texture(hdr_texture);
        hdr_texture_index
    }

    fn _prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> usize {
        let albedo_texture = Texture::create_from_bytes(imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let albedo_texture_index = imagic_context.texture_manager_mut().add_texture(albedo_texture);
        albedo_texture_index
    }

    fn prepare_material(&mut self, imagic: &mut Imagic) -> usize {
        let mut skybox_material = Box::new(SkyboxMaterial::new());
        // let skybox_texture = self._prepare_skybox(imagic.context_mut());
        // skybox_material.set_skybox_map(skybox_texture);
        let albedo_index = self._prepare_albedo(imagic.context_mut());
        skybox_material.set_skybox_map(albedo_index);
        
        let material_index = imagic.context_mut().material_manager_mut().add_material(skybox_material);
        material_index
    }

    fn init(&mut self, imagic: &mut Imagic) {
        let imagic_context = imagic.context_mut();
        // self.prepare_lights(imagic_context);
        self.camera = Camera::new(Vec3::new(0.0, 5.0, 10.0), consts::FRAC_PI_4
            , self.window_size.get_aspect(), 1.0, 100.0, imagic_context);

        let material_index = self.prepare_material(imagic);
        self.cube.init(imagic, material_index);
    }
    
    pub fn run(mut self) {
        let mut imagic = Imagic::new();
        let event_loop = imagic.init(ImagicOption::new(self.window_size, "Cube Demo"));

        self.init(&mut imagic);

        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(event_loop, app);
    }
}

impl ImagicAppTrait for App {
    fn on_update(&mut self, _imagic_context: &mut ImagicContext, _ui_renderer: &mut UIRenderer) {
        // todo!()
    }

    fn on_render_ui(&mut self, _ctx: &egui::Context) {
        // todo!()
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("cube main.");
    let app: App = Default::default();
    app.run();
}