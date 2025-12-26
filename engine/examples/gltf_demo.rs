use imagic::prelude::*;

pub struct GameBehavior {
    material: RR<Material>,
    metallic_roughness_ao: Vec4,
}

impl GameBehavior {
    pub fn new(material: RR<Material>) -> Self {
        let metallic_roughness_ao = if let Some(mra) = material.borrow().get_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO) {
            mra
        } else {
            Vec4::new(1.0, 1.0, 1.0, 1.0)
        };
        Self {
            material,
            metallic_roughness_ao,
        }
    }
}

impl Behavior for GameBehavior {
    impl_as_any!();
    fn on_gui(&mut self, logic_context: &mut LogicContext, ui_context: &egui::Context) {
        egui::Window::new("gltf demo settings").show(ui_context, |ui| {
            ui.label(format!("FPS:{}", logic_context.performance_tracker.fps_counter.fps));
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

fn create_camera(engine: &mut Engine) {
    let current_scene = engine.world.current_scene_mut();
    let camera_node = current_scene.create_node("Main Camera");
    let mut camera = Camera::default();
    camera.clear_color = Some(Color::scalar(0.3));
    camera.fov = std::f32::consts::FRAC_PI_4;
    current_scene.get_node_mut_forcely(&camera_node).transform
        .set_position(Vec3::new(0.0, 1.5, 5.5)); // front
    current_scene.add_component(&camera_node, camera);

    current_scene.add(camera_node);
    let camera_controller =
        CameraController::new(camera_node, CameraTarget::Position(Vec3::ZERO));
    engine.add_behavior(camera_controller);
}

fn load_model(engine: &mut Engine) {
    // Example usage of the ModelLoader
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // 构建目标文件的完整路径
    let model_path = std::path::Path::new(manifest_dir)
        .join("examples")
        .join("assets")
        .join("models")
        .join("DamagedHelmet.glb");
        // .join("monkey.glb");
    let model_loader = ModelLoader::new();
    match model_loader.load(engine, model_path.to_str().unwrap()) {
        Ok(node) => {
            log::info!("Model loaded successfully: {:?}", engine.world.current_scene().get_node_forcely(&node).name);
            if let Some(material) = get_material_in_children(engine.world.current_scene(), &node) {
                let game_behavior = GameBehavior::new(material);
                engine.add_behavior(game_behavior);
            } else {
                log::warn!("Model has no material.");
            }
        }
        Err(e) => log::error!("{}", e),
    }
}

fn get_material_in_children(scene: & Scene, node: &NodeHandle) -> ORR<Material> {
    if let Some(mesh_renderer) = scene.get_component::<MeshRenderer>(node) {
        return Some(mesh_renderer.materials[0].clone());
    } else if let Some(node) = &scene.get_node(&node) && let Some(children) = &node.children {
        for child in children {
            return get_material_in_children(scene, child);
        }
    }
    None
}

fn add_skybox(engine: &mut Engine) {
    let _skybox_node = SkyboxBuilder::create_skybox(
        engine,
        // vec![include_bytes!("./assets/images/hdri/resting_place_2_1k.hdr").to_vec()],
        vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
        // vec![include_bytes!("./assets/images/hdri/newport_loft.hdr").to_vec()],
        TextureFormat::Rgba32Float,
        SkyboxBuilderOptions::default(),
    );
}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let options = EngineOptions {
        window_size: WindowSize::new(800.0, 500.0),
        app_name: "lxy gltf demo",
    };
    let mut engine = Engine::new(options);
    load_model(&mut engine);
    create_camera(&mut engine);
    add_skybox(&mut engine);
    
    engine.run();
}