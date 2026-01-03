//! Engine utils. Add some useful shortcut methods to the [`Engine`] and [`LogicContext`].
//! 
//! # Examples
//! 
//! ```
//! use imagic::core::Engine;
//! 
//! let mut engine = Engine::new();
//! let pbr_material_handle = engine.create_pbr_material();
//! // It is equal to:
//! // Sometimes you have to use code below to avoid the compiler error.
//! let (_, pbr_shader_handle) = engine.shader_manager.get_builtin_pbr_shader();
//! let pbr_material_handle = engine.material_mangaer.create_material();
//! ```

use crate::{
    assets::{Material, MaterialHandle, ModelLoader, ModelLoaderTrait, ShaderHandle},
    core::{Engine, LogicContext, Node, NodeHandle}, math::Vec3,
    prelude::{CameraController, CameraTarget, Component, Light, Transform}
};


impl Engine {
    /// Create a PBR material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_pbr_material(&mut self) -> MaterialHandle {
        let (_, pbr_shader_handle) = self.shader_manager.get_builtin_pbr_shader();
        self.material_manager.create_material(*pbr_shader_handle, &mut self.shader_manager)
    }

    /// Create an unlit material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_unlit_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_unlit_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create a blit material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_blit_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_blit_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create a skybox material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_skybox_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_skybox_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create an equirectangular to cube material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_equirect_to_cube_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_equirect_to_cube_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create a shader from source code.
    /// # Arguments
    /// 
    /// * `shader_source` - The source code of the shader.
    /// * `shader_name` - The name of the shader.
    /// # Returns
    /// 
    /// * `ShaderHandle` - The handle of the created shader.
    pub fn create_shader(&mut self, shader_source: &str, shader_name: String) -> ShaderHandle {
        self.shader_manager.create_shader(shader_source, shader_name)
    }

    /// Create a material from shader handle.
    /// # Arguments
    /// 
    /// * `shader_handle` - The handle of the shader.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_material(&mut self, shader_handle: ShaderHandle) -> MaterialHandle {
        self.material_manager.create_material(shader_handle, &mut self.shader_manager)
    }

    /// Get a material by handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `Option<&Material>` - The material if it exists.
    pub fn get_material(&self, handle: &MaterialHandle) -> Option<&Material> {
        self.material_manager.get_material(handle)
    }

    /// Get a material by handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `&Material` - The material.
    pub fn get_material_forcely(&self, handle: &MaterialHandle) -> &Material {
        self.material_manager.get_material_forcely(handle)
    }

    /// Get a material by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `Option<&mut Material>` - The material if it exists.
    pub fn get_material_mut(&mut self, handle: &MaterialHandle) -> Option<&mut Material> {
        self.material_manager.get_material_mut(handle)
    }

    /// Get a material by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `&mut Material` - The material.
    pub fn get_material_mut_forcely(&mut self, handle: &MaterialHandle) -> &mut Material {
        self.material_manager.get_material_mut_forcely(handle)
    }

    /// Load a model from file.
    /// # Arguments
    /// 
    /// * `path` - The path of the model file.
    /// # Returns
    /// 
    /// * `Result<NodeHandle, Box<dyn std::error::Error>>` - The handle of the loaded model node if successful, or an error.
    pub fn load_model(&mut self, path: &str) -> Result<NodeHandle, Box<dyn std::error::Error>> {
        let mut logic_context = self.get_logic_context();
        let model_loader = ModelLoader::new();
        model_loader.load(&mut logic_context, path)
    }

    /// Get a light component by node handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&Light>` - The light component if it exists.
    pub fn get_light(&mut self, handle: &NodeHandle) -> Option<&Light> {
        self.world.current_scene().get_component(handle)
    }

    /// Get a light component by node handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&Light` - The light component.
    /// # Panics
    /// 
    /// * If the light component does not exist.
    pub fn get_light_forcely(&mut self, handle: &NodeHandle) -> &Light {
        self.world.current_scene().get_component(handle).expect("Engine failed to get Light component forcely.")
    }

    /// Get a light component by node handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut Light>` - The light component if it exists.
    pub fn get_light_mut(&mut self, handle: &NodeHandle) -> Option<&mut Light> {
        self.world.current_scene_mut().get_component_mut(handle)
    }

    /// Get a light component by node handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut Light` - The light component.
    /// # Panics
    /// 
    /// * If the light component does not exist.
    pub fn get_light_mut_forcely(&mut self, handle: &NodeHandle) -> &mut Light {
        self.world.current_scene_mut().get_component_mut(handle).expect("LogicContext failed to get Light component mutably forcely.")
    }

    /// Get a node by handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&Node>` - The node if it exists.
    pub fn get_node(&mut self, handle: &NodeHandle) -> Option<&Node> {
        self.world.current_scene().get_node(handle)
    }

    /// Get a node by handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&Node` - The node.
    /// # Panics
    /// 
    /// * If the node does not exist.
    pub fn get_node_forcely(&mut self, handle: &NodeHandle) -> &Node {
        self.world.current_scene().get_node(handle).expect("Engine failed to get Node forcely.")
    }

    /// Get a node by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut Node>` - The node if it exists.
    pub fn get_node_mut(&mut self, handle: &NodeHandle) -> Option<&mut Node> {
        self.world.current_scene_mut().get_node_mut(handle)
    }

    /// Get a node by handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut Node` - The node.
    /// # Panics
    /// 
    /// * If the node does not exist.
    pub fn get_node_mut_forcely(&mut self, handle: &NodeHandle) -> &mut Node {
        self.world.current_scene_mut().get_node_mut(handle).expect("Engine failed to get Node mutably forcely.")
    }

    /// Get a transform component by node handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&Transform>` - The transform component if it exists.
    pub fn get_transform(&mut self, handle: &NodeHandle) -> Option<&Transform> {
        self.get_node(handle).map(|node| &node.transform)
    }

    /// Get a transform component by node handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&Transform` - The transform component.
    /// # Panics
    /// 
    /// * If the transform component does not exist.
    pub fn get_transform_forcely(&mut self, handle: &NodeHandle) -> &Transform {
        &self.get_node_forcely(handle).transform
    }

    /// Get a transform component by node handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut Transform>` - The transform component if it exists.
    pub fn get_transform_mut(&mut self, handle: &NodeHandle) -> Option<&mut Transform> {
        self.get_node_mut(handle).map(|node| &mut node.transform)
    }

    /// Get a transform component by node handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut Transform` - The transform component.
    /// # Panics
    /// 
    /// * If the transform component does not exist.
    pub fn get_transform_mut_forcely(&mut self, handle: &NodeHandle) -> &mut Transform {
        &mut self.get_node_mut_forcely(handle).transform
    }

    /// Get a component by node handle.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&T>` - The component if it exists.
    pub fn get_component<T: Component>(&mut self, node_handle: &NodeHandle) -> Option<&T> {
        self.world.current_scene().get_component::<T>(node_handle)
    }

    /// Get a component by node handle forcely.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&T` - The component.
    /// # Panics
    /// 
    /// * If the component does not exist.
    pub fn get_component_forcely<T: Component>(&mut self, node_handle: &NodeHandle) -> &T {
        self.world.current_scene().get_component::<T>(node_handle).expect("Engine failed to get Component forcely.")
    }

    /// Get a component by node handle mutably.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut T>` - The component if it exists.
    pub fn get_component_mut<T: Component>(&mut self, node_handle: &NodeHandle) -> Option<&mut T> {
        self.world.current_scene_mut().get_component_mut::<T>(node_handle)
    }

    /// Get a component by node handle mutably forcely.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut T` - The component.
    /// # Panics
    /// 
    /// * If the component does not exist.
    pub fn get_component_mut_forcely<T: Component>(&mut self, node_handle: &NodeHandle) -> &mut T {
        self.world.current_scene_mut().get_component_mut::<T>(node_handle).expect("Engine failed to get Component mutably forcely.")
    }

    /// Create a camera controller with target.
    /// # Arguments
    /// 
    /// * `camera_node_handle` - The handle of the camera node.
    /// * `target` - The target of the camera controller.
    pub fn add_camera_controller_with_target(&mut self, camera_node_handle: NodeHandle, target: CameraTarget) {
        let camera_controller =
            CameraController::new(camera_node_handle, target);
        self.add_behavior(camera_controller);
    }

    /// Create a camera controller with position target at origin.
    /// # Arguments
    /// 
    /// * `camera_node_handle` - The handle of the camera node.
    pub fn add_camera_controller(&mut self, camera_node_handle: NodeHandle) {
        let camera_controller =
            CameraController::new(camera_node_handle, CameraTarget::Position(Vec3::ZERO));
        self.add_behavior(camera_controller);
    }
}

