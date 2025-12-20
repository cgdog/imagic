use imagic::prelude::*;

struct Game {
    engine: Box<Engine>,
}

impl Game {
    pub fn new() -> Self {
        let engine_options = EngineOptions {
            window_size: WindowSize::new(500.0, 500.0),
            app_name: "lxy primitives demo",
        };
        let engine = Engine::new(engine_options);
        Self { engine }
    }

    fn init(&mut self) {
        self.create_camera();
        let _skybox_node = SkyboxBuilder::create_skybox(
            &mut self.engine,
            vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
            TextureFormat::Rgba32Float,
            SkyboxBuilderOptions::default(),
        );
    }

    fn create_camera(&mut self) {
        

        let current_scene = self.engine.world.current_scene_mut();
        let camera_node = current_scene.create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        current_scene.add_component(&camera_node, camera);
        current_scene.get_node_mut_forcely(&camera_node).transform.set_position(Vec3::new(0.0, 4.0, 8.0));

        current_scene.add(camera_node);
        let camera_controller =
            CameraController::new(camera_node, CameraTarget::Position(Vec3::ZERO));
        self.engine.add_behavior(camera_controller);
    }

    pub fn run(&mut self) {
        self.init();
        self.engine.run();
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut game = Game::new();
    game.run();
}
