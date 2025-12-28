
use crate::{
    assets::{MaterialHandle, meshes::mesh::Mesh},
    graphics::uniform::BuiltinUniforms,
    impl_component,
    types::RR,
};


pub struct MeshRenderer {
    pub mesh: RR<Mesh>,
    /// One sub mesh has a seperate material.
    // pub materials: Vec<Material>,
    pub materials: Vec<MaterialHandle>,

    /// Per object builtin uniforms, like model matrix.
    pub(crate) per_object_uniforms: BuiltinUniforms,
}

impl_component!(MeshRenderer);

impl MeshRenderer {
    // pub fn new(mesh: RR<Mesh>, materials: Vec<RR<Material>>) -> Self {
    pub fn new(mesh: RR<Mesh>, materials: Vec<MaterialHandle>) -> Self {
        Self {
            materials,
            mesh,
            per_object_uniforms: BuiltinUniforms::new("MeshRenderer".to_owned()),
        }
    }
}
