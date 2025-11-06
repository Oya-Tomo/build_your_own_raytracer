//! Image processing for rendered output and tone mapping.

use super::material::Color;
use super::vector::Float;

#[allow(dead_code)]

/// Represents an image as a 2D grid of RGB pixels.
/// Internally stores HDR (high dynamic range) values.
#[derive(Clone, Debug)]
pub struct Image {
    /// Image width in pixels
    pub width: usize,
    /// Image height in pixels
    pub height: usize,
    /// Pixel data stored in row-major order: [row0, row1, ...]
    /// Each row contains `width` pixels
    pixels: Vec<Vec<Color>>,
}

/// Trait for tone mapping algorithms that convert HDR values [0, ∞) to LDR [0, 1).
pub trait ToneMapping {
    /// Apply tone mapping to a single color.
    ///
    /// # Arguments
    /// * `color` - The HDR color to tone map
    ///
    /// # Returns
    /// A color in the range [0.0, 1.0] suitable for display
    fn map(&self, color: Color) -> Color;
}

/// Reinhard tone mapping - simple and effective global tone mapping.
/// Formula: mapped = x / (1 + x) for each channel
///
/// This is a simple global tone mapping that preserves local contrast well.
/// It handles very bright values gracefully by compressing them logarithmically.
#[derive(Clone, Debug)]
pub struct Reinhard {
    /// Exposure adjustment factor (> 1.0 darkens, < 1.0 brightens)
    pub exposure: Float,
}

impl Reinhard {
    /// Create a new Reinhard tone mapper with default exposure.
    pub fn new() -> Self {
        Self { exposure: 1.0 }
    }

    /// Create a new Reinhard tone mapper with custom exposure.
    pub fn with_exposure(exposure: Float) -> Self {
        Self { exposure }
    }
}

impl Default for Reinhard {
    fn default() -> Self {
        Self::new()
    }
}

impl ToneMapping for Reinhard {
    fn map(&self, color: Color) -> Color {
        let adjusted = color * self.exposure;
        Color::new(
            adjusted.r / (1.0 + adjusted.r),
            adjusted.g / (1.0 + adjusted.g),
            adjusted.b / (1.0 + adjusted.b),
        )
    }
}

/// Exposure tone mapping - simple linear scaling with gamma correction.
/// Formula: mapped = clamp(x * exposure, 0, 1) with gamma = 1/2.2
///
/// This is the simplest approach: linear multiplication by exposure factor,
/// then gamma correction to account for display gamma.
#[derive(Clone, Debug)]
pub struct Exposure {
    /// Exposure factor (> 1.0 brightens, < 1.0 darkens)
    pub exposure: Float,
    /// Gamma correction factor (typical: 2.2)
    pub gamma: Float,
}

impl Exposure {
    /// Create a new Exposure tone mapper with default settings.
    pub fn new() -> Self {
        Self {
            exposure: 1.0,
            gamma: 2.2,
        }
    }

    /// Create a new Exposure tone mapper with custom exposure.
    pub fn with_exposure(exposure: Float) -> Self {
        Self {
            exposure,
            gamma: 2.2,
        }
    }

    /// Create a new Exposure tone mapper with custom exposure and gamma.
    #[allow(dead_code)]
    pub fn with_exposure_and_gamma(exposure: Float, gamma: Float) -> Self {
        Self { exposure, gamma }
    }
}

impl Default for Exposure {
    fn default() -> Self {
        Self::new()
    }
}

impl ToneMapping for Exposure {
    fn map(&self, color: Color) -> Color {
        let adjusted = color * self.exposure;
        let inv_gamma = 1.0 / self.gamma;

        Color::new(
            adjusted.r.max(0.0).min(1.0).powf(inv_gamma),
            adjusted.g.max(0.0).min(1.0).powf(inv_gamma),
            adjusted.b.max(0.0).min(1.0).powf(inv_gamma),
        )
    }
}

/// ACES Filmic tone mapping - high-quality tone mapping used in the film industry.
/// Based on the Academy Color Encoding System (ACES) RRT (Reference Rendering Transform).
///
/// This is a more sophisticated curve that provides better contrast and color preservation,
/// commonly used in high-end rendering and professional visual effects.
///
/// Reference: Narkowicz, K., Christensen, P., & Esposito, D. (2016).
/// "Improved Color Matching in Real-Time Rendering"
#[derive(Clone, Debug)]
pub struct ACESFilmic;

