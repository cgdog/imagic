use crate::{assets::{Material, MaterialHandle, ShaderHandle, ShaderManager}, core::arena::Arena};

/// Material manager.
pub struct MaterialManager {
    materials: Arena<Material>,
}

impl MaterialManager {

    /// Create a new material manager.
    pub(crate) fn new() -> Self {
        Self {
            materials: Arena::new(),
        }
    }

    /// Create a new material.
    /// # Arguments
    /// 
    /// * `shader_handle` - The shader handle to use.
    /// * `shader_manager` - The shader manager to use.
    pub fn create_material(&mut self, shader_handle: ShaderHandle, shader_manager: &mut ShaderManager) -> MaterialHandle {
        let material = Material::new(shader_handle, shader_manager);
        self.materials.add(material)
    }

    /// Add a material.
    /// # Arguments
    /// * `material` - The material to add.
    /// # Returns
    /// 
    /// * `MaterialHandle` - The handle of the added material.
    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        self.materials.add(material)
    }

    /// Get a material by handle.
    /// # Arguments
    /// 
    /// * `handle` - The material handle to get.
    /// # Returns
    /// 
    /// * `Some(&Material)` - The material if found.
    /// * `None` - The material not found.
    pub fn get_material(&self, handle: &MaterialHandle) -> Option<&Material> {
        self.materials.get(handle)
    }

    /// Get a material by handle forcely.
    /// # Arguments
    /// 
    /// * `handle` - The material handle to get.
    /// # Returns
    /// 
    /// * `&Material` - The material.
    /// # Panics
    /// 
    /// * If the material not found.
    pub fn get_material_forcely(&self, handle: &MaterialHandle) -> &Material {
        self.materials.get_forcely(handle)
    }

    /// Get a material by handle mutably.
    /// # Arguments
    /// 
    /// * `handle` - The material handle to get.
    /// # Returns
    /// 
    /// * `Some(&mut Material)` - The material if found.
    /// * `None` - The material not found.
    pub fn get_material_mut(&mut self, handle: &MaterialHandle) -> Option<&mut Material> {
        self.materials.get_mut(handle)
    }

    /// Get a material by handle mutably forcely.
    /// # Arguments
    /// 
    /// * `handle` - The material handle to get.
    /// # Returns
    /// 
    /// * `&mut Material` - The material.
    /// # Panics
    /// 
    /// * If the material not found.
    pub fn get_material_mut_forcely(&mut self, handle: &MaterialHandle) -> &mut Material {
        self.materials.get_mut_forcely(handle)
    }

    /// Destroy a material by handle.
    /// # Arguments
    /// 
    /// * `handle` - The material handle to destroy.
    /// # Returns
    /// 
    /// * `true` - The material is destroyed.
    /// * `false` - The material not found.
    pub fn destroy_material(&mut self, handle: &MaterialHandle) -> bool {
        self.materials.remove(handle).is_some()
    }
}