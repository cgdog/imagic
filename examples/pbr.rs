use std::{cell::RefCell, f32::consts, rc::Rc};

use log::info;
use imagic::prelude::*;

pub struct App {
    sphere: Sphere,
    camera: usize,
    window_size: (f64, f64),
}

impl Default for App {
    fn default() -> Self {
        Self {
            sphere: Sphere::new(1.0, 256, 256),
            camera: usize::MAX,
            window_size: (500.0, 500.0),
        }
    }
}

impl App {
    
    fn prepare_lights(&mut self, imagic_context: &mut ImagicContext) {

        let transform_manager = imagic_context.transform_manager_mut();
        let point_light_0 = PointLight::new(
            glam::Vec3::new(-10.0,  10.0, 10.0),
            glam::Vec3::new(300.0, 300.0, 300.0),
            transform_manager
        );
        let point_light_1 = PointLight::new(
            glam::Vec3::new(10.0,  10.0, 10.0),
            glam::Vec3::new(300.0, 300.0, 300.0),
            transform_manager
        );
        let point_light_2 = PointLight::new(
            glam::Vec3::new(-10.0,  -10.0, 10.0),
            glam::Vec3::new(300.0, 300.0, 300.0),
            transform_manager
        );
        let point_light_3 = PointLight::new(
            glam::Vec3::new(10.0,  -10.0, 10.0),
            glam::Vec3::new(300.0, 300.0, 300.0),
            transform_manager
        );

        let light_manager = imagic_context.light_manager_mut();
        light_manager.add_point_light(point_light_0);
        light_manager.add_point_light(point_light_1);
        light_manager.add_point_light(point_light_2);
        light_manager.add_point_light(point_light_3);
    }

    fn prepare_material(&mut self, imagic: &mut Imagic) -> usize {
        let graphics_context = imagic.context().graphics_context();
        let mut pbr_material = Box::new(PBRMaterial::new(glam::Vec4::new(1.0, 1.0, 1.0, 1.0), 1.0, 1.0, 1.0));
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

    fn init(&mut self, imagic: &mut Imagic) {
        let imagic_context = imagic.context_mut();

        self.prepare_lights(imagic_context);

        self.camera = Camera::new(glam::Vec3::new(0.0, 1.0, 4.0), consts::FRAC_PI_4
            , 1.0, 1.0, 10.0, imagic_context);

        let pbr_material_index = self.prepare_material(imagic);
        self.sphere.init(imagic, pbr_material_index);
    }

    pub fn run(mut self) {
        let mut imagic = Imagic::new();
        let event_loop = imagic.init(ImagicOption::new(self.window_size.0, self.window_size.1, "PBR Demo"));

        self.init(&mut imagic);

        let app: Rc<RefCell<Box<dyn ImagicAppTrait>>> = Rc::new(RefCell::new(Box::new(self)));
        imagic.run(event_loop, app);
    }
}

impl ImagicAppTrait for App {
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("pbr main.");
    let app: App = Default::default();
    app.run();
}