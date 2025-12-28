use imagic::prelude::*;

struct ChangeMipLevelBehavior {
    node: NodeHandle,
    cur_mip_level: f32,
    cur_time: f32,
    change_threshold: f32,
    max_lod: f32,
}

impl ChangeMipLevelBehavior {
    pub fn new(node: NodeHandle) -> Self {
        Self {
            node,
            cur_mip_level: 0.0,
            cur_time: 0.0,
            change_threshold: 1.0,
            max_lod: 10.0,
        }
    }
}

impl Behavior for ChangeMipLevelBehavior {
    impl_as_any!();
    fn on_update(&mut self, logic_context: &mut LogicContext) {
        self.cur_time += logic_context.time.delta();
        if self.cur_time >= self.change_threshold {
            self.cur_time = 0.0;
            self.cur_mip_level = self.cur_mip_level + 1.0;
            if self.cur_mip_level > self.max_lod {
                self.cur_mip_level = 0.0;
            }

            if let Some(renderer) = logic_context.world.current_scene_mut().get_component_mut::<MeshRenderer>(&self.node) {
                let material_mut_ref = logic_context.material_manager.get_material_mut_forcely(&renderer.materials[0]);
                material_mut_ref.set_float("lod", self.cur_mip_level);
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("2d mipmap demo.");
    let engine_options = EngineOptions {
        window_size: WindowSize::new(500.0, 500.0),
        app_name: "lxy 2d mipmap demo",
    };
    let mut engine = Engine::new(engine_options);
    let color_texture_handle = engine.texture_sampler_manager.create_texture_from_image(
        vec![include_bytes!("./assets/images/lena.png").to_vec()],
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        true,
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

    let shader_handle = engine.shader_manager.create_shader(
        include_str!("shaders/show_mipmaps_2d.wgsl"),
        "custom/show_mipmaps_2d".to_owned(),
    );
    let quad_node = engine.world.current_scene_mut().create_node("quad");
    let mesh: Mesh = Quad::default().into();
    let mesh = RR_new!(mesh);

    let show_mipmap_material_handle = engine.material_manager.create_material(shader_handle, &mut engine.shader_manager);
    
    let show_mipmap_material_mut_ref = engine.material_manager.get_material_mut_forcely(&show_mipmap_material_handle);
    show_mipmap_material_mut_ref.set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
    show_mipmap_material_mut_ref.set_albedo_map(color_texture_handle);
    show_mipmap_material_mut_ref.set_albedo_map_sampler(color_map_sampler);
    show_mipmap_material_mut_ref.set_float("lod", 0.0);
    
    let mesh_renderer = MeshRenderer::new(mesh, vec![show_mipmap_material_handle]);
    engine.world.current_scene_mut().add_component(&quad_node, mesh_renderer);

    let camera_node = engine.world.current_scene_mut().create_node("Main Camera");
    let mut camera = Camera::default();
    camera.clear_color = Some(Color::scalar(0.3));
    engine.world.current_scene_mut().add_component(&camera_node, camera);

    engine.add_behavior(ChangeMipLevelBehavior::new(quad_node.clone()));
    let scene = engine.world.current_scene_mut();
    scene.add(quad_node);
    scene.add(camera_node);

    engine.run();
}
