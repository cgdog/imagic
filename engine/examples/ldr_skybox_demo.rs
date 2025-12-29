use imagic::prelude::*;

struct Game {
    engine: Box::<Engine>,
     _sphere: NodeHandle,
}

struct GameBehavior {
    material: MaterialHandle,
    metallic_roughness_ao: Vec4,
}

impl Behavior for GameBehavior {
    impl_as_any!();

    fn on_start(&mut self, _logic_context: &mut LogicContext) {
        log::info!("Game started.");
    }

    fn on_gui(&mut self, logic_context: &mut LogicContext, ctx: &egui::Context) {
        egui::Window::new("Skybox LOD Control").show(ctx, |ui| {
            if ui.add(
                egui::Slider::new(&mut self.metallic_roughness_ao.x, 0.0..=1.0)
                    .text("metallic")
                    .step_by(0.1),
            ).changed() {
                logic_context.material_manager.get_material_mut_forcely(&self.material)
                    .set_vec4f("_metallic_roughness_ao", self.metallic_roughness_ao);
            }

            if ui.add(
                egui::Slider::new(&mut self.metallic_roughness_ao.y, 0.0..=1.0)
                    .text("roughness")
                    .step_by(0.1),
            ).changed() {
                logic_context.material_manager.get_material_mut_forcely(&self.material)
                    .set_vec4f("_metallic_roughness_ao", self.metallic_roughness_ao);
            }
        });
    }
}

impl Game {
    pub fn new() -> Self {
        let engine_options = EngineOptions {
            window_size: WindowSize::new(800.0, 500.0),
            app_name: "lxy LDR skybox demo",
        };
        let mut engine = Engine::new(engine_options);
        let material = Self::create_material(&mut engine);
        let sphere = Self::create_sphere(&mut engine, material);
        let game_behavior = GameBehavior { material: material, metallic_roughness_ao: Vec4::new(0.0, 1.0, 1.0, 1.0) };
        engine.add_behavior(game_behavior);
        Self { engine, _sphere: sphere }
    }

    fn create_material(engine: &mut Engine) -> MaterialHandle {
        let (_, pbr_shader) = engine.shader_manager.get_builtin_pbr_shader();
        let pbr_material = engine.material_manager.create_material(*pbr_shader, &mut engine.shader_manager);
        {
            let pbr_material_mut_ref = engine.material_manager.get_material_mut_forcely(&pbr_material);
            pbr_material_mut_ref.set_vec4f("_albedo_color", Vec4::new(1.0, 1.0, 1.0, 1.0));
            pbr_material_mut_ref.set_vec4f("_metallic_roughness_ao", Vec4::new(0.0, 1.0, 1.0, 1.0));
            // _reflection_cube_sampler
            let reflection_cube_sampler = engine.texture_sampler_manager.create_sampler(
                AddressMode::ClampToEdge,
                AddressMode::ClampToEdge,
                AddressMode::ClampToEdge,
                FilterMode::Linear,
                FilterMode::Linear,
                FilterMode::Nearest,
            );
            pbr_material_mut_ref.set_sampler("_reflection_cube_sampler", reflection_cube_sampler);
        }
        pbr_material
    }

    fn create_sphere(engine: &mut Engine, material: MaterialHandle) -> NodeHandle {
        let scene = engine.world.current_scene_mut();
        let uv_sphere_node = scene.create_node("UVSphere");
        let mesh: Mesh = UVSphere::default().into();
        let mesh_handle = engine.mesh_manager.add_mesh(mesh);
        let mesh_renderer = MeshRenderer::new(mesh_handle, vec![material]);
        scene.add_component(&uv_sphere_node, mesh_renderer);
        scene.add(uv_sphere_node.clone());
        uv_sphere_node
    }

    fn init(&mut self) {
        self.create_camera();
        let _skybox_node = SkyboxBuilder::create_skybox(
            &mut self.engine,
            vec![
                include_bytes!("./assets/images/ldr_skybox/right.jpg").to_vec(),
                include_bytes!("./assets/images/ldr_skybox/left.jpg").to_vec(),
                include_bytes!("./assets/images/ldr_skybox/top.jpg").to_vec(),
                include_bytes!("./assets/images/ldr_skybox/bottom.jpg").to_vec(),
                include_bytes!("./assets/images/ldr_skybox/front.jpg").to_vec(),
                include_bytes!("./assets/images/ldr_skybox//back.jpg").to_vec(),
            ],
            TextureFormat::Rgba8UnormSrgb,
            SkyboxBuilderOptions::default(),
        );
    }

    fn create_camera(&mut self) {
        
        let current_scene = self.engine.world.current_scene_mut();
        let camera_node = current_scene.create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        camera.fov = std::f32::consts::FRAC_PI_4;
        current_scene.get_node_mut_forcely(&camera_node).transform
            .set_position(Vec3::new(0.0, 1.5, 1.5)); // front
        current_scene.add_component(&camera_node, camera);

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