impl ACESFilmic {
    /// Create a new ACES Filmic tone mapper.
    pub fn new() -> Self {
        Self
    }

    /// ACES RRT + ODT (Reference Rendering Transform + Output Device Transform)
    /// Simplified version of the ACES curve.
    fn aces_tone_map(x: Float) -> Float {
        const A: Float = 2.51;
        const B: Float = 0.03;
        const C: Float = 2.43;
        const D: Float = 0.59;
        const E: Float = 0.14;

        ((x * (A * x + B)) / (x * (C * x + D) + E))
            .max(0.0)
            .min(1.0)
    }
}

impl Default for ACESFilmic {
    fn default() -> Self {
        Self::new()
    }
}

impl ToneMapping for ACESFilmic {
    fn map(&self, color: Color) -> Color {
        Color::new(
            Self::aces_tone_map(color.r),
            Self::aces_tone_map(color.g),
            Self::aces_tone_map(color.b),
        )
    }
}

impl Image {
    /// Create a new image from a 2D vector of colors.
    ///
    /// # Arguments
    /// * `pixels` - 2D vector of colors (rows × columns)
    ///
    /// # Returns
    /// An Image containing the provided pixels
    pub fn from_pixels(pixels: Vec<Vec<Color>>) -> Self {
        let height = pixels.len();
        let width = if height > 0 { pixels[0].len() } else { 0 };

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Get a pixel at the specified coordinates.
    /// Returns None if coordinates are out of bounds.
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if y < self.height && x < self.width {
            Some(self.pixels[y][x])
        } else {
            None
        }
    }

    /// Get mutable reference to a pixel at the specified coordinates.
    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut Color> {
        if y < self.height && x < self.width {
            Some(&mut self.pixels[y][x])
        } else {
            None
        }
    }

    /// Compute the average luminance of the image for exposure correction.
    /// Uses the formula: Luminance = 0.299 * R + 0.587 * G + 0.114 * B
    pub fn average_luminance(&self) -> Float {
        let mut total = 0.0;
        let mut count = 0;

        for row in &self.pixels {
            for color in row {
                total += 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
                count += 1;
            }
        }

        if count > 0 {
            total / count as Float
        } else {
            0.0
        }
    }

    /// Apply exposure correction to the image.
    /// Multiplies all pixel values by the given exposure factor.
    ///
    /// # Arguments
    /// * `exposure` - Exposure factor (> 1.0 brightens, < 1.0 darkens)
    pub fn apply_exposure(&mut self, exposure: Float) {
        for row in &mut self.pixels {
            for color in row {
                *color = *color * exposure;
            }
        }
    }

