use imagic::prelude::*;

struct MoveBehavior {
    node_id: NodeId,
    cur_pos: Vec3,
    speed: f32,
    dir: f32,
    min_x: f32,
    max_x: f32,
}

impl MoveBehavior {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            cur_pos: Vec3::ZERO,
            speed: 1.0,
            dir: 1.0,
            min_x: -0.45,
            max_x: 0.45,
        }
    }
}

impl Behavior for MoveBehavior {
    impl_as_any!();
    fn on_update(&mut self, logic_context: &mut LogicContext) {
        self.cur_pos.x = self.cur_pos.x + self.speed * self.dir * logic_context.time.delta();
        if self.cur_pos.x <= self.min_x {
            self.cur_pos.x = self.min_x;
            self.dir = -self.dir;
        } else if self.cur_pos.x >= self.max_x {
            self.cur_pos.x = self.max_x;
            self.dir = -self.dir;
        }
        logic_context.world.current_scene_mut().get_node_mut_forcely(&self.node_id).transform.set_position(self.cur_pos);
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("unlit demo.");
    let engine_options = EngineOptions {
        window_size: WindowSize::new(500.0, 500.0),
        app_name: "lxy unlit demo",
    };
    let mut engine = Engine::new(engine_options);
    {
        let color_texture_handle = engine.texture_sampler_manager.create_texture_from_image(
            vec![include_bytes!("./assets/images/lena.png").to_vec()],
            TextureDimension::D2,
            TextureFormat::Rgba8UnormSrgb,
            true,
            // 没有填充 mipmaps 时, jpg 图片可能是黑色，显示了非 lod 0 的内容。
            true,
        );
        let color_map_sampler = engine.texture_sampler_manager.create_sampler(
            AddressMode::ClampToEdge,
            AddressMode::ClampToEdge,
            AddressMode::ClampToEdge,
            FilterMode::Linear,
            FilterMode::Linear,
            FilterMode::Linear,
        );

        let shader = engine.shader_manager.get_builtin_unlit_shader();
        let quad_node =engine.world.current_scene_mut().create_node("quad");
        {
            let mesh: Mesh = Quad::default().into();
            let mesh = RR_new!(mesh);

            let unlit_material = Material::new(shader);
            {
                let mut unlit_material_mut_ref = unlit_material.borrow_mut();
                unlit_material_mut_ref.set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
                unlit_material_mut_ref.set_albedo_map(color_texture_handle);
                unlit_material_mut_ref.set_albedo_map_sampler(color_map_sampler);
            }
            let mesh_renderer = MeshRenderer::new(mesh, vec![unlit_material]);
            // quad_node_mut_ref.add_component(mesh_renderer);
            engine.world.current_scene_mut().add_component(&quad_node, mesh_renderer);
        }

        let camera_node = engine.world.current_scene_mut().create_node("Main Camera");
        let mut camera = Camera::default();
        camera.clear_color = Some(Color::scalar(0.3));
        {
            engine.world.current_scene_mut().add_component(&camera_node, camera);
            engine.world.current_scene_mut().get_node_mut_forcely(&camera_node)
                .transform
                .set_position(Vec3::new(0.0, 0.0, 2.5));
        }

        engine.add_behavior(MoveBehavior::new(quad_node));
        let scene = engine.world.current_scene_mut();
        scene.add(quad_node);
        scene.add(camera_node);
    }

    engine.run();
}
