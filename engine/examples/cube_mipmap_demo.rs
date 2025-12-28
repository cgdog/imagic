use imagic::prelude::*;

struct Game {
    #[allow(unused)]
    engine: Box<Engine>,
}

struct SkyboxBehavior {
    skybox_material: MaterialHandle,
    skybox_node: NodeHandle,
    lod: f32,
    background_cube_map: TextureHandle,
    reflection_cube_map: TextureHandle,
    show_background_cube_map: bool,
}

impl Behavior for SkyboxBehavior {
    impl_as_any!();

    fn on_start(&mut self, logic_context: &mut LogicContext) {
        if let Some(mesh_renderer) = logic_context.world.current_scene_mut().get_component_mut::<MeshRenderer>(&self.skybox_node) {
            mesh_renderer.materials[0] = self.skybox_material;
        }

        if let Some(skybox_component) = logic_context.world.current_scene().get_component::<Skybox>(&self.skybox_node) {
            self.background_cube_map = skybox_component.background_cube_map;
            self.reflection_cube_map = skybox_component.reflection_cube_map;
            let material_mut_ref = logic_context.material_manager.get_material_mut_forcely(&self.skybox_material);
            material_mut_ref.set_texture("skybox_cube_texture", self.background_cube_map);
            material_mut_ref.render_state.cull_mode = CullMode::Front;
            material_mut_ref.set_float("lod", self.lod);
            material_mut_ref.mark_dirty();
        }
    }

    fn on_gui(&mut self, logic_context: &mut LogicContext, ctx: &egui::Context) {
        egui::Window::new("Skybox LOD Control").show(ctx, |ui| {
            if ui.add(
                egui::Slider::new(&mut self.lod, 0.0..=7.0)
                    .text("Skybox LOD")
                    .step_by(1.0),
            ).changed() {
                logic_context.material_manager.get_material_mut_forcely(&self.skybox_material).set_float("lod", self.lod);
            }
            if ui.add(egui::RadioButton::new(self.show_background_cube_map, "Show background map")).clicked() {
                self.show_background_cube_map = true;
                let material_mut = logic_context.material_manager.get_material_mut_forcely(&self.skybox_material);
                material_mut.set_texture("skybox_cube_texture", self.background_cube_map);
                material_mut.mark_dirty();
                log::warn!("Switched to background cube map");
            }
            if ui.add(egui::RadioButton::new(!self.show_background_cube_map, "Show reflection map")).clicked() {
                self.show_background_cube_map = false;
                let material_mut = logic_context.material_manager.get_material_mut_forcely(&self.skybox_material);
                material_mut.set_texture("skybox_cube_texture", self.reflection_cube_map);
                material_mut.mark_dirty();
                log::warn!("Switched to reflection cube map");
            }
        });
    }
}

impl Game {
    pub fn new() -> Self {
        let engine_options = EngineOptions {
            window_size: WindowSize::new(800.0, 500.0),
            app_name: "lxy cube_mipmap demo",
        };
        let engine = Engine::new(engine_options);
        Self { engine }
    }

    fn init(&mut self) {
        self.create_camera();
        let skybox_node = SkyboxBuilder::create_skybox(
            &mut self.engine,
            // vec![include_bytes!("./assets/images/hdri/resting_place_2_1k.hdr").to_vec()],
            // vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
            vec![include_bytes!("./assets/images/hdri/newport_loft.hdr").to_vec()],
            TextureFormat::Rgba32Float,
            SkyboxBuilderOptions::default(),
        );

        let custom_skybox_shader =
            self.engine.shader_manager.create_shader(include_str!("./shaders/custom_skybox.wgsl"), "custom_skybox".to_owned());
        let skybox_material = self.engine.material_manager.create_material(custom_skybox_shader, &mut self.engine.shader_manager);
        let skybox_behavior = SkyboxBehavior {
            skybox_material: skybox_material,
            skybox_node,
            lod: 0.0,
            background_cube_map: TextureHandle::INVALID,
            reflection_cube_map: TextureHandle::INVALID,
            show_background_cube_map: true,
        };
        self.engine.add_behavior(skybox_behavior);
    }

    fn create_camera(&mut self) {
        let current_scene = self.engine.world.current_scene_mut();
        let camera_node = current_scene.create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        camera.fov = std::f32::consts::FRAC_PI_4;
        current_scene.add_component(&camera_node, camera);
        {
            current_scene.get_node_mut_forcely(&camera_node)
                .transform
                .set_position(Vec3::new(0.0, 0.0, 1.5)); // front

        }

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
