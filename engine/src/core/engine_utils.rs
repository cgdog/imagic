use crate::{assets::{Material, MaterialHandle, ModelLoader, ModelLoaderTrait, ShaderHandle}, core::{Engine, LogicContext, NodeHandle}, math::Vec3, prelude::{CameraController, CameraTarget}};


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

}