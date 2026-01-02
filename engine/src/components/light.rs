use bytemuck::{Pod, Zeroable};

use crate::{impl_component, math::Color};

/// The shape of the area light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaLightShape {
    Rectangle { width: f32, height: f32 },
    Disk { radius: f32 },
    Line { length: f32 },
}

/// The type of light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    /// A light that emits light in a specific direction.
    Directional {
        // The direction of the light. provided by transform.
        // direction: Vec3,
        // angular_radius: f32,
    },
    /// A light that emits light in all directions.
    Point {
        /// The range or max distance of the light.
        range: f32,
    },
    /// A light that emits light in a specific direction and shape.
    Spot {
        // The direction of the light. provided by transform.
        // direction: Vec3,
        /// The range or max distance of the light.
        range: f32,
        /// The inner cone angle of the spot light.
        inner_cos: f32,
        /// The outer cone angle of the spot light.
        outer_cos: f32,
        /// The falloff exponent of the spot light.
        falloff_exponent: f32,
    },
    /// A light that emits light in a specific shape.
    Area {
        /// The shape of the area light.
        shape: AreaLightShape,
        /// Whether the light emits light in both directions.
        two_sided: bool,
    },
}

impl LightType {
    pub(crate) fn as_u32(&self) -> u32 {
        match self {
            LightType::Directional { .. } => 0,
            LightType::Point { .. } => 1,
            LightType::Spot { .. } => 2,
            LightType::Area { .. } => 3,
        }
    }
}

/// A light source in the scene. It is a component that can be attached to a node.
pub struct Light {
    /// Whether the light is enabled.
    pub enabled: bool,
    /// The color of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
    /// Whether the light casts shadow.
    pub cast_shadow: bool,
    /// The type of the light.
    pub light_type: LightType,
}

impl_component!(Light);

impl Light {
    /// Creates a new light with the given parameters.
    /// 
    /// # Arguments
    /// 
    /// * `light_type` - The type of the light.
    /// * `color` - The color of the light.
    /// * `intensity` - The intensity of the light.
    /// * `cast_shadow` - Whether the light casts shadow.
    pub fn new(
        light_type: LightType,
        color: Color,
        intensity: f32,
        cast_shadow: bool,
    ) -> Self {
        Self {
            enabled: true,
            color,
            intensity,
            cast_shadow,
            light_type,
        }
    }

    /// Creates a new directional light with the given parameters.
    /// 
    /// # Arguments
    /// 
    /// * `color` - The color of the light.
    /// * `intensity` - The intensity of the light.
    /// * `cast_shadow` - Whether the light casts shadow.
    pub fn new_directional_light(
        // direction: Vec3,
        color: Color,
        intensity: f32,
        cast_shadow: bool,
    ) -> Self {
        Self::new(
            LightType::Directional {},
            color,
            intensity,
            cast_shadow,
        )
    }

    /// Creates a new point light with the given parameters.
    /// 
    /// # Arguments
    /// 
    /// * `range` - The range or max distance of the light.
    /// * `color` - The color of the light.
    /// * `intensity` - The intensity of the light.
    /// * `cast_shadow` - Whether the light casts shadow.
    pub fn new_point_light(
        range: f32,
        color: Color,
        intensity: f32,
        cast_shadow: bool,
    ) -> Self {
        Self::new(
            LightType::Point { range },
            color,
            intensity,
            cast_shadow,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct GPULightData {
    /// flags.x is light type, 0: directional, 1: point, 2: spot, 3: area.
    pub(crate) flags: [u32; 4],
    /// color.w is range or max distance for spot or point light.
    pub(crate) color: [f32;4],
    /// For spot light: direction.w is inner cone cosin.
    pub(crate) direction: [f32; 4],
    /// For spot light: position.w is outter cone cosin.
    pub(crate) position: [f32; 4],
}

impl Default for GPULightData {
    fn default() -> Self {
        Self {
            flags: [0; 4],
            color: [0.0; 4],
            direction: [0.0; 4],
            position: [0.0; 4],
        }
    }
}

pub(crate) struct LightsGPUData {
    pub(crate) lights_count: [u32; 4],
    pub(crate) lights_info: Vec<GPULightData>,
}

impl Default for LightsGPUData {
    fn default() -> Self {
        Self {
            lights_count: [0; 4],
            lights_info: Vec::new(),
        }
    }
}

impl LightsGPUData {
    pub(crate) fn to_vec_u8(&self) -> Vec<u8> {
        let header_size = std::mem::size_of::<[u32; 4]>(); // 16
        let stride = std::mem::size_of::<GPULightData>();  // 64

        let total_size = header_size + stride * self.lights_info.len();

        let mut bytes = Vec::with_capacity(total_size);

        // header
        bytes.extend_from_slice(bytemuck::bytes_of(&self.lights_count));

        // array
        bytes.extend_from_slice(bytemuck::cast_slice(&self.lights_info));

        bytes
    }
}