    /// Convert the HDR image to 8-bit RGB format using the specified tone mapper.
    ///
    /// # Arguments
    /// * `tone_mapper` - A struct implementing the ToneMapping trait
    ///
    /// # Returns
    /// A vector of (R, G, B) tuples in row-major order
    pub fn convert<T: ToneMapping>(&self, tone_mapper: &T) -> Vec<(u8, u8, u8)> {
        let mut result = Vec::with_capacity(self.width * self.height);

        for row in &self.pixels {
            for color in row {
                let tone_mapped = tone_mapper.map(*color);
                // Convert to 8-bit RGB
                let r = (tone_mapped.r * 255.0).clamp(0.0, 255.0) as u8;
                let g = (tone_mapped.g * 255.0).clamp(0.0, 255.0) as u8;
                let b = (tone_mapped.b * 255.0).clamp(0.0, 255.0) as u8;
                result.push((r, g, b));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation() {
        let pixels = vec![
            vec![Color::black(), Color::white()],
            vec![Color::red(), Color::blue()],
        ];
        let image = Image::from_pixels(pixels);

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 2);
    }

    #[test]
    fn test_get_pixel() {
        let pixels = vec![
            vec![Color::black(), Color::white()],
            vec![Color::red(), Color::blue()],
        ];
        let image = Image::from_pixels(pixels);

        assert_eq!(image.get_pixel(0, 0).unwrap(), Color::black());
        assert_eq!(image.get_pixel(1, 0).unwrap(), Color::white());
        assert_eq!(image.get_pixel(0, 1).unwrap(), Color::red());
        assert_eq!(image.get_pixel(1, 1).unwrap(), Color::blue());
        assert!(image.get_pixel(2, 0).is_none()); // out of bounds
    }

    #[test]
    fn test_average_luminance() {
        let pixels = vec![vec![Color::new(1.0, 1.0, 1.0), Color::black()]];
        let image = Image::from_pixels(pixels);

        let avg_lum = image.average_luminance();
        // First pixel: 0.299 + 0.587 + 0.114 = 1.0
        // Second pixel: 0.0
        // Average: 0.5
        assert!((avg_lum - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_apply_exposure() {
        let pixels = vec![vec![Color::new(1.0, 2.0, 3.0)]];
        let mut image = Image::from_pixels(pixels);

        image.apply_exposure(2.0);
        let color = image.get_pixel(0, 0).unwrap();

        assert_eq!(color, Color::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_reinhard_basic() {
        let mapper = Reinhard::new();
        let color = Color::new(1.0, 1.0, 1.0);
        let mapped = mapper.map(color);

        // 1.0 / (1 + 1.0) = 0.5
        assert!((mapped.r - 0.5).abs() < 0.001);
        assert!((mapped.g - 0.5).abs() < 0.001);
        assert!((mapped.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_reinhard_with_exposure() {
        let mapper = Reinhard::with_exposure(2.0);
        let color = Color::new(1.0, 1.0, 1.0);
        let mapped = mapper.map(color);

        // (2.0 * 1.0) / (1 + 2.0 * 1.0) = 2.0 / 3.0 ≈ 0.667
        assert!((mapped.r - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_reinhard_bright_values() {
        let mapper = Reinhard::new();
        let bright = Color::new(10.0, 20.0, 100.0);
        let mapped = mapper.map(bright);

        // Very bright values should map to close to 1.0
        assert!(mapped.r > 0.9);
        assert!(mapped.g > 0.95);
        assert!(mapped.b > 0.99);
    }

    #[test]
    fn test_exposure_basic() {
        let mapper = Exposure::with_exposure(1.0);
        let color = Color::new(0.5, 0.5, 0.5);
        let mapped = mapper.map(color);

        // 0.5^(1/2.2) ≈ 0.735
        assert!((mapped.r - 0.5_f32.powf(1.0 / 2.2)).abs() < 0.001);
    }

    #[test]
    fn test_exposure_clamps() {
        let mapper = Exposure::with_exposure(0.5);
        let color = Color::new(2.0, 2.0, 2.0);
        let mapped = mapper.map(color);

        // (0.5 * 2.0)^(1/2.2) = 1.0^(1/2.2) = 1.0 (clamped)
        assert!(mapped.r <= 1.0);
        assert!(mapped.g <= 1.0);
        assert!(mapped.b <= 1.0);
    }

    #[test]
    fn test_aces_filmic_basic() {
        let mapper = ACESFilmic::new();
        let color = Color::new(1.0, 1.0, 1.0);
        let mapped = mapper.map(color);

        // ACES should map 1.0 to a reasonable display value
        assert!(mapped.r > 0.0 && mapped.r <= 1.0);
        assert!(mapped.g > 0.0 && mapped.g <= 1.0);
        assert!(mapped.b > 0.0 && mapped.b <= 1.0);
    }

    #[test]
    fn test_aces_filmic_black() {
        let mapper = ACESFilmic::new();
        let color = Color::black();
        let mapped = mapper.map(color);

        assert_eq!(mapped, Color::black());
    }

    #[test]
    fn test_convert_with_reinhard() {
        let pixels = vec![vec![Color::new(2.0, 4.0, 0.5)]];
        let image = Image::from_pixels(pixels);
        let mapper = Reinhard::new();

        let rgb8_data = image.convert(&mapper);
        assert_eq!(rgb8_data.len(), 1);

        let (r, g, b) = rgb8_data[0];
        // 2.0/(1+2.0) = 2/3 ≈ 0.667 * 255 ≈ 170
        // 4.0/(1+4.0) = 4/5 = 0.8 * 255 = 204
        // 0.5/(1+0.5) = 1/3 ≈ 0.333 * 255 ≈ 85
        assert!(r >= 168 && r <= 172);
        assert!(g >= 202 && g <= 206);
        assert!(b >= 83 && b <= 87);
    }

    #[test]
    fn test_convert_with_aces() {
        let pixels = vec![vec![Color::new(0.5, 0.5, 0.5)]];
        let image = Image::from_pixels(pixels);
        let mapper = ACESFilmic::new();

        let rgb8_data = image.convert(&mapper);
        assert_eq!(rgb8_data.len(), 1);

        let (r, g, b) = rgb8_data[0];
        // All values should be equal and reasonable
        assert!(r > 0);
        assert_eq!(r, g);
        assert_eq!(g, b);
    }
}
