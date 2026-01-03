use wgpu::VertexBufferLayout;

use crate::{assets::meshes::{vertex_format::VertexFormat}, math::{color::Color, Vec2, Vec3, Vec4}};

pub(crate) type VertexAttribute = wgpu::VertexAttribute;

/// The vertex attributes of a mesh, including position, normal, tangent, color, uv, uv1, etc.
/// Each attribute is stored in a separate vector, and the index of each attribute corresponds to the
/// same vertex.
/// For example, the position of the first vertex is stored in position[0], the normal of the first vertex
/// is stored in normal[0], and so on.
/// The length of each attribute vector which is not empty should be the same, which is the number of vertices in the mesh.
/// 
/// In WGSL, the vertex attribute locations always start from 0 and increment in the order of position, normal, tangent, color, uv, and uv1.
/// If an attribute does not exist, its location is skipped.
/// 
/// For example:
/// - If there exists 4 vertex attributes **position, normal, tangent, color**, then the wgsl will be:
/// ```wgsl
/// struct VSInput {
///    @location(0) position: vec3f,
///    @location(1) normal: vec3f,
///    @location(2) tangent: vec3f,
///    @location(3) color: vec3f,
/// }
/// ```
/// In this case, if your wgsl shader only need vertex attributes postion and color, then your wgsl could be:
/// ```wgsl
/// struct VSInput {
///    @location(0) position: vec3f,
///    // note the location of color is 3 instead of 1.
///    @location(3) color: vec3f,
/// }
/// ```
/// - If there only exists 3 vertex attributes **position, normal, color** without tangent, then the wgsl will be:
/// ```wgsl
/// struct VSInput {
///    @location(0) position: vec3f,
///    @location(1) normal: vec3f,
///    @location(2) color: vec3f,
/// }
/// 
pub struct VertexAttributes {
    /// The position of each vertex.
    pub position: Vec<Vec3>,
    /// The normal of each vertex.
    pub normal: Vec<Vec3>,
    /// The tangent of each vertex.
    pub tangent: Vec<Vec4>,
    /// The color of each vertex.
    pub color: Vec<Color>,
    /// The uv of each vertex.
    pub uv: Vec<Vec2>,
    /// The uv1 of each vertex.
    pub uv1: Vec<Vec2>,
    
    pub(crate) attributes: Vec<VertexAttribute>,
    pub(crate) array_stride: u64,
    pub(crate) is_dirty: bool,
}

impl Default for VertexAttributes {
    fn default() -> Self {
        Self {
            is_dirty: true,
            array_stride: 12,
            position: Vec::new(),
            normal: Vec::new(),
            tangent: Vec::new(),
            color: Vec::new(),
            uv: Vec::new(),
            uv1: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

impl VertexAttributes {
    /// Create the vertex buffer layout which is required when computing renderpipeline.
    pub(crate) fn compute_vertex_buffer_layout(&'_ self) -> wgpu::VertexBufferLayout<'_> {
        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &self.attributes,
        };

        vertex_buffer_layout
    }

    pub(crate) fn compute_vertex_attributes(&mut self) {
        if !self.is_dirty {
            return;
        }
        self.is_dirty = false;
        let attributes = &mut self.attributes;
        // let mut attributes: Vec<VertexAttribute> = vec![];
        attributes.clear();
        let mut cur_offset = 0;
        let mut cur_shader_location = 0;
        if self.position.len() > 0 {
            let attribute = VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: cur_offset,
                shader_location: cur_shader_location,
            };
            attributes.push(attribute);
            cur_offset += 12;
            cur_shader_location += 1;
        }

        if self.normal.len() > 0 {
            let attribute = VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: cur_offset,
                shader_location: cur_shader_location,
            };
            attributes.push(attribute);
            cur_offset += 12;
            cur_shader_location += 1;
        }

        if self.tangent.len() > 0 {
            let attribute = VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: cur_offset,
                shader_location: cur_shader_location,
            };
            attributes.push(attribute);
            cur_offset += 16;
            cur_shader_location += 1;
        }

        if self.color.len() > 0 {
            let attribute = VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: cur_offset,
                shader_location: cur_shader_location,
            };
            attributes.push(attribute);
            cur_offset += 16;
            cur_shader_location += 1;
        }

        if self.uv.len() > 0 {
            let attribute = VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: cur_offset,
                shader_location: cur_shader_location,
            };
            attributes.push(attribute);
            cur_offset += 8;
            cur_shader_location += 1;
        }

        if self.uv1.len() > 0 {
            let attribute = VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: cur_offset,
                shader_location: cur_shader_location,
            };
            attributes.push(attribute);
            cur_offset += 8;
            // cur_shader_location += 1;
        }
        self.array_stride = cur_offset;
    }

    /// Get the vertex data in a flat vector.
    /// 
    /// # Returns
    /// * `Vec<f32>` - The vertex data in a flat vector.
    pub fn content(&self) -> Vec<f32> {
        let mut data: Vec<f32> = vec![];
        let vertex_num = self.position.len();
        for i in 0..vertex_num {
            let pos = &self.position[i];
            data.push(pos.x);
            data.push(pos.y);
            data.push(pos.z);

            if self.normal.len() > 0 {
                let normal = &self.normal[i];
                data.push(normal.x);
                data.push(normal.y);
                data.push(normal.z);
            }

            if self.tangent.len() > 0 {
                let tangent = &self.tangent[i];
                data.push(tangent.x);
                data.push(tangent.y);
                data.push(tangent.z);
                data.push(tangent.w);
            }

            if self.color.len() > 0 {
                let color = &self.color[i];
                data.push(color.r);
                data.push(color.g);
                data.push(color.b);
                data.push(color.a);
            }

            if self.uv.len() > 0 {
                let uv = &self.uv[i];
                data.push(uv.x);
                data.push(uv.y);
            }

            if self.uv1.len() > 0 {
                let uv1 = &self.uv1[i];
                data.push(uv1.x);
                data.push(uv1.y);
            }
        }
        data
    }

}