pub type Vec4 = glam::Vec4;
pub type Vec3 = glam::Vec3;
pub type Vec2 = glam::Vec2;
pub type IVec3 = glam::IVec3;
pub type UVec3 = glam::UVec3;
pub type U16Vec3 = glam::U16Vec3;

pub type UVec4 = glam::UVec4;
pub type IVec4 = glam::IVec4;

pub type Mat4 = glam::Mat4;
pub type Mat3 = glam::Mat3;

pub type Quat = glam::Quat;

pub type EulerRot = glam::EulerRot;

pub mod color;
pub mod spherical_coordinate;
pub mod spherical_harmonics;

pub use color::*;
pub use spherical_coordinate::*;