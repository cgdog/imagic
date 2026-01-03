use crate::{assets::{Mesh, MeshHandle}, core::arena::Arena};

/// The mesh manager of the engine.
/// 
/// It provides the essential APIs for engine users to create and manage meshes.
pub struct MeshManager {
    meshes: Arena<Mesh>,
}

impl MeshManager {
    /// Create a new mesh manager.
    pub(crate) fn new() -> Self {
        Self {
            meshes: Arena::new(),
        }
    }

    /// Add a new mesh to this mesh manager.
    /// # Arguments
    /// * `mesh` - The mesh to be added.
    /// 
    /// # Returns
    /// * `MeshHandle` - The handle of the added mesh.
    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshHandle {
        self.meshes.add(mesh)
    }

    /// Remove a mesh from this mesh manager.
    /// # Arguments
    /// * `handle` - The handle of the mesh to be removed.
    /// 
    /// # Returns
    /// * `Option<Mesh>` - The removed mesh. None if the handle is invalid.
    pub fn remove_mesh(&mut self, handle: &MeshHandle) -> Option<Mesh> {
        self.meshes.remove(handle)
    }

    /// Get a mesh reference by its handle.
    /// # Arguments
    /// * `handle` - The handle of the mesh to be retrieved.
    /// 
    /// # Returns
    /// * `Option<&Mesh>` - The mesh. None if the handle is invalid.
    pub fn get_mesh(&self, handle: &MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle)
    }

    /// Get a mesh reference by its handle forcely.
    /// # Arguments
    /// * `handle` - The handle of the mesh to be retrieved.
    /// 
    /// # Returns
    /// * `&Mesh` - The mesh. Panics if the handle is invalid.
    /// # Panics
    /// 
    /// * If the mesh not found.
    pub fn get_mesh_forcely(&self, handle: &MeshHandle) -> &Mesh {
        self.meshes.get_forcely(handle)
    }

    /// Get mutable reference of a mesh by its handle.
    /// # Arguments
    /// * `handle` - The handle of the mesh to be retrieved.
    /// 
    /// # Returns
    /// * `Option<&mut Mesh>` - The mesh. None if the handle is invalid.
    pub fn get_mesh_mut(&mut self, handle: &MeshHandle) -> Option<&mut Mesh> {
        self.meshes.get_mut(handle)
    }

    /// Get mutable reference of a mesh by its handle forcely.
    /// # Arguments
    /// * `handle` - The handle of the mesh to be retrieved.
    /// 
    /// # Returns
    /// * `&mut Mesh` - The mesh. Panics if the handle is invalid.
    /// # Panics
    /// 
    /// * If the mesh not found.
    pub fn get_mesh_mut_forcely(&mut self, handle: &MeshHandle) -> &mut Mesh {
        self.meshes.get_mut_forcely(handle)
    }
}