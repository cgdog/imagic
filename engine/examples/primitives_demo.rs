use imagic::prelude::*;

struct Game {
    engine: Box<Engine>,
    material: RR<Material>,
}

struct GameBehavior {
    material: RR<Material>,
    is_wireframe_mode: bool,
}

impl GameBehavior {
    pub fn new(material: RR<Material>) -> Self {
        Self {
            material,
            is_wireframe_mode: false,
        }
    }
}

impl Behavior for GameBehavior {
    impl_as_any!();
    fn on_gui(&mut self, _engine: &mut LogicContext, ctx: &egui::Context) {
        egui::Window::new("Settings")
        // .resizable(true)
        // .vscroll(true)
        .default_open(false)
        // .default_size([100.0, 10.0])
        .default_pos([1.0, 1.0])
        .show(ctx, |ui| {
            if ui
                .checkbox(&mut self.is_wireframe_mode, "Wireframe Mode")
                .changed()
            {
                if self.is_wireframe_mode {
                    log::info!("Switch to Wireframe Mode");
                    self.material.borrow_mut().render_state.polygon_mode = PolygonMode::Line;
                    self.material.borrow_mut().mark_dirty();
                } else {
                    log::info!("Switch to Fill Mode");
                    self.material.borrow_mut().render_state.polygon_mode = PolygonMode::Fill;
                    self.material.borrow_mut().mark_dirty();
                }
            }
        });
    }
}

impl Game {
    pub fn new() -> Self {
        let engine_options = EngineOptions {
            window_size: WindowSize::new(500.0, 500.0),
            app_name: "lxy primitives demo",
        };
        let mut engine = Engine::new(engine_options);
        let material = Self::create_material(&mut engine);
        engine.add_behavior(GameBehavior::new(
            material.clone(),
        ));

        Self {
            engine,
            material,
        }
    }

    fn create_material(engine: &mut Engine) -> RR<Material> {
        const IS_SHOW_UV: bool = true;
        if IS_SHOW_UV {
            let shader_show_uv =
                Shader::new(include_str!("shaders/show_uv.wgsl"), "show_uv".to_owned());
            let material_show_uv = Material::new(shader_show_uv);
            // material_show_uv.borrow_mut().render_state.cull_mode = CullMode::None;
            material_show_uv
        } else {
            let albedo_map = engine.texture_sampler_manager.create_texture_from_image(
                vec![include_bytes!("./assets/images/lena.png").to_vec()],
                TextureDimension::D2,
                TextureFormat::Rgba8UnormSrgb,
                false,
                true,
            );
            
            let shader = engine.shader_manager.get_builtin_unlit_shader();
            let unlit_material = Material::new(shader);
            unlit_material
                .borrow_mut()
                .set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
            unlit_material.borrow_mut().set_albedo_map(albedo_map);
            unlit_material
        }
    }

    fn init(&mut self) {
        self.create_camera();
        self.create_quad();
        self.create_cuboid();
        self.create_sphere();
    }

    pub fn run(&mut self) {
        self.init();
        self.engine.run();
    }

    fn create_camera(&mut self) {
        let current_scene = self.engine.world.current_scene_mut();
        let camera_node = current_scene.create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        current_scene.get_node_mut_forcely(&camera_node).transform
            .set_position(Vec3::new(0.0, 8.0, 12.0)); // front
        current_scene.add_component(&camera_node, camera);

        current_scene.add(camera_node);
        let camera_controller =
            CameraController::new(camera_node, CameraTarget::Position(Vec3::ZERO));
        self.engine.add_behavior(camera_controller);
    }

    fn create_quad(&mut self) {
        let scene = self.engine.world.current_scene_mut();
        let quad_node = scene.create_node("quad");
        {
            let transform = &mut scene.get_node_mut_forcely(&quad_node).transform;
            transform.set_position_y(-0.5);
            transform.set_rotation_euler(Vec3::new(
                -90.0f32.to_radians(),
                0.0,
                0.0,
            ));
            transform.set_uniform_scale(5.0);
            let mesh: Mesh = Quad::new(2.0, 1.0, 2, 1).into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![self.material.clone()]);
            scene.add_component(&quad_node, mesh_renderer);
        }
        scene.add(quad_node);
    }

    fn create_cuboid(&mut self) {
        let scene = self.engine.world.current_scene_mut();
        let cuboid_node =
            scene.create_node("Cuboid");
        {
            let mesh: Mesh = Cuboid::default().into();
            // let mesh: Mesh = Cuboid::new(2.0, 1.0, 1.0, 2, 1, 1).into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![self.material.clone()]);
            scene.add_component(&cuboid_node, mesh_renderer);
        }
        scene.add(cuboid_node);
    }

    fn create_sphere(&mut self) {
        let scene = self.engine.world.current_scene_mut();
        let uv_sphere_node =
            scene.create_node("UVSphere");
        {
            scene.get_node_mut_forcely(&uv_sphere_node).transform.set_position_x(1.5);
            let mesh: Mesh = UVSphere::default().into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![self.material.clone()]);
            scene.add_component(&uv_sphere_node, mesh_renderer);
        }
        scene.add(uv_sphere_node);
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut game = Game::new();
    game.run();
}
