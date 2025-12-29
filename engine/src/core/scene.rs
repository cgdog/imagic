use crate::{
    assets::{
        MaterialManager, ShaderManager, Texture, TextureHandle, TextureSamplerManager, environment::{ibldata::IBLData, skybox::Skybox}
    },
    components::mesh_renderer::MeshRenderer,
    core::{NodeArena, NodeHandle},
    graphics::graphics_context::GraphicsContext,
    math::{Mat4, color::Color},
    prelude::{Camera, Component, component_storage::ComponentStorages},
    time::Time,
};

use super::node::Node;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SH {
    sh: [Color; 9],
}

/// A scene contains nodes and components, and manages their lifecycle.
pub struct Scene {
    /// Root nodes which have no parent.
    pub root_nodes: Vec<NodeHandle>,
    /// The arena for node allocation.
    pub(crate) node_arena: NodeArena,
    /// The clear color of the scene.
    pub clear_color: Color,
    /// Whether the scene has fog enabled.
    pub fog_enabled: bool,
    /// The color of the fog.
    pub fog_color: Color,
    /// The IBL data of the scene.
    pub ibl_data: Option<IBLData>,
    /// Cached camera nodes in the scene used to render the scene.
    pub(crate) cached_cameras: Vec<NodeHandle>,
    /// Cached renderable nodes in the scene used to render the scene.
    pub(crate) cached_renderables: Vec<NodeHandle>,
    /// Cached skybox node in the scene used to render the scene.
    pub(crate) cached_skybox_: NodeHandle,
    /// The SH coefficients of the scene.
    pub(crate) sh: SH,

    /// Component storages for the scene.
    pub(crate) component_storages: ComponentStorages,
}

impl Scene {
    /// Create a new node.
    pub fn new() -> Self {
        let scene = Scene {
            root_nodes: vec![],
            node_arena: NodeArena::new(),
            clear_color: Color::new(0.0, 0.0, 0.0, 1.0),
            fog_enabled: false,
            fog_color: Color::new(0.0, 0.0, 0.0, 1.0),
            ibl_data: None,
            cached_cameras: vec![],
            cached_renderables: vec![],
            cached_skybox_: NodeHandle::INVALID,
            sh: Default::default(),
            component_storages: ComponentStorages::new(),
        };
        scene
    }

    /// Add a node into the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The node to add.
    pub fn add(&mut self, node_id: NodeHandle) {
        self.root_nodes.push(node_id);
    }

    pub fn get_node(&self, node_id: &NodeHandle) -> Option<&Node> {
        self.node_arena.get(node_id)
    }

