use imagic::prelude::*;

struct Game {
    engine: Box<Engine>,
    // material: RR<Material>,
    // primitive_ids: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum MaterialType {
    ShowUv,
    Unlit,
    Pbr,
}

impl MaterialType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MaterialType::ShowUv => "Show UV",
            MaterialType::Unlit => "Unlit",
            MaterialType::Pbr => "PBR",
        }
    }
}

struct GameBehavior {
    materials: Vec<MaterialHandle>,
    is_wireframe_mode: bool,
    material_type: MaterialType,
    primitive_ids: Vec<NodeHandle>,
}

impl GameBehavior {
    pub fn new(materials: Vec<MaterialHandle>, primitive_ids: Vec<NodeHandle>, material_type: MaterialType) -> Self {
        Self {
            materials,
            is_wireframe_mode: false,
            material_type,
            primitive_ids,
        }
    }

    pub fn change_material(&mut self, logic_context: &mut LogicContext) {
        let material_handle = &self.materials[self.material_type as usize];
        let material = logic_context.material_manager.get_material_mut_forcely(material_handle);
        if self.is_wireframe_mode {
            material.render_state.polygon_mode = PolygonMode::Line;
        } else {
            material.render_state.polygon_mode = PolygonMode::Fill;
        }
        material.mark_dirty();
        self.primitive_ids.iter().for_each(|node_id| {
            let mesh_renderer = logic_context.world.current_scene_mut().get_component_mut::<MeshRenderer>(node_id).unwrap();
            mesh_renderer.materials[0] = *material_handle;
        });
    }
}

