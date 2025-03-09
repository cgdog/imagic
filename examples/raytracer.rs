use imagic::prelude::raytracer::light::Light;
use imagic::prelude::raytracer::material::Material;
use imagic::prelude::raytracer::primitives::model::Model;
use imagic::prelude::raytracer::primitives::quad::Quad;
use imagic::prelude::raytracer::primitives::sphere::Sphere;
use imagic::prelude::raytracer::primitives::Primitive;
use imagic::prelude::{raytracer::RayTracer, *};
use log::info;

// #[allow(unused)]
// struct App {
//     window_size: WindowSize,
//     ray_tracer: RayTracer,
//     img: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
// }

// impl Default for App {
//     fn default() -> Self {
//         let width = 1024;
//         let heigit = 768;
//         Self {
//             window_size: WindowSize::new(width as f32, heigit as f32),
//             ray_tracer: RayTracer::new(width, heigit, None),
//             img: None,
//         }
//     }
// }

// impl ImagicAppTrait for App {
//     fn init(&mut self, _imagic: &mut ImagicContext) {}

//     fn get_imagic_option(&self) -> ImagicOption {
//         ImagicOption::new(self.window_size, "Raytracer")
//     }

//     fn on_render_ui(&mut self, ctx: &egui::Context) {
//         egui::Window::new("Imagic - Raytracer")
//             .resizable(true)
//             .vscroll(true)
//             .default_open(true)
//             .default_size([100.0, 10.0])
//             .show(&ctx, |ui| {
//                 if ui.button("Raytracing").clicked() {
//                     // let img = self.ray_tracer.render();
//                     // img.save("tracer_result.png").unwrap();
//                     // self.img = Some(img);
//                 }
//                 if let Some(_img) = &self.img {
//                     // ui.image(img);
//                     ui.label("See the result tracer_result.png at the root folder");
//                 } else {
//                     ui.label("Waiting...");
//                 }
//             });
//     }
// }

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let env_map = image::load_from_memory(include_bytes!("./assets/envmap.jpg"))
        .expect("Failed to load envmap.jpg");
    let env_map = env_map.flipv();
    let env_map = env_map.to_rgb8();

    let mut raytracer = RayTracer::new(1024, 768, Some(env_map));
    let ivory = Material::new(
        1.0,
        Vec4::new(0.6, 0.3, 0.1, 0.0),
        ColorRGB::new(0.4, 0.4, 0.3),
        50.0,
    );
    let glass = Material::new(
        1.5,
        Vec4::new(0.0, 0.5, 0.1, 0.8),
        ColorRGB::new(0.6, 0.7, 0.8),
        125.0,
    );
    let red_rubber = Material::new(
        1.0,
        Vec4::new(0.9, 0.1, 0.0, 0.0),
        ColorRGB::new(0.3, 0.1, 0.1),
        10.0,
    );
    let mirror = Material::new(
        1.0,
        Vec4::new(0.0, 10.0, 0.8, 0.0),
        ColorRGB::new(1.0, 1.0, 1.0),
        1425.0,
    );

    let duck = Material::new(
        1.2,
        Vec4::new(0.0, 0.5, 0.1, 0.8),
        ColorRGB::new(0.6, 0.7, 0.8),
        125.0,
    );

    // let duck = Material::new(
    //     1.0,
    //     Vec4::new(0.9, 0.1, 0.0, 0.0),
    //     ColorRGB::new(0.3, 0.3, 0.1),
    //     10.0,
    // );
    let duck_position = Vec3::new(-12.5, 4.0, -5.0);

    let mut renderable_items: Vec<Box<dyn Primitive>> = vec![
        Box::new(Sphere::new(Vec3::new(-3.0, 0.0, -16.0), 2.0, ivory)),
        Box::new(Sphere::new(Vec3::new(-1.0, -1.5, -12.0), 2.0, glass)),
        Box::new(Sphere::new(Vec3::new(1.5, -0.5, -18.0), 3.0, red_rubber)),
        Box::new(Sphere::new(Vec3::new(7.0, 5.0, -18.0), 4.0, mirror)),
        // the checker pattern plane
        Box::new(Quad::checker_pattern()),
        // the duck
        Box::new(Model::new("examples/assets/models/duck.obj", duck, duck_position)),
    ];

    let lights = vec![
        Light::new(Vec3::new(-20.0, 20.0, 20.0), 1.5),
        Light::new(Vec3::new(30.0, 50.0, -25.0), 1.8),
        Light::new(Vec3::new(30.0, 20.0, 30.0), 1.7),
    ];

    let imgbuf = raytracer.render(&mut renderable_items, &lights);
    imgbuf.save("tracer_result.png").unwrap();

    info!("Finished. See the tracer_result.png at the root folder");
    // let mut imagic = Imagic::new(Box::new(App::default()));
    // imagic.run();
}
