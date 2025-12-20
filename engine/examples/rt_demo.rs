use imagic::prelude::*;

/// A struct uniform for custom shader.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorInfo {
    input_color: Color,
    final_filter: Color,
}

/// Behavior to control the color change of custom shader / material.
struct CustomShaderBehavior {
    quad_node: NodeId,
    /// Available colors.
    colors: Vec<Color>,
    cur_time: f32,
    max_time: f32,
    cur_color_index: usize,
    change_color_over_time: bool,
}

impl Behavior for CustomShaderBehavior {
    impl_as_any!();

    fn on_start(&mut self, _engine: &mut LogicContext) {
        log::info!("rt_demo behavior start");
    }

    fn on_update(&mut self, engine: &mut LogicContext) {
        if !self.change_color_over_time {
            return;
        }
        self.cur_time += engine.time.delta();

        // info!("custom shader total time: {}, delta time: {}", _time.elapsed(), _time.delta());
        if let Some(mesh_renderer) = engine.world.current_scene_mut().get_component_mut::<MeshRenderer>(&self.quad_node)
        {
            let material = &mut mesh_renderer.materials[0];
            if self.cur_time >= self.max_time {
                self.cur_time = 0.0;
                self.cur_color_index = (self.cur_color_index + 1) % self.colors.len();
            }
            let ratio = self.cur_time / self.max_time;
            let next_color_index = (self.cur_color_index + 1) % self.colors.len();
            let color =
                self.colors[self.cur_color_index].mix(&self.colors[next_color_index], ratio);
            // material.borrow_mut().set_color("color", color);
            material.borrow_mut().set_struct(
                "color",
                bytemuck::bytes_of(&ColorInfo {
                    input_color: color,
                    final_filter: Color::WHITE, // you can change this to other color to see the effect.
                })
                .to_vec(),
            );
        }
    }

    fn on_destroy(&mut self, _engine: &mut LogicContext) {
        log::info!("rt demo behavior destroied");
    }

    fn on_gui(&mut self, _engine: &mut LogicContext, ctx: &egui::Context) {
        egui::Window::new("Imagic - rt_demo")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .default_size([100.0, 10.0])
            .default_pos([1.0, 1.0])
            .show(&ctx, |ui| {
                ui.checkbox(
                    &mut self.change_color_over_time,
                    "change circle color over time",
                );
            });
    }
}

impl CustomShaderBehavior {
    pub fn new(quad_node: NodeId) -> Self {
        Self {
            quad_node,
            colors: vec![
                Color::BLUE,
                Color::RED,
                Color::PURPLE,
                Color::new(1.0, 0.0784, 0.5765, 1.0),
            ],
            cur_time: 0.0,
            max_time: 2.5,
            cur_color_index: 0,
            change_color_over_time: true,
        }
    }
}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("hi, rt_demo");
    let engine_options = EngineOptions {
        window_size: WindowSize::new(500.0, 500.0),
        app_name: "lxy rt demo",
    };
    let mut engine = Engine::new(engine_options);
    let shader = Shader::new(
        include_str!("shaders/circle.wgsl"),
        "custom/circle".to_owned(),
    );
    let material = Material::new(shader);
    let mesh: Mesh = Quad::default().into();
    let mesh = RR_new!(mesh);
    // material.borrow_mut().set_color("color", Color::BLUE);
    let quad_node_for_rt = engine.world.current_scene_mut().create_node("quad_for_rt");
    {
        let node = engine.world.current_scene_mut().get_node_mut_forcely(&quad_node_for_rt);
        node.layer = Layer::RenderTarget;
        node.transform.set_uniform_scale(2.0);

        let mesh_renderer = MeshRenderer::new(mesh.clone(), vec![material]);
        engine.world.current_scene_mut().add_component(&quad_node_for_rt, mesh_renderer);
    }
    let custom_behavior = CustomShaderBehavior::new(quad_node_for_rt);
    engine.add_behavior(custom_behavior);

    let rt_width = 500;
    let rt_height = 500;
    let depth_attachment = engine.texture_sampler_manager.create_attachment(
        rt_width,
        rt_height,
        1,
        TextureDimension::D2,
        1,
        TextureFormat::Depth24PlusStencil8,
    );
    let color_attachment = engine.texture_sampler_manager.create_attachment(
        rt_width,
        rt_height,
        1,
        TextureDimension::D2,
        1,
        TextureFormat::Bgra8UnormSrgb,
    );
    let color_map_sampler = engine.texture_sampler_manager.create_sampler(
        AddressMode::ClampToEdge,
        AddressMode::ClampToEdge,
        AddressMode::ClampToEdge,
        FilterMode::Linear,
        FilterMode::Linear,
        FilterMode::Linear,
    );

    let camera_node_with_rt = engine.world.current_scene_mut().create_node("Camera with RT");
    let mut camera_with_rt = Camera::default();
    camera_with_rt.visible_layers = Layer::RenderTarget.into();
    camera_with_rt.clear_color = None;
    camera_with_rt.priority = 0;
    camera_with_rt.set_depth_attachment(depth_attachment);
    camera_with_rt.set_color_attachment(color_attachment);
    engine.world.current_scene_mut().add_component(&camera_node_with_rt, camera_with_rt);
    engine.world.current_scene_mut().get_node_mut_forcely(&camera_node_with_rt).transform
        .set_position(Vec3::new(0.0, 0.0, 1.5));

    let shader = engine.shader_manager.get_builtin_unlit_shader();
    let quad_node = engine.world.current_scene_mut().create_node("quad");
    let unlit_material = Material::new(shader);
    {
        let mut unlit_material_mute_ref = unlit_material.borrow_mut();
        unlit_material_mute_ref.set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
        unlit_material_mute_ref.set_albedo_map(color_attachment);
        unlit_material_mute_ref.set_albedo_map_sampler(color_map_sampler);
    }
    let mesh_renderer = MeshRenderer::new(mesh, vec![unlit_material]);
    engine.world.current_scene_mut().add_component(&quad_node, mesh_renderer);

    let main_camera_node = engine.world.current_scene_mut().create_node("Main Camera");
    let mut main_camera = Camera::default();
    main_camera.clear_color = Some(Color::scalar(0.3));
    main_camera.priority = 1;
    engine.world.current_scene_mut().add_component(&main_camera_node, main_camera);
    {
        engine.world.current_scene_mut().get_node_mut_forcely(&main_camera_node)
            .transform
            .set_position(Vec3::new(0.0, 0.0, 1.5));
    }

    let scene = engine.world.current_scene_mut();
    scene.add(quad_node_for_rt);
    scene.add(camera_node_with_rt);

    scene.add(quad_node);
    scene.add(main_camera_node);

    engine.run();
}