impl Behavior for GameBehavior {
    impl_as_any!();
    fn on_gui(&mut self, logic_context: &mut LogicContext, ctx: &egui::Context) {
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
                let material_mut_ref = logic_context.material_manager.get_material_mut_forcely(&self.materials[self.material_type as usize]);
                if self.is_wireframe_mode {
                    log::info!("Switch to Wireframe Mode");
                    material_mut_ref.render_state.polygon_mode = PolygonMode::Line;
                } else {
                    log::info!("Switch to Fill Mode");
                    material_mut_ref.render_state.polygon_mode = PolygonMode::Fill;
                }
                material_mut_ref.mark_dirty();
            }
            let old_material_type = self.material_type;
            let _response = egui::ComboBox::from_label("Material Type")
                .selected_text(self.material_type.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.material_type, MaterialType::ShowUv, "Show UV");
                    ui.selectable_value(&mut self.material_type, MaterialType::Unlit, "Unlit");
                    ui.selectable_value(&mut self.material_type, MaterialType::Pbr, "PBR");
                });
            if self.material_type != old_material_type {
                // 下拉列表发生了变化
                println!("Material Type changed to {:?}", self.material_type);
                self.change_material(logic_context);
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
        let engine = Engine::new(engine_options);
                
        Self {
            engine,
        }
    }

    fn create_material(engine: &mut Engine) -> Vec<MaterialHandle> {
        let mut materials = Vec::new();
        {
            let shader_show_uv =
                engine.shader_manager.create_shader(include_str!("shaders/show_uv.wgsl"), "show_uv".to_owned());
            let material_show_uv = engine.material_manager.create_material(shader_show_uv, &mut engine.shader_manager);
            // material_show_uv.borrow_mut().render_state.cull_mode = CullMode::None;
            materials.push(material_show_uv);
        }
        {
            let albedo_map = engine.texture_sampler_manager.create_texture_from_image(
                vec![include_bytes!("./assets/images/lena.png").to_vec()],
                TextureDimension::D2,
                TextureFormat::Rgba8UnormSrgb,
                false,
                true,
            );
            
            let (_, shader_handle) = engine.shader_manager.get_builtin_unlit_shader();
            let unlit_material_handle = engine.material_manager.create_material(*shader_handle, &mut engine.shader_manager);
            let unlit_material = engine.material_manager.get_material_mut_forcely(&unlit_material_handle);
            unlit_material.set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
            unlit_material.set_albedo_map(albedo_map);
            materials.push(unlit_material_handle);
        }
        {
            let (_, shader_handle) = engine.shader_manager.get_builtin_pbr_shader();
            let pbr_material_handle = engine.material_manager.create_material(*shader_handle, &mut engine.shader_manager);
            {
                let pbr_material_mut_ref = engine.material_manager.get_material_mut_forcely(&pbr_material_handle);
                pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_ALBEDO_COLOR, Vec4::new(1.0, 1.0, 1.0, 1.0));
                pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO, Vec4::new(0.0, 1.0, 1.0, 1.0));
            }
            materials.push(pbr_material_handle);
        }
        materials
    }

    fn init(&mut self) {
        let _skybox_node = SkyboxBuilder::create_skybox(
            &mut self.engine,
            vec![include_bytes!("./assets/images/hdri/spruit_sunrise_2k.hdr").to_vec()],
            TextureFormat::Rgba32Float,
            SkyboxBuilderOptions::default(),
        );

        let material_type = MaterialType::Pbr;
        let materials = Self::create_material(&mut self.engine);

        let cur_material_handle = materials[material_type as usize];

        let _camera_node = self.create_camera();
        let quad_node = self.create_quad(cur_material_handle);
        let cuboid_node = self.create_cuboid(cur_material_handle);
        let sphere_node = self.create_sphere(cur_material_handle);
        let mut primitive_ids = Vec::new();
        primitive_ids.push(quad_node);
        primitive_ids.push(cuboid_node);
        primitive_ids.push(sphere_node);

        
        self.engine.add_behavior(GameBehavior::new(
            materials,
            primitive_ids,
            material_type,
        ));

    }

    pub fn run(&mut self) {
        self.init();
        self.engine.run();
    }

    fn create_camera(&mut self) -> NodeHandle {
        let current_scene = self.engine.world.current_scene_mut();
        let camera_node = current_scene.create_node("Main Camera");
        let mut camera = Camera::default();
        // let mut camera = Camera::new_orthogonal(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        camera.clear_color = Some(Color::scalar(0.3));
        current_scene.get_node_mut_forcely(&camera_node).transform
            .set_position(Vec3::new(0.0, 8.0, 12.0)); // front
        current_scene.add_component(&camera_node, camera);

        current_scene.add(camera_node);
        let camera_controller =
            CameraController::new(camera_node, CameraTarget::Position(Vec3::ZERO));
        self.engine.add_behavior(camera_controller);
        camera_node
    }

    fn create_quad(&mut self, cur_material: MaterialHandle) -> NodeHandle {
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
            let mesh_handle = self.engine.mesh_manager.add_mesh(mesh);
            let mesh_renderer = MeshRenderer::new(mesh_handle, vec![cur_material]);
            scene.add_component(&quad_node, mesh_renderer);
        }
        scene.add(quad_node);
        quad_node
    }

    fn create_cuboid(&mut self, cur_material: MaterialHandle) -> NodeHandle {
        let scene = self.engine.world.current_scene_mut();
        let cuboid_node =
            scene.create_node("Cuboid");
        {
            let mesh: Mesh = Cuboid::default().into();
            // let mesh: Mesh = Cuboid::new(2.0, 1.0, 1.0, 2, 1, 1).into();
            let mesh_handle = self.engine.mesh_manager.add_mesh(mesh);
            let mesh_renderer = MeshRenderer::new(mesh_handle, vec![cur_material]);
            scene.add_component(&cuboid_node, mesh_renderer);
        }
        scene.add(cuboid_node);
        cuboid_node
    }

    fn create_sphere(&mut self, cur_material: MaterialHandle) -> NodeHandle {
        let scene = self.engine.world.current_scene_mut();
        let uv_sphere_node =
            scene.create_node("UVSphere");
        {
            scene.get_node_mut_forcely(&uv_sphere_node).transform.set_position_x(1.5);
            let mesh: Mesh = UVSphere::default().into();
            let mesh_handle = self.engine.mesh_manager.add_mesh(mesh);
            let mesh_renderer = MeshRenderer::new(mesh_handle, vec![cur_material]);
            scene.add_component(&uv_sphere_node, mesh_renderer);
        }
        scene.add(uv_sphere_node);
        uv_sphere_node
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut game = Game::new();
    game.run();
}
