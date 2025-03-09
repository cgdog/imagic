use imagic::image::{ImageBuffer, Rgb};
use imagic::{prelude::{raytracer::RayTracer, *}, window::WindowSize};
use log::info;

struct App {
    window_size: WindowSize,
    ray_tracer: RayTracer,
    img: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
}

impl Default for App {
    fn default() -> Self {
        let width = 1024;
        let heigit = 768;
        Self {
            window_size: WindowSize::new(width as f32, heigit as f32),
            ray_tracer: RayTracer::new(width, heigit),
            img: None,
        }
    }
}

impl ImagicAppTrait for App {
    fn init(&mut self, _imagic: &mut ImagicContext) {
        
    }

    fn get_imagic_option(& self) -> ImagicOption {
        ImagicOption::new(self.window_size, "Raytracer")
    }

    fn on_render_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Imagic - Raytracer")
            .resizable(true)
            .vscroll(true)
            .default_open(true)
            .default_size([100.0, 10.0])
            .show(&ctx, |ui| {
                if ui.button("Raytracing").clicked() {
                    let img = self.ray_tracer.render();
                    img.save("tracer_result.png").unwrap();
                    self.img = Some(img);
                }
                if let Some(_img) = &self.img {
                    // ui.image(img);
                    ui.label("See the result tracer_result.png at the root folder");
                } else {
                    ui.label("Waiting...");
                }
            });
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut raytracer = RayTracer::new(1024, 768);
    let imgbuf = raytracer.render();
    imgbuf.save("tracer_out.png").unwrap();

    info!("Finished. See the tracer_result.png at the root folder");
    // let mut imagic = Imagic::new(Box::new(App::default()));
    // imagic.run();
}