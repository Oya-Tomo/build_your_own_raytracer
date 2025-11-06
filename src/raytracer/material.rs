//! Material definitions for the raytracer.

use super::vector::Float;

/// Color represented in RGB format.
///
/// Values are in the range [0.0, ∞) representing linear light energy.
/// This allows HDR (high dynamic range) rendering where values can exceed 1.0.
/// Use `tone_map()` or `to_rgb8()` to convert to display-ready values.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: Float,
    pub g: Float,
    pub b: Float,
}

impl Color {
    /// Create a new color from RGB components (0.0 to 1.0).
    pub fn new(r: Float, g: Float, b: Float) -> Self {
        Self { r, g, b }
    }

    /// Black color.
    pub const fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    /// White color.
    pub const fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    /// Red color.
    pub const fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        }
    }

    /// Green color.
    pub const fn green() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 0.0,
        }
    }

    /// Blue color.
    pub const fn blue() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 1.0,
        }
    }
}

// Operator implementations for Color
use std::ops::{Add, Mul};

impl Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl Mul<Float> for Color {
    type Output = Self;
    fn mul(self, rhs: Float) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl Mul<Color> for Float {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl Mul for Color {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}

/// Material properties for rendering.
///
/// Uses several components to define how light interacts:
/// - albedo: Surface color reflecting diffuse light (0.0 to 1.0 per channel)
/// - diffuse_rate: Portion of light scattered in all directions (matte surface)
/// - specular_rate: Portion of light reflected sharply (mirror-like surface)
/// - transmission_rate: Portion of light transmitted through (transparency)
/// - absorption: Absorption coefficient per channel for Beer's law attenuation
#[derive(Copy, Clone, Debug)]
pub struct Material {
    /// Surface color (albedo) for diffuse reflection (0.0 to 1.0 per channel).
    pub albedo: Color,
    /// Diffuse reflectivity (0.0 to 1.0) - matte reflection component.
    pub diffuse_rate: Float,
    /// Specular reflectivity (0.0 to 1.0) - mirror-like reflection component.
    pub specular_rate: Float,
    /// Transmission rate (0.0 to 1.0) - transparency component.
    pub transmission_rate: Float,
    /// Refractive index for Snell's law when light is transmitted.
    /// Typical values: vacuum=1.0, air≈1.0, glass≈1.5, diamond≈2.4
    pub refractive_index: Float,
    /// Absorption coefficient per channel for Beer's law.
    /// Used to simulate light absorption when passing through the material.
    /// Higher values = stronger absorption. (0, 0, 0) = no absorption (vacuum/air).
    pub absorption: Color,
}

impl Material {
    /// Create a new material with specified properties.
    ///
    /// # Arguments
    /// * `albedo` - Surface color (0.0-1.0 per channel)
    /// * `diffuse_rate` - Diffuse (matte) reflection rate (0.0-1.0)
    /// * `specular_rate` - Specular (mirror) reflection rate (0.0-1.0)
    /// * `transmission_rate` - Transmission (transparency) rate (0.0-1.0)
    /// * `refractive_index` - Refractive index for refraction
    /// * `absorption` - Absorption coefficient per channel (Beer's law)
    pub fn new(
        albedo: Color,
        diffuse_rate: Float,
        specular_rate: Float,
        transmission_rate: Float,
        refractive_index: Float,
        absorption: Color,
    ) -> Self {
        Self {
            albedo,
            diffuse_rate: diffuse_rate.max(0.0).min(1.0),
            specular_rate: specular_rate.max(0.0).min(1.0),
            transmission_rate: transmission_rate.max(0.0).min(1.0),
            refractive_index,
            absorption,
        }
    }

    /// Create a purely diffuse (matte) material.
    pub fn matte(albedo: Color, diffuse_rate: Float) -> Self {
        Self::new(albedo, diffuse_rate, 0.0, 0.0, 1.0, Color::black())
    }

    /// Create a purely specular (mirror-like) material.
    pub fn mirror(albedo: Color, specular_rate: Float) -> Self {
        Self::new(albedo, 0.0, specular_rate, 0.0, 1.0, Color::black())
    }

    /// Create a purely transparent (dielectric) material.
    pub fn transparent(albedo: Color, transmission_rate: Float, refractive_index: Float) -> Self {
        Self::new(
            albedo,
            0.0,
            0.0,
            transmission_rate,
            refractive_index,
            Color::black(),
        )
    }

    /// Create a perfect diffuse surface (white Lambertian material).
    pub fn diffuse_surface() -> Self {
        Self::matte(Color::white(), 0.8)
    }

    /// Create a perfect mirror.
    pub fn perfect_mirror() -> Self {
        Self::mirror(Color::white(), 1.0)
    }

    /// Create glass (partially transparent with some reflection).
    pub fn glass(transmission_rate: Float) -> Self {
        Self::new(
            Color::white(),
            0.0,
            0.1,
            transmission_rate,
            1.5,            // typical glass refractive index
            Color::black(), // no absorption
        )
    }

    /// Create a metallic material (e.g., steel, gold, copper).
    pub fn metal(albedo: Color, specular_rate: Float, diffuse_rate: Float) -> Self {
        Self::new(
            albedo,
            diffuse_rate,
            specular_rate,
            0.0, // metals are opaque
            1.0,
            Color::black(), // no absorption in vacuum
        )
    }

    /// Create a perfect metallic mirror.
    pub fn perfect_metal() -> Self {
        Self::metal(Color::white(), 1.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(0.5, 0.7, 0.9);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.7);
        assert_eq!(color.b, 0.9);
    }

    #[test]
    fn test_color_predefined() {
        assert_eq!(Color::black(), Color::new(0.0, 0.0, 0.0));
        assert_eq!(Color::white(), Color::new(1.0, 1.0, 1.0));
        assert_eq!(Color::red(), Color::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_add() {
        let c1 = Color::new(0.3, 0.4, 0.5);
        let c2 = Color::new(0.1, 0.2, 0.3);
        let result = c1 + c2;
        assert_eq!(result, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn test_color_mul_scalar() {
        let color = Color::new(0.5, 0.6, 0.7);
        let result = color * 2.0;
        assert_eq!(result, Color::new(1.0, 1.2, 1.4));
    }

    #[test]
    fn test_color_mul_color() {
        let c1 = Color::new(0.5, 0.6, 0.8);
        let c2 = Color::new(0.2, 0.5, 0.5);
        let result = c1 * c2;
        assert_eq!(result, Color::new(0.1, 0.3, 0.4));
    }
}