    pub fn get_node_forcely(&self, node_id: &NodeHandle) -> &Node {
        self.node_arena.get_forcely(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &NodeHandle) -> Option<&mut Node> {
        self.node_arena.get_mut(node_id)
    }

    pub fn get_node_mut_forcely(&mut self, node_id: &NodeHandle) -> &mut Node {
        self.node_arena.get_mut_forcely(node_id)
    }

    /// Add a node into the scene by attach it to the give parent node which is in the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to attach.
    /// * `parent_id` - The ID of the parent node.
    /// 
    /// # Returns
    /// 
    /// * `bool` - Returns `true` if the attachment was successful, `false` otherwise.
    pub fn attach_to_parent(&mut self, node_id: &NodeHandle, parent_id: NodeHandle) -> bool {
        self.node_arena.attach_to_parent(parent_id, node_id)
    }

    /// Detach a node from its parent in the scene.
    pub fn detach_from_parent(&mut self, node_id: &NodeHandle) -> bool {
        self.node_arena.detach_from_parent(node_id)
    }

    /// Create a new node with the given name.
    /// # Arguments
    /// 
    /// * `name` - The name of the node.
    /// 
    /// # Returns
    /// 
    /// * `NodeHandle` - The ID of the created node.
    pub fn create_node(&mut self, name: impl Into<String>) -> NodeHandle {
        self.node_arena.create_node(name)
    }

    /// Destroy a node in the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to destroy.
    pub fn destroy_node(&mut self, node_id: &NodeHandle) {
        if let Some(node) = self.node_arena.get_mut(node_id) {
            for (component_type_id, component_id) in &node.components {
                self.component_storages.remove_component_internal(component_id, component_type_id);
            }
            self.node_arena.destroy_node(node_id);        
        }
    }

    /// Add a component to a node in the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to add the component to.
    /// * `component` - The component to add.
    /// 
    /// # Returns
    /// 
    /// * `Option<T>` - Returns the old component if it exists, `None` otherwise.
    pub fn add_component<T: Component>(&mut self, node_id: &NodeHandle, component: T) -> Option<T>  {
        if let Some(node) = self.node_arena.get_mut(node_id) {
            let component_type_id = std::any::TypeId::of::<T>();
            if let Some(old_component_id) = node.components.get(&component_type_id) {
                #[cfg(debug_assertions)]
                {
                    log::warn!("Try to add more than one component of type {:?} to node {:?}, the old one is replaced and returned.", component_type_id, node_id);
                }
                self.component_storages.replace_component(old_component_id, component_type_id, component)
            } else {
                let component_id = self.component_storages.add_component(component, component_type_id);
                node.components.insert(component_type_id, component_id);
                if component_type_id == std::any::TypeId::of::<Camera>() {
                    self.cached_cameras.push(*node_id);
                } else if component_type_id == std::any::TypeId::of::<MeshRenderer>() {
                    self.cached_renderables.push(*node_id);
                } else if  component_type_id == std::any::TypeId::of::<Skybox>() {
                    self.cached_skybox_ = *node_id;
                }
                None
            }
        } else {
            #[cfg(debug_assertions)]
            {
                log::warn!("Invalid node id {} when add component", node_id);
            }
            None
        }
    }

    /// Remove a component from a node in the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to remove the component from.
    /// 
    /// # Returns
    /// 
    /// * `Option<T>` - Returns the removed component if it exists, `None` otherwise.
    pub fn remove_component<T: Component>(&mut self, node_id: &NodeHandle) -> Option<T> {
        if let Some(node) = self.node_arena.get_mut(node_id) {
            let component_type_id = std::any::TypeId::of::<T>();
            if let Some(component_id) = node.components.remove(&component_type_id) {
                if component_type_id == std::any::TypeId::of::<Camera>() {
                    self.cached_cameras.retain(|&id| id != *node_id);
                } else if component_type_id == std::any::TypeId::of::<MeshRenderer>() {
                    self.cached_renderables.retain(|&id| id != *node_id);
                } else if  component_type_id == std::any::TypeId::of::<Skybox>() {
                    self.cached_skybox_ = NodeHandle::INVALID;
                }
                self.component_storages.remove_component(&component_id, component_type_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a reference to a component of a node in the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to get the component from.
    /// 
    /// # Returns
    /// 
    /// * `Option<&T>` - Returns a reference to the component if it exists, `None` otherwise.
    pub fn get_component<T: Component>(&self, node_id: &NodeHandle) -> Option<&T> {
        let component_type_id = std::any::TypeId::of::<T>();
        if let Some(node) = self.node_arena.get(node_id) && let Some(component_id) = node.components.get(&component_type_id) {
            self.component_storages.get_component(component_id)
        } else {
            None
        }
    }

    /// Get a mutable reference to a component of a node in the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the node to get the component from.
    /// 
    /// # Returns
    /// 
    /// * `Option<&mut T>` - Returns a mutable reference to the component if it exists, `None` otherwise.
    pub fn get_component_mut<T: Component>(&mut self, node_id: &NodeHandle) -> Option<&mut T> {
        let component_type_id = std::any::TypeId::of::<T>();
        if let Some(node) = self.node_arena.get(node_id) && let Some(component_id) = node.components.get(&component_type_id) {
            self.component_storages.get_component_mut(component_id)
        } else {
            None
        }
    }

    /// Set the IBL data of the scene.
     /// # Arguments
     /// 
     /// * `ibl_data` - The IBL data to set.
    pub fn set_ibl_data(&mut self, ibl_data: Option<IBLData>) {
        self.ibl_data = ibl_data
    }

    /// Lifecycle method called when the scene is initialized.
    pub(crate) fn on_init(&mut self, _time: &mut Time) {
    }

    /// Try to initialize the skybox if it exists.
    /// # Arguments
    /// 
    /// * `graphics_context` - The graphics context to use for initialization.
    /// * `texture_sampler_manager` - The texture sampler manager to use for initialization.
    /// * `shader_manager` - The shader manager to use for initialization.
    /// * `material_manager` - The material manager to use for initialization.
    pub(crate) fn try_init_skybox(
        &mut self,
        graphics_context: &mut GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager,
        shader_manager: &mut ShaderManager,
        material_manager: &mut MaterialManager,
    ) {
        if NodeHandle::INVALID != self.cached_skybox_ {
            let skybox_node_id = self.cached_skybox_;
            let mut need_refresh_skybox_texture = false;
            let mut skybox_texture_handle = TextureHandle::INVALID;
            if let Some(skybox_component) = self.get_component_mut::<Skybox>(&skybox_node_id)
                && skybox_component.should_init()
            {
                need_refresh_skybox_texture = true;
                skybox_component.on_init(graphics_context, texture_sampler_manager, shader_manager);
                skybox_texture_handle = skybox_component.background_cube_map;
                self.sh.sh = skybox_component.sh;
            }
            if need_refresh_skybox_texture {
                if let Some(mesh_renderer) = self.get_component_mut::<MeshRenderer>(&skybox_node_id)
                    && let Some(skybox_material_handle) = mesh_renderer.materials.first()
                    && let Some(skybox_material) = material_manager.get_material_mut(skybox_material_handle)
                {
                    skybox_material.set_texture("skybox_cube_texture", skybox_texture_handle);
                }
            }
        }
    }

    /// Get the environment reflection information of the scene.
    /// # Returns
    /// 
    /// * `(TextureHandle, TextureHandle)` - Returns the reflection cube map texture handle and the BRDF LUT texture handle.
    pub fn get_environment_reflection_info(&self) -> (TextureHandle, TextureHandle) {
        if NodeHandle::INVALID != self.cached_skybox_ {
            if let Some(skybox_component) = self.get_component::<Skybox>(&self.cached_skybox_) {
                return (
                    skybox_component.reflection_cube_map,
                    skybox_component.brdf_lut,
                );
            }
        }
        (Texture::default_cube_texture(), Texture::white())
    }

    /// Lifecycle method called when the scene is updated.
    /// # Arguments
    /// 
    /// * `time` - The time delta since the last update.
    pub(crate) fn on_update(&mut self, _time: &mut Time) {
        for root_node in &self.root_nodes {
            Self::update_node_hierarchy(&mut self.node_arena, root_node, None);
        }
    }

    fn update_node_hierarchy(node_arena: &mut NodeArena, node_id: &NodeHandle, parent_model_matrix: Option<Mat4>) {
        let mut model_matrix = None;
        let mut children = None;
        let mut node_is_valid = false;
        if let Some(node) = node_arena.get_mut(node_id) {
            node_is_valid = true;
            node.on_update(parent_model_matrix);
            model_matrix = Some(node.transform.model_matrix);
            children = node.children.clone();
        }

        if node_is_valid {
            if let Some(children) = &mut children {
                for child in children {
                    Self::update_node_hierarchy(node_arena, &child, model_matrix);
                }
            }
        }
    }

    /// Lifecycle method called when the scene is stopped.
    pub(crate) fn on_stop(&mut self, _time: &mut Time) {
    }
}