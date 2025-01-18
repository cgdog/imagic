//! Spherical coordinates related.

use core::f32;
use glam::Vec3;

/// Spherical coordinate.
#[derive(Clone, Copy)]
pub struct Spherical {
    pub radius: f32,
    /// Polar angle. Its range is [0, Pi], where 0 is positive y axis and Pi is negative y axis.
    pub theta: f32,
    /// Azimuthal angle in xz plane.
    pub phi: f32,
}

impl Default for Spherical {
    fn default() -> Self {
        Self {
            radius: 1.0,
            theta: 0.0,
            phi: 0.0,
        }
    }
}

impl Spherical {

    pub fn create_from_cartesian(cartesian: Vec3) -> Self {
        let radius = cartesian.length();
        let theta = (cartesian.y / radius).acos();
        let phi = cartesian.z.atan2(cartesian.x);
        let spherical = Self {
            radius,
            theta,
            phi,
        };
        spherical
    }

    pub fn from_cartesian(&mut self, cartesian: Vec3) {
        self.radius = cartesian.length();
        self.theta = (cartesian.y / self.radius).acos();
        self.phi = cartesian.z.atan2(cartesian.x);
    }

    pub fn to_cartesian(&self) -> Vec3 {
        let y = self.radius * self.theta.cos();
        let r_xz = self.radius * self.theta.sin();
        let x = r_xz * self.phi.cos();
        let z = r_xz * self.phi.sin();
        Vec3::new(x, y, z)
    }
}