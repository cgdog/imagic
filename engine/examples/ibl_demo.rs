use imagic::prelude::*;

struct Game {
    engine: Box<Engine>,
    _sphere: NodeId,
}

struct GameBehavior {
    material: RR<Material>,
    metallic_roughness_ao: Vec4,
}

impl Behavior for GameBehavior {
    impl_as_any!();

    fn on_start(&mut self, _logic_context: &mut LogicContext) {
        log::info!("Game started.");
        // let scene = _logic_context.world.current_scene_mut();
        // let cuboid_node: std::rc::Rc<std::cell::RefCell<Node>> =
        //     Node::new_with_name(scene, "Cuboid");
        // {
        //     let mut cuboid_node_mut_ref = cuboid_node.borrow_mut();
        //     cuboid_node_mut_ref.transform.set_position_x(-1.5);
        //     let mesh: Mesh = Cuboid::default().into();
        //     let mesh = RR_new!(mesh);
        //     let mesh_renderer = MeshRenderer::new(mesh, vec![self.material.clone()]);
        //     cuboid_node_mut_ref.add_component(mesh_renderer);
        // }
        // scene.add(cuboid_node);
    }

    fn on_gui(&mut self, engine: &mut LogicContext, ctx: &egui::Context) {
        egui::Window::new("Skybox LOD Control").show(ctx, |ui| {
            ui.label(format!("FPS: {}", engine.performance_tracker.fps_counter.fps));
            if ui.add(
                egui::Slider::new(&mut self.metallic_roughness_ao.x, 0.0..=1.0)
                    .text("metallic")
                    .step_by(0.1),
            ).changed() {
                self.material.borrow_mut().set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO, self.metallic_roughness_ao);
            }

            if ui.add(
                egui::Slider::new(&mut self.metallic_roughness_ao.y, 0.0..=1.0)
                    .text("roughness")
                    .step_by(0.1),
            ).changed() {
                self.material.borrow_mut().set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO, self.metallic_roughness_ao);
            }
        });
    }
}

impl Game {
    pub fn new() -> Self {
        let engine_options = EngineOptions {
            window_size: WindowSize::new(800.0, 500.0),
            app_name: "lxy ibl demo",
        };
        let mut engine = Engine::new(engine_options);
        let material = Self::create_material(&mut engine);
        let sphere = Self::create_sphere(&mut engine.world, material.clone());
        let game_behavior = GameBehavior { material: material, metallic_roughness_ao: Vec4::new(0.0, 1.0, 1.0, 1.0) };
        engine.add_behavior(game_behavior);
        Self { engine, _sphere: sphere }
    }

    fn create_material(engine: &mut Engine) -> RR<Material> {
        let pbr_shader = engine.shader_manager.get_builtin_pbr_shader();
        let pbr_material = Material::new(pbr_shader);
        {
            let mut pbr_material_mut_ref = pbr_material.borrow_mut();
            pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_ALBEDO_COLOR, Vec4::new(1.0, 1.0, 1.0, 1.0));
            pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO, Vec4::new(0.0, 1.0, 1.0, 1.0));
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

    fn create_sphere(world: &mut World, material: RR<Material>) -> NodeId {
        let scene = world.current_scene_mut();
        let uv_sphere_node = scene.create_node("UVSphere");
        {
            let mesh: Mesh = UVSphere::default().into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![material]);
            scene.add_component::<MeshRenderer>(&uv_sphere_node, mesh_renderer);
        }
        scene.add(uv_sphere_node.clone());
        uv_sphere_node
    }

    fn init(&mut self) {
        self.create_camera();
        let _skybox_node = SkyboxBuilder::create_skybox(
            &mut self.engine,
            // vec![include_bytes!("./assets/images/hdri/resting_place_2_1k.hdr").to_vec()],
            vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
            // vec![include_bytes!("./assets/images/hdri/newport_loft.hdr").to_vec()],
            TextureFormat::Rgba32Float,
            SkyboxBuilderOptions::default(),
        );
    }

    fn create_camera(&mut self) {
        let current_scene = self.engine.world.current_scene_mut();
        let camera_node_id = current_scene.create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        camera.fov = std::f32::consts::FRAC_PI_4;
        current_scene.add_component(&camera_node_id, camera);
        {
            current_scene.get_node_mut_forcely(&camera_node_id)
                .transform
                .set_position(Vec3::new(0.0, 1.5, 1.5)); // front
        }

        current_scene.add(camera_node_id);
        let camera_controller =
            CameraController::new(camera_node_id, CameraTarget::Position(Vec3::ZERO));
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
