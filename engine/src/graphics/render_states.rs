
/// Cull mode when face culling.
#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CullMode {
    /// Cull front faces.
    Front = 0,
    /// Cull back faces.
    #[default]
    Back = 1,
    /// Do not cull any faces.
    None = 2,
}

/// Vertex winding order which defines the "front face" of a triangle.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindingOrder {
    /// Triangles with vertices in counter clockwise order are considered the front face.
    ///
    /// This is the default with right handed coordinate spaces.
    #[default]
    Ccw = 0,

    /// Triangles with vertices in clockwise order are considered the front face.
    ///
    /// This is the default with left handed coordinate spaces.
    Cw = 1,
}

impl From<WindingOrder> for wgpu::FrontFace {
    fn from(value: WindingOrder) -> Self {
        match value {
            WindingOrder::Ccw => wgpu::FrontFace::Ccw,
            WindingOrder::Cw => wgpu::FrontFace::Cw,
        }
    }
}

pub type PolygonMode = wgpu::PolygonMode;

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub enum RenderQueue {
    #[default]
    Opaque,
    Skybox,
    Transparent,
}

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct RenderState {
    pub cull_mode: CullMode,
    pub front_face: WindingOrder,
    pub polygon_mode: PolygonMode,
    pub render_queue: RenderQueue,
}
