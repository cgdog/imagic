
use crate::{
    assets::{MaterialHandle, MeshHandle},
    graphics::uniform::BuiltinUniforms,
    impl_component,
};


pub struct MeshRenderer {
    pub mesh: MeshHandle,
    /// One sub mesh has a seperate material.
    // pub materials: Vec<Material>,
    pub materials: Vec<MaterialHandle>,

    /// Per object builtin uniforms, like model matrix.
    pub(crate) per_object_uniforms: BuiltinUniforms,
}

impl_component!(MeshRenderer);

impl MeshRenderer {
    /// Create a new mesh renderer component.
    /// 
    /// # Arguments
    /// 
    /// * `mesh` - The mesh handle.
    /// * `materials` - The materials handles.
    pub fn new(mesh: MeshHandle, materials: Vec<MaterialHandle>) -> Self {
        Self {
            materials,
            mesh,
            per_object_uniforms: BuiltinUniforms::new("MeshRenderer".to_owned()),
        }
    }
}
