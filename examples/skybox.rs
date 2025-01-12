use std::{cell::RefCell, f32::consts, rc::Rc};
use std::time::{SystemTime, UNIX_EPOCH};

use imagic::window::WindowSize;
use log::info;
use imagic::prelude::*;

pub struct App {
    cube: Cube,
    sphere: Sphere,
    first_camera_id: ID,
    second_camera_id: ID,
    window_size: WindowSize,
    camera_z: f32,
    rotate_camera: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cube: Cube::new(1.0, 1.0, 1.0, 1, 1, 1),
            sphere: Sphere::new(1.0, 256, 256),
            first_camera_id: INVALID_ID,
            second_camera_id: INVALID_ID,
            window_size: WindowSize::new(800.0, 500.0),
            camera_z: 8.0,
            rotate_camera: true,
        }
    }
}

impl App {

    fn prepare_lights(&mut self, imagic_context: &mut ImagicContext) {

        let transform_manager = imagic_context.transform_manager_mut();
        let point_light_0 = PointLight::new(
            Vec3::new(-10.0,  10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            transform_manager
        );
        let point_light_1 = PointLight::new(
            Vec3::new(10.0,  10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            transform_manager
        );
        let point_light_2 = PointLight::new(
            Vec3::new(-10.0,  -10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            transform_manager
        );
        let point_light_3 = PointLight::new(
            Vec3::new(10.0,  -10.0, 10.0),
            ColorRGB::new(300.0, 300.0, 300.0),
            transform_manager
        );

        let light_manager = imagic_context.light_manager_mut();
        light_manager.add_point_light(point_light_0);
        light_manager.add_point_light(point_light_1);
        light_manager.add_point_light(point_light_2);
        light_manager.add_point_light(point_light_3);
    }

    fn prepare_pbr_material(&mut self, imagic: &mut Imagic) -> ID {
        let graphics_context = imagic.context().graphics_context();
        let mut pbr_material = Box::new(PBRMaterial::new(Vec4::new(1.0, 1.0, 1.0, 1.0), 1.0, 1.0, 1.0));
        let albedo_texture = Texture::create_from_bytes(graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_basecolor.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let normal_texture = Texture::create_from_bytes(graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_normal.png"), wgpu::TextureFormat::Rgba8Unorm);
        let metallic_texture = Texture::create_from_bytes(graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_metallic.png"), wgpu::TextureFormat::Rgba8Unorm);
        let roughness_texture = Texture::create_from_bytes(graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/rustediron2_roughness.png"), wgpu::TextureFormat::Rgba8Unorm);
        let ao_texture = Texture::create_from_bytes(graphics_context,
            include_bytes!("./assets/pbr/rustediron1-alt2-bl/ao.png"), wgpu::TextureFormat::Rgba8Unorm);
        
        let texture_manager = imagic.context_mut().texture_manager_mut();

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

        let pbr_material_index = imagic.context_mut().material_manager_mut().add_material(pbr_material);
        pbr_material_index
    }


    fn prepare_skybox(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let mut hdr_loader = HDRLoader{};
        let cwd = std::env::current_dir().unwrap();
        let hdr_path = cwd.join("examples/assets/pbr/hdr/newport_loft.hdr");
        let hdr_texture = hdr_loader.load(hdr_path.to_str().unwrap(), imagic_context.graphics_context());
        let hdr_texture_index = imagic_context.texture_manager_mut().add_texture(hdr_texture);
        hdr_texture_index
    }

    fn prepare_ldr_skybox(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let cube_texture = Texture::create_cube_texture_from_bytes(
            imagic_context.graphics_context(),
            [
                include_bytes!("./assets/skybox/right.jpg"),
                include_bytes!("./assets/skybox/right.jpg"),
                include_bytes!("./assets/skybox/right.jpg"),
                include_bytes!("./assets/skybox/right.jpg"),
                include_bytes!("./assets/skybox/right.jpg"),
                include_bytes!("./assets/skybox/right.jpg"),
            ],
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let cube_texture_id = imagic_context.texture_manager_mut().add_texture(cube_texture);
        
        let mut skybox_material = Box::new(SkyboxMaterial::new());
        skybox_material.set_skybox_map(cube_texture_id);
        // skybox_material.set_cull_mode(wgpu::Face::Back);
        let skybox_material_id = imagic_context.material_manager_mut().add_material(skybox_material);
        skybox_material_id
    }

    fn _prepare_albedo(&mut self, imagic_context: &mut ImagicContext) -> ID {
        let albedo_texture = Texture::create_from_bytes(imagic_context.graphics_context(),
            include_bytes!("./assets/lena.png"), wgpu::TextureFormat::Rgba8UnormSrgb);
        let albedo_texture_index = imagic_context.texture_manager_mut().add_texture(albedo_texture);
        albedo_texture_index
    }

    #[allow(unused)]
    fn prepare_equirect_to_cube_material(&mut self, imagic: &mut Imagic) -> ID {
        let mut equirectangular_to_cube_material = Box::new(EquirectangularToCubeMaterial::new());
        let skybox_texture = self.prepare_skybox(imagic.context_mut());
        equirectangular_to_cube_material.set_equirectangular_map(skybox_texture);
        
        let material_index = imagic.context_mut().material_manager_mut().add_material(equirectangular_to_cube_material);
        material_index
    }

    fn add_camera(&mut self, imagic: &mut Imagic, camera_pos: Vec3, viewport: Vec4, clear_color: Vec4, layer_mask: LayerMask) -> ID {
        let imagic_context = imagic.context_mut();
        let camera_id = Camera::new(camera_pos, consts::FRAC_PI_4
            , self.window_size.get_half_width() / self.window_size.get_height(), 0.01, 500.0, imagic_context);
        
        let camera = imagic.context_mut().camera_manager_mut().get_camera_mut(camera_id);
        camera.set_viewport(viewport);
        camera.set_clear_color(clear_color);
        camera.layer_mask = layer_mask;
        camera_id
    }

    fn init(&mut self, imagic: &mut Imagic) {
        // first camera
        let first_viewport = Vec4::new(0.0, 0.0, 0.5, 1.0);
        let first_clear_color = Color::new(0.1, 0.1, 0.1, 1.0);
        let first_camera_pos = Vec3::new(0.0, 0.0, self.camera_z);
        // TODO: 让 Cube 以天空盒的形式渲染
        let first_camera_layer_mask = LayerMask::new(Layer::Default.into());
        self.first_camera_id = self.add_camera(imagic, first_camera_pos, first_viewport, first_clear_color, first_camera_layer_mask);

        let second_viewport = Vec4::new(0.5, 0.0, 0.5, 1.0);
        let second_clear_color = Color::new(0.1, 0.2, 0.3, 1.0);
        let second_camera_pos = Vec3::new(0.0, 0.0, self.camera_z);
        let second_camera_layer_mask = LayerMask::new(Layer::RenderTarget.into());
        self.second_camera_id = self.add_camera(imagic, second_camera_pos, second_viewport, second_clear_color, second_camera_layer_mask);

        // let equirect_to_cube_material_index = self.prepare_equirect_to_cube_material(imagic);
        let skybox_material_id = self.prepare_ldr_skybox(imagic.context_mut());
        self.cube.init(imagic, skybox_material_id);
        self.cube.set_layer(Layer::RenderTarget, imagic.context_mut().render_item_manager_mut());

        self.prepare_lights(imagic.context_mut());

        let pbr_material_index = self.prepare_pbr_material(imagic);
        self.sphere.init(imagic, pbr_material_index);
        // self.sphere.set_layer(Layer::Custom1, imagic.context_mut().render_item_manager_mut());
        
    }
    
    pub fn run(mut self) {
        let mut imagic = Imagic::new();
        let event_loop = imagic.init(ImagicOption::new(self.window_size, "Skybox Demo"));

        self.init(&mut imagic);

        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(event_loop, app);
    }

    fn _rotate_camera(&mut self, imagic_context: &mut ImagicContext, camera_id: ID) {
        let camera_transform = *imagic_context.camera_manager().get_camera(camera_id).transform();
        // let cur_camera_pos = imagic_context.transform_manager().get_transform(camera_transform).get_position();
        let cur_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        // info!("cur_time: {}", cur_time);
        let camera_new_pos = Vec3::new(self.camera_z * cur_time.cos() as f32, 4.5, self.camera_z * cur_time.sin() as f32);
        imagic_context.transform_manager_mut().get_transform_mut(camera_transform).set_position(camera_new_pos);
        let camera = imagic_context.camera_manager().get_camera(camera_id);
        camera.update_uniform_buffers(imagic_context.graphics_context(), imagic_context.transform_manager(), imagic_context.buffer_manager());
    }
}

impl ImagicAppTrait for App {
    fn on_update(&mut self, _imagic_context: &mut ImagicContext, _ui_renderer: &mut UIRenderer) {
        if self.rotate_camera {
            self._rotate_camera(_imagic_context, self.first_camera_id);
            self._rotate_camera(_imagic_context, self.second_camera_id);
        }
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - Skybox")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .default_size([100.0, 5.0])
        .show(&ctx, |ui| {
            if self.rotate_camera {
                if ui.button("Stop Rotate").clicked() {
                    info!("Stop Rotate");
                    self.rotate_camera = !self.rotate_camera;
                }
            } else {
                if ui.button("Rotate").clicked() {
                    info!("Rotate.");
                    self.rotate_camera = !self.rotate_camera;
                }
            }
        });
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("cube main.");
    let app: App = Default::default();
    app.run();
}