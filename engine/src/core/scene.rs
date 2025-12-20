use crate::{
    assets::{
        INVALID_TEXTURE_HANDLE, Texture, TextureHandle, TextureSamplerManager,
        environment::{ibldata::IBLData, skybox::Skybox},
    },
    components::mesh_renderer::MeshRenderer,
    core::{INVALID_NODE_ID, NodeArena, NodeId},
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

pub struct Scene {
    /// Root nodes which have no parent.
    pub root_nodes: Vec<NodeId>,
    pub(crate) node_arena: NodeArena,
    pub clear_color: Color,
    pub fog_enabled: bool,
    pub fog_color: Color,
    pub ibl_data: Option<IBLData>,
    pub(crate) cached_cameras: Vec<NodeId>,
    pub(crate) cached_renderables: Vec<NodeId>,
    pub(crate) cached_skybox_: NodeId,
    pub(crate) sh: SH,

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
            cached_skybox_: INVALID_NODE_ID,
            sh: Default::default(),
            component_storages: ComponentStorages::new(),
        };
        scene
    }

    /// Add a node into the scene.
    /// # Arguments
    /// 
    /// * `node_id` - The node to add.
    pub fn add(&mut self, node_id: NodeId) {
        self.root_nodes.push(node_id);
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node> {
        self.node_arena.get(node_id)
    }

    pub fn get_node_forcely(&self, node_id: &NodeId) -> &Node {
        self.node_arena.get_forcely(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.node_arena.get_mut(node_id)
    }

    pub fn get_node_mut_forcely(&mut self, node_id: &NodeId) -> &mut Node {
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
    pub fn attach_to_parent(&mut self, node_id: &NodeId, parent_id: NodeId) -> bool {
        self.node_arena.attach_to_parent(parent_id, node_id)
    }

    pub fn detach_from_parent(&mut self, node_id: &NodeId) -> bool {
        self.node_arena.detach_from_parent(node_id)
    }

    pub fn create_node(&mut self, name: impl Into<String>) -> NodeId {
        self.node_arena.create_node(name)
    }

    pub fn destroy_node(&mut self, node_id: &NodeId) {
        if let Some(node) = self.node_arena.get_mut(node_id) {
            for (component_type_id, component_id) in &node.components {
                self.component_storages.remove_component_internal(component_id, component_type_id);
            }
            self.node_arena.destroy_node(node_id);        
        }
    }

    pub fn add_component<T: Component>(&mut self, node_id: &NodeId, component: T) -> Option<T>  {
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

    pub fn remove_component<T: Component>(&mut self, node_id: &NodeId) -> Option<T> {
        if let Some(node) = self.node_arena.get_mut(node_id) {
            let component_type_id = std::any::TypeId::of::<T>();
            if let Some(component_id) = node.components.remove(&component_type_id) {
                if component_type_id == std::any::TypeId::of::<Camera>() {
                    self.cached_cameras.retain(|&id| id != *node_id);
                } else if component_type_id == std::any::TypeId::of::<MeshRenderer>() {
                    self.cached_renderables.retain(|&id| id != *node_id);
                } else if  component_type_id == std::any::TypeId::of::<Skybox>() {
                    self.cached_skybox_ = INVALID_NODE_ID;
                }
                self.component_storages.remove_component(&component_id, component_type_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_component<T: Component>(&self, node_id: &NodeId) -> Option<&T> {
        let component_type_id = std::any::TypeId::of::<T>();
        if let Some(node) = self.node_arena.get(node_id) && let Some(component_id) = node.components.get(&component_type_id) {
            self.component_storages.get_component(component_id)
        } else {
            None
        }
    }

    pub fn get_component_mut<T: Component>(&mut self, node_id: &NodeId) -> Option<&mut T> {
        let component_type_id = std::any::TypeId::of::<T>();
        if let Some(node) = self.node_arena.get(node_id) && let Some(component_id) = node.components.get(&component_type_id) {
            self.component_storages.get_component_mut(component_id)
        } else {
            None
        }
    }

    pub fn set_ibl_data(&mut self, ibl_data: Option<IBLData>) {
        self.ibl_data = ibl_data
    }

    pub(crate) fn on_init(&mut self, _time: &mut Time) {
    }

    pub(crate) fn try_init_skybox(
        &mut self,
        graphics_context: &mut GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager,
    ) {
        if INVALID_NODE_ID != self.cached_skybox_ {
            let skybox_node_id = self.cached_skybox_;
            let mut need_refresh_skybox_texture = false;
            let mut skybox_texture_handle = INVALID_TEXTURE_HANDLE;
            if let Some(skybox_component) = self.get_component_mut::<Skybox>(&skybox_node_id)
                && skybox_component.should_init()
            {
                need_refresh_skybox_texture = true;
                skybox_component.on_init(graphics_context, texture_sampler_manager);
                skybox_texture_handle = skybox_component.background_cube_map;
                self.sh.sh = skybox_component.sh;
            }
            if need_refresh_skybox_texture {
                if let Some(mesh_renderer) = self.get_component_mut::<MeshRenderer>(&skybox_node_id)
                    && let Some(skybox_material) = mesh_renderer.materials.first()
                {
                    skybox_material
                        .borrow_mut()
                        .set_texture("skybox_cube_texture", skybox_texture_handle);
                }
            }
        }
    }

    pub fn get_environment_reflection_info(&self) -> (TextureHandle, TextureHandle) {
        if INVALID_NODE_ID != self.cached_skybox_ {
            if let Some(skybox_component) = self.get_component::<Skybox>(&self.cached_skybox_) {
                return (
                    skybox_component.reflection_cube_map,
                    skybox_component.brdf_lut,
                );
            }
        }
        (Texture::default_cube_texture(), Texture::white())
    }

    pub(crate) fn on_update(&mut self, _time: &mut Time) {
        for root_node in &self.root_nodes {
            Self::update_node_hierarchy(&mut self.node_arena, root_node, None);
        }
    }

    fn update_node_hierarchy(node_arena: &mut NodeArena, node_id: &NodeId, parent_model_matrix: Option<Mat4>) {
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

    pub(crate) fn on_stop(&mut self, _time: &mut Time) {
    }
}