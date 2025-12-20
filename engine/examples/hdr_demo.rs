use imagic::prelude::*;
use log::info;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("hdr demo.");
    let engine_options = EngineOptions {
        window_size: WindowSize::new(1000.0, 500.0),
        app_name: "lxy hdr demo",
    };
    let mut engine = Engine::new(engine_options);
    let world = &mut engine.world;

    let color_texture_handle = engine.texture_sampler_manager.create_texture_from_image(
        vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
        TextureDimension::D2,
        TextureFormat::Rgba32Float,
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
    let quad_node = world.current_scene_mut().create_node("quad");
    {
        world.current_scene_mut().get_node_mut_forcely(&quad_node).transform.scale.x = 2.0;
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
        world.current_scene_mut().add_component(&quad_node, mesh_renderer);
    }

    let camera_node = world.current_scene_mut().create_node("Main Camera");
    let mut camera = Camera::default();
    camera.clear_color = Some(Color::scalar(0.3));
    world.current_scene_mut().add_component(&camera_node, camera);
    world.current_scene_mut().get_node_mut_forcely(&camera_node)
        .transform
        .set_position(Vec3::new(0.0, 0.0, 2.5));

    let scene = world.current_scene_mut();
    scene.add(quad_node);
    scene.add(camera_node);

    engine.run();
}
