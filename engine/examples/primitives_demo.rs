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
    materials: Vec<RR<Material>>,
    is_wireframe_mode: bool,
    material_type: MaterialType,
    primitive_ids: Vec<NodeId>,
}

impl GameBehavior {
    pub fn new(material: Vec<RR<Material>>, primitive_ids: Vec<NodeId>, material_type: MaterialType) -> Self {
        Self {
            materials: material,
            is_wireframe_mode: false,
            material_type,
            primitive_ids,
        }
    }

    pub fn change_material(&mut self, logic_context: &mut LogicContext) {
        let material = self.materials[self.material_type as usize].clone();
        if self.is_wireframe_mode {
            material.borrow_mut().render_state.polygon_mode = PolygonMode::Line;
        } else {
            material.borrow_mut().render_state.polygon_mode = PolygonMode::Fill;
        }
        self.primitive_ids.iter().for_each(|node_id| {
            let mesh_renderer = logic_context.world.current_scene_mut().get_component_mut::<MeshRenderer>(node_id).unwrap();
            mesh_renderer.materials[0] = material.clone();
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
                if self.is_wireframe_mode {
                    log::info!("Switch to Wireframe Mode");
                    self.materials[self.material_type as usize].borrow_mut().render_state.polygon_mode = PolygonMode::Line;
                    self.materials[self.material_type as usize].borrow_mut().mark_dirty();
                } else {
                    log::info!("Switch to Fill Mode");
                    self.materials[self.material_type as usize].borrow_mut().render_state.polygon_mode = PolygonMode::Fill;
                    self.materials[self.material_type as usize].borrow_mut().mark_dirty();
                }
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

    fn create_material(engine: &mut Engine) -> Vec<RR<Material>> {
        let mut materials = Vec::new();
        {
            let shader_show_uv =
                Shader::new(include_str!("shaders/show_uv.wgsl"), "show_uv".to_owned());
            let material_show_uv = Material::new(shader_show_uv);
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
            
            let shader = engine.shader_manager.get_builtin_unlit_shader();
            let unlit_material = Material::new(shader);
            unlit_material
                .borrow_mut()
                .set_albedo_color(Color::new(1.0, 1.0, 1.0, 1.0));
            unlit_material.borrow_mut().set_albedo_map(albedo_map);
            materials.push(unlit_material);
        }
        {
            let shader = engine.shader_manager.get_builtin_pbr_shader();
            let pbr_material = Material::new(shader);
            {
                let mut pbr_material_mut_ref = pbr_material.borrow_mut();
                pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_ALBEDO_COLOR, Vec4::new(1.0, 1.0, 1.0, 1.0));
                pbr_material_mut_ref.set_vec4f(BuiltinShaderUniformNames::_METALLIC_ROUGHNESS_AO, Vec4::new(0.0, 1.0, 1.0, 1.0));
            }
            materials.push(pbr_material);
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

        let cur_material = materials[material_type as usize].clone();

        let _camera_node = self.create_camera();
        let quad_node = self.create_quad(cur_material.clone());
        let cuboid_node = self.create_cuboid(cur_material.clone());
        let sphere_node = self.create_sphere(cur_material);
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

    fn create_camera(&mut self) -> NodeId {
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

    fn create_quad(&mut self, cur_material: RR<Material>) -> NodeId {
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
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![cur_material]);
            scene.add_component(&quad_node, mesh_renderer);
        }
        scene.add(quad_node);
        quad_node
    }

    fn create_cuboid(&mut self, cur_material: RR<Material>) -> NodeId {
        let scene = self.engine.world.current_scene_mut();
        let cuboid_node =
            scene.create_node("Cuboid");
        {
            let mesh: Mesh = Cuboid::default().into();
            // let mesh: Mesh = Cuboid::new(2.0, 1.0, 1.0, 2, 1, 1).into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![cur_material]);
            scene.add_component(&cuboid_node, mesh_renderer);
        }
        scene.add(cuboid_node);
        cuboid_node
    }

    fn create_sphere(&mut self, cur_material: RR<Material>) -> NodeId {
        let scene = self.engine.world.current_scene_mut();
        let uv_sphere_node =
            scene.create_node("UVSphere");
        {
            scene.get_node_mut_forcely(&uv_sphere_node).transform.set_position_x(1.5);
            let mesh: Mesh = UVSphere::default().into();
            let mesh = RR_new!(mesh);
            let mesh_renderer = MeshRenderer::new(mesh, vec![cur_material]);
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
