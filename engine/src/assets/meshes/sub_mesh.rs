/// A sub-mesh of a mesh. A SubMesh is corresponding to single material.
pub struct SubMesh {
    /// The start index of the sub-mesh in the index buffer.
    pub index_start: u32,
    /// The number of indices in the sub-mesh.
    pub index_count: u32,
    /// The base vertex of the sub-mesh.
    pub base_vertex: u32,
}

impl SubMesh {
    /// Creates a new sub-mesh.
    /// 
    /// # Arguments
    /// 
    /// * `index_start` - The start index of the sub-mesh in the index buffer.
    /// * `index_count` - The number of indices in the sub-mesh.
    /// * `base_vertex` - The base vertex of the sub-mesh.
    pub fn new(index_start: u32, index_count: u32, base_vertex: u32) -> Self {
        Self {
            index_start,
            index_count,
            base_vertex,
        }
    }
}
