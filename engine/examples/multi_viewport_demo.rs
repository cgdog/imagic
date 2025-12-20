use imagic::prelude::*;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let engine_options = EngineOptions {
        window_size: WindowSize::new(1000.0, 500.0),
        app_name: "lxy multi_viewport demo",
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
    
    let shader = engine.shader_manager.get_builtin_unlit_shader();

    let quad_node = engine.world.current_scene_mut().create_node("quad");
    let mesh: Mesh = Quad::default().into();
    let mesh = RR_new!(mesh);

    let unlit_material = Material::new(shader);
    {
        let mut unlit_material_mut_ref = unlit_material.borrow_mut();
        unlit_material_mut_ref.set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
        unlit_material_mut_ref.set_albedo_map(color_texture_handle);
        unlit_material_mut_ref.set_albedo_map_sampler(color_map_sampler);
        // unlit_material_mut_ref.render_state.polygon_mode = imagic::graphics::render_states::PolygonMode::Line;
    }
    let mesh_renderer = MeshRenderer::new(mesh, vec![unlit_material]);
    engine.world.current_scene_mut().add_component(&quad_node, mesh_renderer);

    let camera_node = engine.world.current_scene_mut().create_node("Main Camera");
    let mut camera = Camera::default();
    camera.clear_color = Some(Color::scalar(0.3));
    // camera.clear_color = Some(Color::BLUE);
    camera.set_viewport(0.0, 0.0, 0.5, 1.0);
    engine.world.current_scene_mut().get_node_mut_forcely(&camera_node).transform
        .set_position(Vec3::new(0.0, 0.0, 1.5));
    engine.world.current_scene_mut().add_component(&camera_node, camera);

    let camera_node2 = engine.world.current_scene_mut().create_node("Second Camera");
    let mut camera2 = Camera::default();
    camera2.clear_color = None;
    // camera2.clear_color = Some(Color::RED);
    camera2.set_viewport(0.5, 0.0, 0.5, 1.0);
    engine.world.current_scene_mut().add_component(&camera_node2, camera2);
    engine.world.current_scene_mut().get_node_mut_forcely(&camera_node2).transform
        .set_position(Vec3::new(0.0, 0.0, 2.5));

    let scene = engine.world.current_scene_mut();
    scene.add(quad_node);
    scene.add(camera_node);
    scene.add(camera_node2);

    engine.run();
}
