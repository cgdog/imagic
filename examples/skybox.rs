use std::{cell::RefCell, f32::consts, rc::Rc};
use std::time::{SystemTime, UNIX_EPOCH};

use log::info;
use imagic::prelude::*;

pub struct App {
    cube: Cube,
    camera: usize,
    window_size: (f64, f64),
    camera_z: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cube: Cube::new(1.0, 1.0, 1.0, 1, 1, 1),
            camera: usize::MAX,
            window_size: (500.0, 500.0),
            camera_z: 8.0,
        }
    }
}

impl App {

    fn prepare_skybox(&mut self, imagic_context: &mut ImagicContext) -> usize {
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
        let mut equirectangular_to_cube_material = Box::new(EquirectangularToCubeMaterial::new());
        let skybox_texture = self.prepare_skybox(imagic.context_mut());
        equirectangular_to_cube_material.set_equirectangular_map(skybox_texture);
        // let albedo_index = self._prepare_albedo(imagic.context_mut());
        // equirectangular_to_cube_material.set_equirectangular_map(albedo_index);
        
        let material_index = imagic.context_mut().material_manager_mut().add_material(equirectangular_to_cube_material);
        material_index
    }

    fn init(&mut self, imagic: &mut Imagic) {
        let imagic_context = imagic.context_mut();
        // self.prepare_lights(imagic_context);
        self.camera = Camera::new(glam::Vec3::new(0.0, 0.0, self.camera_z), consts::FRAC_PI_4
            , (self.window_size.0 / self.window_size.1) as f32, 1.0, 100.0, imagic_context);

        let material_index = self.prepare_material(imagic);
        self.cube.init(imagic, material_index);
    }
    
    pub fn run(mut self) {
        let mut imagic = Imagic::new();
        let event_loop = imagic.init(ImagicOption::new(self.window_size.0, self.window_size.1, "Cube Demo"));

        self.init(&mut imagic);

        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(event_loop, app);
    }

    fn _rotate_camera(&mut self, imagic_context: &mut ImagicContext) {
        let camera_transform = *imagic_context.camera_manager().get_camera(self.camera).transform();
        // let cur_camera_pos = imagic_context.transform_manager().get_transform(camera_transform).get_position();
        let cur_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        // info!("cur_time: {}", cur_time);
        let camera_new_pos = glam::Vec3::new(self.camera_z * cur_time.cos() as f32, 4.5, self.camera_z * cur_time.sin() as f32);
        imagic_context.transform_manager_mut().get_transform_mut(camera_transform).set_position(camera_new_pos);
        let camera = imagic_context.camera_manager().get_camera(self.camera);
        camera.update_uniform_buffers(imagic_context.graphics_context(), imagic_context.transform_manager(), imagic_context.buffer_manager());
    }
}

impl ImagicAppTrait for App {
    fn on_update(&mut self, imagic_context: &mut ImagicContext, _ui_renderer: &mut UIRenderer) {
        self._rotate_camera(imagic_context);
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