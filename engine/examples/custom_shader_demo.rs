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
}

impl Behavior for CustomShaderBehavior {
    impl_as_any!();

    fn on_start(&mut self, _logic_context: &mut LogicContext) {
        log::info!("custom shader behavior start");
    }

    fn on_update(&mut self, logic_context: &mut LogicContext) {

        self.cur_time += logic_context.time.delta();

        // info!("custom shader total time: {}, delta time: {}", _time.elapsed(), _time.delta());
        if let Some(mesh_renderer) = logic_context.world.current_scene_mut().get_component_mut::<MeshRenderer>(&self.quad_node)
        {
            let material = &mut mesh_renderer.materials[0];
            if self.cur_time >= self.max_time {
                self.cur_time = 0.0;
                self.cur_color_index = (self.cur_color_index + 1) % self.colors.len();
            }
            let ratio = self.cur_time / self.max_time;
            let next_color_index = (self.cur_color_index + 1) % self.colors.len();
            let color = self.colors[self.cur_color_index].mix(&self.colors[next_color_index], ratio);
            // material.borrow_mut().set_color("color", color);
            material.borrow_mut().set_struct("color", bytemuck::bytes_of(&ColorInfo {
                input_color: color,
                final_filter: Color::WHITE, // you can change this to other color to see the effect.
            }).to_vec());
        }
    }

    fn on_destroy(&mut self, _: &mut LogicContext) {
        log::info!("custom shader behavior destroied");
    }
}

impl CustomShaderBehavior {
    pub fn new(node: NodeId) -> Self {
        Self {
            quad_node: node,
            colors: vec![Color::BLUE, Color::RED, Color::PURPLE, Color::new(1.0, 0.0784, 0.5765, 1.0)],
            cur_time: 0.0,
            max_time: 2.5,
            cur_color_index: 0,
        }
    }
}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("hi, custom shader");
    let engine_options = EngineOptions {
        window_size: WindowSize::new(500.0, 500.0),
        app_name: "lxy custom shader demo",
    };
    let mut engine = Engine::new(engine_options);
    let shader = Shader::new(
        include_str!("shaders/circle.wgsl"),
        "custom/circle".to_owned(),
    );

    let material = Material::new(shader);
    {
        let quad_node = engine.world.current_scene_mut().create_node("quad");
        engine.world.current_scene_mut().get_node_mut_forcely(&quad_node)
            .transform
            .set_uniform_scale(2.0);
        let mesh: Mesh = Quad::default().into();
        let mesh = RR_new!(mesh);
        let mesh_renderer = MeshRenderer::new(mesh, vec![material]);
        engine.world.current_scene_mut().add_component(&quad_node, mesh_renderer);
        
        let custom_behavior = CustomShaderBehavior::new(quad_node);
        engine.add_behavior(custom_behavior);

        let camera_node = engine.world.current_scene_mut().create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        
        let scene = engine.world.current_scene_mut();
        scene.add_component(&camera_node, camera);
        scene.add(quad_node);
        scene.add(camera_node);
    }
    
    engine.run();
}
