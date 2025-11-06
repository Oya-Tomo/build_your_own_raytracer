//! Basic geometric types and helpers used by the raytracer.
// Simple 3D vector type used throughout the renderer.

pub type Float = f32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Vec3 {
    /// Create a new Vec3.
    pub const fn new(x: Float, y: Float, z: Float) -> Self {
        Self { x, y, z }
    }

    /// Zero vector.
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Component-wise product (Hadamard).
    pub fn hadamard(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    /// Dot product.
    pub fn dot(self, other: Self) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Squared length.
    pub fn length_squared(self) -> Float {
        self.dot(self)
    }

    /// Euclidean length.
    pub fn length(self) -> Float {
        self.length_squared().sqrt()
    }

    /// Return a normalized vector. If length is zero, returns the original vector.
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { self } else { self / len }
    }

    /// Reflect this vector around a normal.
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * 2.0 * self.dot(normal)
    }
}

// Operator implementations
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<Float> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Float) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3> for Float {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Div<Float> for Vec3 {
    type Output = Self;
    fn div(self, rhs: Float) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

// Indexing: 0 -> x, 1 -> y, 2 -> z
impl Index<usize> for Vec3 {
    type Output = Float;
    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vec3 index out of bounds: {}", idx),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vec3 index out of bounds: {}", idx),
        }
    }
}

// Conversions
impl From<[Float; 3]> for Vec3 {
    fn from(a: [Float; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
}

impl From<Vec3> for [Float; 3] {
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

mod test {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        assert_eq!(v1 + v2, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(v2 - v1, Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(-v1, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(v1 * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * v1, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(v2 / 2.0, Vec3::new(2.0, 2.5, 3.0));

        assert_eq!(v1.dot(v2), 32.0);
        assert_eq!(v1.cross(v2), Vec3::new(-3.0, 6.0, -3.0));
        assert_eq!(v1.length_squared(), 14.0);
        assert_eq!(v1.length(), (14.0f32).sqrt());
        assert!(v1.normalize().length() - 1.0 < 1e-6);
    }
}
