use crate::math::Vec3;

/// Spherical coordinate system, which provide functions to convert to or from coordinates between Cartesian coordinate.
pub struct SphericalCoordinate {
    pub theta: f32,
    pub phi: f32,
    pub radius: f32,
}

impl Default for SphericalCoordinate {
    fn default() -> Self {
        Self { theta: 0.0, phi: 0.0, radius: 1.0 }
    }
}

impl SphericalCoordinate {
    pub fn new(theta: f32, phi: f32, radius: f32) -> Self {
        Self {
            theta,
            phi,
            radius
        }
    }

    pub fn from_cartesian(xyz: Vec3) -> Self {
        let radius = xyz.length();
        let theta = (xyz.y / radius).acos();
        let phi = (xyz.x / xyz.z).atan();
        Self { theta, phi, radius }
    }

    pub fn to_cartesian(&self) -> Vec3 {
        let y = self.radius * self.theta.cos();
        let xz = self.radius * self.theta.sin();
        let x = xz * self.phi.sin();
        let z = xz * self.phi.cos();
        Vec3::new(x, y, z)
    }
}