impl<'a> LogicContext<'a> {

    /// Create a PBR material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_pbr_material(&mut self) -> MaterialHandle {
        let (_, pbr_shader_handle) = self.shader_manager.get_builtin_pbr_shader();
        self.material_manager.create_material(*pbr_shader_handle, &mut self.shader_manager)
    }

    /// Create an unlit material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_unlit_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_unlit_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create a blit material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_blit_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_blit_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create a skybox material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_skybox_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_skybox_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create an equirectangular to cube material.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_equirect_to_cube_material(&mut self) -> MaterialHandle {
        let (_, shader_handle) = self.shader_manager.get_builtin_equirect_to_cube_shader();
        self.material_manager.create_material(*shader_handle, &mut self.shader_manager)
    }

    /// Create a shader from source code.
    /// # Arguments
    /// 
    /// * `shader_source` - The source code of the shader.
    /// * `shader_name` - The name of the shader.
    /// # Returns
    /// 
    /// * `ShaderHandle` - The handle of the created shader.
    pub fn create_shader(&mut self, shader_source: &str, shader_name: String) -> ShaderHandle {
        self.shader_manager.create_shader(shader_source, shader_name)
    }

    /// Create a material from shader handle.
    /// # Arguments
    /// 
    /// * `shader_handle` - The handle of the shader.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the created material.
    pub fn create_material(&mut self, shader_handle: ShaderHandle) -> MaterialHandle {
        self.material_manager.create_material(shader_handle, &mut self.shader_manager)
    }

    /// Get a material by handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `Option<&Material>` - The material if it exists.
    pub fn get_material(&self, handle: &MaterialHandle) -> Option<&Material> {
        self.material_manager.get_material(handle)
    }

    /// Get a material by handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `&Material` - The material.
    pub fn get_material_forcely(&self, handle: &MaterialHandle) -> &Material {
        self.material_manager.get_material_forcely(handle)
    }

    /// Get a material by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `Option<&mut Material>` - The material if it exists.
    pub fn get_material_mut(&mut self, handle: &MaterialHandle) -> Option<&mut Material> {
        self.material_manager.get_material_mut(handle)
    }

    /// Get a material by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the material.
    /// # Returns
    /// 
    /// * `&mut Material` - The material.
    pub fn get_material_mut_forcely(&mut self, handle: &MaterialHandle) -> &mut Material {
        self.material_manager.get_material_mut_forcely(handle)
    }

    /// Load a model from file.
    /// # Arguments
    /// 
    /// * `path` - The path of the model file.
    /// # Returns
    /// 
    /// * `Result<NodeHandle, Box<dyn std::error::Error>>` - The handle of the loaded model node if successful, or an error.
    pub fn load_model(&mut self, path: &str) -> Result<NodeHandle, Box<dyn std::error::Error>> {
        let model_loader = ModelLoader::new();
        model_loader.load(self, path)
    }

    /// Get a light component by node handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&Light>` - The light component if it exists.
    pub fn get_light(&mut self, handle: &NodeHandle) -> Option<&Light> {
        self.world.current_scene().get_component(handle)
    }

    /// Get a light component by node handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&Light` - The light component.
    /// # Panics
    /// 
    /// * If the light component does not exist.
    pub fn get_light_forcely(&mut self, handle: &NodeHandle) -> &Light {
        self.world.current_scene().get_component(handle).expect("LogicContext failed to get Light component forcely.")
    }

    /// Get a light component by node handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut Light>` - The light component if it exists.
    pub fn get_light_mut(&mut self, handle: &NodeHandle) -> Option<&mut Light> {
        self.world.current_scene_mut().get_component_mut(handle)
    }

    /// Get a light component by node handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut Light` - The light component.
    /// # Panics
    /// 
    /// * If the light component does not exist.
    pub fn get_light_mut_forcely(&mut self, handle: &NodeHandle) -> &mut Light {
        self.world.current_scene_mut().get_component_mut(handle).expect("LogicContext failed to get Light component mutably forcely.")
    }

    /// Get a node by handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&Node>` - The node if it exists.
    pub fn get_node(&mut self, handle: &NodeHandle) -> Option<&Node> {
        self.world.current_scene().get_node(handle)
    }

    /// Get a node by handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&Node` - The node.
    /// # Panics
    /// 
    /// * If the node does not exist.
    pub fn get_node_forcely(&mut self, handle: &NodeHandle) -> &Node {
        self.world.current_scene().get_node(handle).expect("LogicContext failed to get Node forcely.")
    }

    /// Get a node by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut Node>` - The node if it exists.
    pub fn get_node_mut(&mut self, handle: &NodeHandle) -> Option<&mut Node> {
        self.world.current_scene_mut().get_node_mut(handle)
    }

    /// Get a node by handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut Node` - The node.
    /// # Panics
    /// 
    /// * If the node does not exist.
    pub fn get_node_mut_forcely(&mut self, handle: &NodeHandle) -> &mut Node {
        self.world.current_scene_mut().get_node_mut(handle).expect("LogicContext failed to get Node mutably forcely.")
    }

    /// Get a transform component by node handle.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&Transform>` - The transform component if it exists.
    pub fn get_transform(&mut self, handle: &NodeHandle) -> Option<&Transform> {
        self.get_node(handle).map(|node| &node.transform)
    }

    /// Get a transform component by node handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&Transform` - The transform component.
    /// # Panics
    /// 
    /// * If the transform component does not exist.
    pub fn get_transform_forcely(&mut self, handle: &NodeHandle) -> &Transform {
        &self.get_node_forcely(handle).transform
    }

    /// Get a transform component by node handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut Transform>` - The transform component if it exists.
    pub fn get_transform_mut(&mut self, handle: &NodeHandle) -> Option<&mut Transform> {
        self.get_node_mut(handle).map(|node| &mut node.transform)
    }

    /// Get a transform component by node handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut Transform` - The transform component.
    /// # Panics
    /// 
    /// * If the transform component does not exist.
    pub fn get_transform_mut_forcely(&mut self, handle: &NodeHandle) -> &mut Transform {
        &mut self.get_node_mut_forcely(handle).transform
    }

    /// Get a component by node handle.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&T>` - The component if it exists.
    pub fn get_component<T: Component>(&mut self, node_handle: &NodeHandle) -> Option<&T> {
        self.world.current_scene().get_component::<T>(node_handle)
    }

    /// Get a component by node handle forcely.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&T` - The component.
    /// # Panics
    /// 
    /// * If the component does not exist.
    pub fn get_component_forcely<T: Component>(&mut self, node_handle: &NodeHandle) -> &T {
        self.world.current_scene().get_component::<T>(node_handle).expect("LogicContext failed to get Component forcely.")
    }

    /// Get a component by node handle mutably.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `Option<&mut T>` - The component if it exists.
    pub fn get_component_mut<T: Component>(&mut self, node_handle: &NodeHandle) -> Option<&mut T> {
        self.world.current_scene_mut().get_component_mut::<T>(node_handle)
    }

    /// Get a component by node handle mutably forcely.
    /// # Arguments
    /// 
    /// * `node_handle` - The handle of the node.
    /// # Returns
    /// 
    /// * `&mut T` - The component.
    /// # Panics
    /// 
    /// * If the component does not exist.
    pub fn get_component_mut_forcely<T: Component>(&mut self, node_handle: &NodeHandle) -> &mut T {
        self.world.current_scene_mut().get_component_mut::<T>(node_handle).expect("LogicContext failed to get Component mutably forcely.")
    }

}