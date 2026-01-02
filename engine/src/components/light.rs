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
pub(crate) enum LightType {
    /// A light that emits light in a specific direction.
    Directional {
        // angular_radius: f32,
    },
    /// A light that emits light in all directions.
    Point {
        /// The max distance of the light.
        max_distance: f32,
    },
    /// A light that emits light in a specific direction and shape.
    Spot {
        /// The max distance of the spot light.
        max_distance: f32,
        /// The inner angle in radians of the spot light.
        inner_angle: f32,
        /// The outer angle in radians of the spot light.
        outer_angle: f32,
        // The falloff exponent of the spot light.
        // falloff_exponent: f32,
    },
    /// A light that emits light in a specific shape.
    #[allow(unused)]
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
    pub(crate) light_type: LightType,
    /// The cached spot light outer cos.
    pub(crate) cached_outer_cos: f32,
    /// The cached spot light inner cos.
    pub(crate) cached_inner_cos: f32,
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
    pub(crate) fn new(
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
            cached_outer_cos: 0.0,
            cached_inner_cos: 0.0,
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
        max_distance: f32,
        color: Color,
        intensity: f32,
        cast_shadow: bool,
    ) -> Self {
        Self::new(
            LightType::Point { max_distance },
            color,
            intensity,
            cast_shadow,
        )
    }

    /// Creates a new spot light with the given parameters.
    /// 
    /// # Arguments
    /// 
    /// * `max_distance` - The max distance of the light.
    /// * `inner_angle` - The inner angle in radians of the spot light.
    /// * `outer_angle` - The outer angle in radians of the spot light.
    /// * `color` - The color of the light.
    /// * `intensity` - The intensity of the light.
    /// * `cast_shadow` - Whether the light casts shadow.
    pub fn new_spot_light(
        max_distance: f32,
        inner_angle: f32,
        outer_angle: f32,
        color: Color,
        intensity: f32,
        cast_shadow: bool,
    ) -> Self {
        let mut light = Self::new(
            LightType::Spot {
                max_distance,
                inner_angle,
                outer_angle,
            },
            color,
            intensity,
            cast_shadow,
        );
        light.cached_outer_cos = outer_angle.cos();
        light.cached_inner_cos = inner_angle.cos();
        light
    }

    /// Sets the max distance of the point or spot light.
    /// 
    /// # Arguments
    /// 
    /// * `max_distance` - The max distance of the point or spot light.
    pub fn set_max_distance(&mut self, max_distance: f32) {
        match &mut self.light_type {
            LightType::Point { max_distance: point_max_distance } => {
                *point_max_distance = max_distance;
            }
            LightType::Spot { max_distance: spot_range, .. } => {
                *spot_range = max_distance;
            }
            _ => {
                log::warn!("Light type {:?} does not support max distance", self.light_type);
            }
        }
    }

    /// Gets the max distance of the point or spot light.
    /// 
    /// # Returns
    /// 
    /// The max distance of the point or spot light.
    pub fn get_max_distance(&self) -> f32 {
        match &self.light_type {
            LightType::Point { max_distance } => *max_distance,
            LightType::Spot { max_distance: range, .. } => *range,
            _ => {
                log::warn!("Light type {:?} does not support max distance", self.light_type);
                0.0
            }
        }
    }

    /// Sets the outer angle in radians of the spot light.
    /// 
    /// # Arguments
    /// 
    /// * `outer_angle` - The outer angle in radians of the spot light.
    pub fn set_outer_angle(&mut self, outer_angle: f32) {
        match &mut self.light_type {
            LightType::Spot { outer_angle: spot_outer_angle, .. } => {
                *spot_outer_angle = outer_angle;
                self.cached_outer_cos = outer_angle.cos();
            }
            _ => {
                log::warn!("Light type {:?} does not support outer angle", self.light_type);
            }
        }
    }

    /// Gets the outer angle in radians of the spot light.
    /// 
    /// # Returns
    /// 
    /// The outer angle in radians of the spot light.
    pub fn get_outer_angle(&self) -> f32 {
        match &self.light_type {
            LightType::Spot { outer_angle, .. } => *outer_angle,
            _ => {
                log::warn!("Light type {:?} does not support outer angle", self.light_type);
                0.0
            }
        }
    }
    /// Sets the inner angle in radians of the spot light.
    /// 
    /// # Arguments
    /// 
    /// * `inner_angle` - The inner angle in radians of the spot light.
    pub fn set_inner_angle(&mut self, inner_angle: f32) {
        match &mut self.light_type {
            LightType::Spot { inner_angle: spot_inner_angle, .. } => {
                *spot_inner_angle = inner_angle;
                self.cached_inner_cos = inner_angle.cos();
            }
            _ => {
                log::warn!("Light type {:?} does not support inner angle", self.light_type);
            }
        }
    }

    /// Gets the inner angle in radians of the spot light.
    /// 
    /// # Returns
    /// 
    /// The inner angle in radians of the spot light.
    pub fn get_inner_angle(&self) -> f32 {
        match &self.light_type {
            LightType::Spot { inner_angle, .. } => *inner_angle,
            _ => {
                log::warn!("Light type {:?} does not support inner angle", self.light_type);
                0.0
            }
        }
    }

    /// Sets the outer and inner angle in radians of the spot light.
    /// 
    /// # Arguments
    /// 
    /// * `outer_angle` - The outer angle in radians of the spot light.
    /// * `inner_angle` - The inner angle in radians of the spot light.
    pub fn set_outter_inner_angle(&mut self, outer_angle: f32, inner_angle: f32) {
        self.set_outer_angle(outer_angle);
        self.set_inner_angle(inner_angle);
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