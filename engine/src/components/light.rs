use crate::{impl_component, math::{Color, Vec3}};

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
        /// The direction of the light.
        direction: Vec3,
        // angular_radius: f32,
    },
    /// A light that emits light in all directions.
    Point {
        /// The range or max distance of the light.
        range: f32,
    },
    /// A light that emits light in a specific direction and shape.
    Spot {
        /// The direction of the light.
        direction: Vec3,
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
    /// * `direction` - The direction of the light.
    /// * `color` - The color of the light.
    /// * `intensity` - The intensity of the light.
    /// * `cast_shadow` - Whether the light casts shadow.
    pub fn new_directional(
        direction: Vec3,
        color: Color,
        intensity: f32,
        cast_shadow: bool,
    ) -> Self {
        Self::new(
            LightType::Directional { direction },
            color,
            intensity,
            cast_shadow,
        )
    }
}
