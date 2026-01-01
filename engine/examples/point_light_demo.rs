use imagic::prelude::*;

struct GameBehavior {
    skybox_node_handle: NodeHandle,
    point_light_node_handle: NodeHandle,
    enable_skybox: bool,
    enable_point_light: bool,
    point_light_intensity: f32,
    point_light_pos_x: f32,
    point_light_pos_z: f32,
}

impl Behavior for GameBehavior {
    impl_as_any!();
    fn on_gui(&mut self, logic_context: &mut LogicContext, ui_context: &egui::Context) {
        egui::Window::new("Point Light Demo").show(ui_context, |ui|{
            if ui.checkbox(&mut self.enable_skybox, "Enable Skybox").changed() {
                let node = logic_context.world.current_scene_mut().get_node_mut_forcely(&self.skybox_node_handle);
                node.enabled = self.enable_skybox;
            }
            if ui.checkbox(&mut self.enable_point_light, "Enable Point Light").changed() {
                let node = logic_context.world.current_scene_mut().get_node_mut_forcely(&self.point_light_node_handle);
                node.enabled = self.enable_point_light;
            }

            if ui.add(egui::Slider::new(&mut self.point_light_intensity, 0.0..=10.0).text("Intensity")).changed() {
                if let Some(light) = logic_context.world.current_scene_mut().get_component_mut::<Light>(&self.point_light_node_handle) {
                    light.intensity = self.point_light_intensity;
                }
            }

            if ui.add(egui::Slider::new(&mut self.point_light_pos_x, -10.0..=10.0).text("X")).changed() {
                let transform = &mut logic_context.world.current_scene_mut().get_node_mut_forcely(&self.point_light_node_handle).transform;
                transform.set_position_x(self.point_light_pos_x);
            }
            if ui.add(egui::Slider::new(&mut self.point_light_pos_z, -10.0..=10.0).text("Z")).changed() {
                let transform = &mut logic_context.world.current_scene_mut().get_node_mut_forcely(&self.point_light_node_handle).transform;
                transform.set_position_z(self.point_light_pos_z);
            }
        });
    }
}

fn create_camera(engine: &mut Engine) -> NodeHandle {
    let current_scene = engine.world.current_scene_mut();
    let camera_node_handle = current_scene.create_node("Main Camera");
    let mut camera = Camera::default();
    // let mut camera = Camera::new_orthogonal(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
    camera.clear_color = Some(Color::scalar(0.3));
    current_scene.get_node_mut_forcely(&camera_node_handle).transform
        .set_position(Vec3::new(0.0, 8.0, 12.0)); // front
    current_scene.add_component(&camera_node_handle, camera);

    current_scene.add(camera_node_handle);
    engine.add_camera_controller(camera_node_handle);
    camera_node_handle
}

fn create_quad(engine: &mut Engine, cur_material: MaterialHandle) -> NodeHandle {
    let scene = engine.world.current_scene_mut();
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
        let mesh_handle = engine.mesh_manager.add_mesh(mesh);
        let mesh_renderer = MeshRenderer::new(mesh_handle, vec![cur_material]);
        scene.add_component(&quad_node, mesh_renderer);
    }
    scene.add(quad_node);
    quad_node
}

fn create_cuboid(engine: &mut Engine, cur_material: MaterialHandle) -> NodeHandle {
    let scene = engine.world.current_scene_mut();
    let cuboid_node =
        scene.create_node("Cuboid");
    {
        let mesh: Mesh = Cuboid::default().into();
        // let mesh: Mesh = Cuboid::new(2.0, 1.0, 1.0, 2, 1, 1).into();
        let mesh_handle = engine.mesh_manager.add_mesh(mesh);
        let mesh_renderer = MeshRenderer::new(mesh_handle, vec![cur_material]);
        scene.add_component(&cuboid_node, mesh_renderer);
    }
    scene.add(cuboid_node);
    cuboid_node
}

fn create_sphere(engine: &mut Engine, cur_material: MaterialHandle) -> NodeHandle {
    let scene = engine.world.current_scene_mut();
    let uv_sphere_node =
        scene.create_node("UVSphere");
    {
        scene.get_node_mut_forcely(&uv_sphere_node).transform.set_position_x(1.5);
        let mesh: Mesh = UVSphere::default().into();
        let mesh_handle = engine.mesh_manager.add_mesh(mesh);
        let mesh_renderer = MeshRenderer::new(mesh_handle, vec![cur_material]);
        scene.add_component(&uv_sphere_node, mesh_renderer);
    }
    scene.add(uv_sphere_node);
    uv_sphere_node
}

fn create_material(engine: &mut Engine) -> MaterialHandle {
    let (_, shader_handle) = engine.shader_manager.get_builtin_pbr_shader();
    let pbr_material_handle = engine.material_manager.create_material(*shader_handle, &mut engine.shader_manager);
    {
        let pbr_material_mut_ref = engine.material_manager.get_material_mut_forcely(&pbr_material_handle);
        pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_ALBEDO_COLOR, Vec4::new(1.0, 1.0, 1.0, 1.0));
        pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO, Vec4::new(0.0, 1.0, 1.0, 1.0));
    }
    pbr_material_handle
}

fn create_light(engine: &mut Engine) -> NodeHandle {
    let scene = engine.world.current_scene_mut();
    let light_node_handle = scene.create_node("Point Light");
    {
        let light = Light::new_point_light(
            5.0,
            Color::WHITE,
            5.0,
            true,
        );
        scene.add_component(&light_node_handle, light);
    }
    scene.add(light_node_handle);
    scene.get_node_mut_forcely(&light_node_handle).transform
        .set_position(Vec3::new(0.0, 1.0, 0.0)); // front
    light_node_handle
}

fn init(engine: &mut Engine) {
    let _skybox_node = SkyboxBuilder::create_skybox(
        engine,
        vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
        TextureFormat::Rgba32Float,
        SkyboxBuilderOptions::default(),
    );

    let cur_material_handle = create_material(engine);
    let _camera_node = create_camera(engine);
    let _quad_node = create_quad(engine, cur_material_handle);
    let _cuboid_node = create_cuboid(engine, cur_material_handle);
    let _sphere_node = create_sphere(engine, cur_material_handle);

    let _point_light_node = create_light(engine);

    let game_behavior = GameBehavior {
        skybox_node_handle: _skybox_node,
        point_light_node_handle: _point_light_node,
        enable_skybox: true,
        enable_point_light: true,
        point_light_intensity: 5.0,
        point_light_pos_x: 0.0,
        point_light_pos_z: 0.0,
    };
    engine.add_behavior(game_behavior);
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Point Light Demo started.");

    let engine_options = EngineOptions {
        window_size: WindowSize::new(1000.0, 500.0),
        app_name: "lxy point light demo",
    };
    let mut engine = Engine::new(engine_options);
    init(&mut engine);
    engine.run();
    log::info!("Point Light Demo finished.");
}