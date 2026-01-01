use bytemuck::{Pod, Zeroable};

use crate::math::{Vec3, Vec4};

pub type ColorChannelType = f32;

/// Color
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Color {
    pub r: ColorChannelType,
    pub g: ColorChannelType,
    pub b: ColorChannelType,
    pub a: ColorChannelType,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const PURPLE: Color = Color {
        r: 0.5,
        g: 0.0,
        b: 0.5,
        a: 1.0,
    };    

    pub fn new(r: ColorChannelType, g: ColorChannelType, b: ColorChannelType, a: ColorChannelType) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: ColorChannelType, g: ColorChannelType, b: ColorChannelType) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_array(arr: [f32; 4]) -> Self {
        Self { r: arr[0], g: arr[1], b: arr[2], a: arr[3] }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn scalar(value: ColorChannelType) -> Self {
        Self {
            r: value,
            g: value,
            b: value,
            a: 1.0
        }
    }

    pub fn mix(&self, other: & Color, ratio: f32) -> Color {
        let one_minus_ratio = 1.0 - ratio;
        Color::new(
            self.r * one_minus_ratio + other.r * ratio,
            self.g * one_minus_ratio + other.g * ratio,
            self.b * one_minus_ratio + other.b * ratio,
            self.a * one_minus_ratio + other.a * ratio,
        )
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        Self { r: value.r as f64, g: value.g as f64, b: value.b as f64, a: value.a as f64 }
    }
}

impl From<wgpu::Color> for Color {
    fn from(value: wgpu::Color) -> Self {
        Self { r: value.r as ColorChannelType, g: value.g as ColorChannelType, b: value.b as ColorChannelType, a: value.a as ColorChannelType }
    }
}

impl From<Color> for Vec4 {
    fn from(c: Color) -> Self {
        Vec4::new(c.r as f32, c.g as f32, c.b as f32, c.a as f32)
    }
}

impl From<Color> for Vec3 {
    fn from(c: Color) -> Self {
        Vec3::new(c.r as f32, c.g as f32, c.b as f32)
    }
}

impl From<Vec4> for Color {
    fn from(v: Vec4) -> Self {
        Color::new(v.x as ColorChannelType, v.y as ColorChannelType, v.z as ColorChannelType, v.w as ColorChannelType)
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Color::rgb(v.x as ColorChannelType, v.y as ColorChannelType, v.z as ColorChannelType)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

impl std::ops::Add for Color {
    type Output = Color;
    fn add(self, rhs: Self) -> Self::Output {
        Color::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl std::ops::Add<ColorChannelType> for Color {
    type Output = Color;
    fn add(self, rhs: ColorChannelType) -> Self::Output {
        Color::new(
            self.r + rhs,
            self.g + rhs,
            self.b + rhs,
            self.a + rhs,
        )
    }
}

impl std::ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
    }
}

impl std::ops::AddAssign<ColorChannelType> for Color {
    fn add_assign(&mut self, rhs: ColorChannelType) {
        self.r += rhs;
        self.g += rhs;
        self.b += rhs;
        self.a += rhs;
    }
}

impl std::ops::Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Self) -> Self::Output {
         Color::new(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a,
        )
    }
}

impl std::ops::Sub<ColorChannelType> for Color {
    type Output = Color;
    fn sub(self, rhs: ColorChannelType) -> Self::Output {
        Color::new(
            self.r - rhs,
            self.g - rhs,
            self.b - rhs,
            self.a - rhs,
        )
    }
}

impl std::ops::SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self.a -= rhs.a;
    }
}

impl std::ops::SubAssign<ColorChannelType> for Color {
    fn sub_assign(&mut self, rhs: ColorChannelType) {
        self.r -= rhs;
        self.g -= rhs;
        self.b -= rhs;
        self.a -= rhs;
    }
}

impl std::ops::Mul for Color {
    type Output = Color;
    fn mul(self, rhs: Self) -> Self::Output {
        Color::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a
        )
    }
}

impl std::ops::Mul<ColorChannelType> for Color {
    type Output = Color;
    fn mul(self, rhs: ColorChannelType) -> Self::Output {
        Color::new(
            self.r * rhs,
            self.g * rhs,
            self.b * rhs,
            self.a * rhs
        )
    }
}

impl std::ops::MulAssign<ColorChannelType> for Color {
    fn mul_assign(&mut self, rhs: ColorChannelType) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self.a *= rhs;
    }
}

impl std::ops::DivAssign<ColorChannelType> for Color {
    fn div_assign(&mut self, rhs: ColorChannelType) {
        assert_ne!(rhs, 0.0);
        self.r.div_assign(rhs);
        self.g.div_assign(rhs);
        self.b.div_assign(rhs);
        self.a.div_assign(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_color() {
        let a = Color::new(1.0, 1.0, 1.0, 1.0);
        let b = Color::new(1.0, 2.0, 3.0, 4.0);
        let c = a + b;
        assert_eq!(c, Color::new(2.0, 3.0, 4.0, 5.0));
    }
}