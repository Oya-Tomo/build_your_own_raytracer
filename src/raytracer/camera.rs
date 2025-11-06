//! Camera and ray generation for the raytracer.

use super::Ray;
use super::vector::{Float, Vec3};
use std::f32::consts::PI;

/// A camera that generates rays for rendering.
#[derive(Clone, Debug)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Direction the camera is looking towards
    pub direction: Vec3,
    /// Up vector for the camera (defines the roll)
    pub up: Vec3,
    /// Vertical field of view in degrees
    pub fov_degrees: Float,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Subdivision count per pixel for anti-aliasing (e.g., 2 = 2x2 grid)
    pub subdivisions: u32,
}

impl Camera {
    /// Create a new camera.
    ///
    /// # Arguments
    /// * `position` - Camera position in world space
    /// * `direction` - Direction the camera is looking (will be normalized)
    /// * `up` - Up vector (will be normalized)
    /// * `fov_degrees` - Vertical field of view in degrees (typically 45-90)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    pub fn new(
        position: Vec3,
        direction: Vec3,
        up: Vec3,
        fov_degrees: Float,
        width: u32,
        height: u32,
        subdivisions: u32,
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            up: up.normalize(),
            fov_degrees,
            width,
            height,
            subdivisions,
        }
    }

    /// Build the camera basis vectors (right, up, forward).
    fn build_basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = self.direction;
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        (right, up, forward)
    }

    /// Generate rays for all pixels with anti-aliasing support.
    ///
    /// Returns a Vec<Vec<Vec<Ray>>> where:
    /// - First dimension: rows (y)
    /// - Second dimension: columns (x)
    /// - Third dimension: samples within each pixel (subdivisions x subdivisions)
    pub fn generate_rays(&self) -> Vec<Vec<Vec<Ray>>> {
        let (right, up, forward) = self.build_basis();

        // Convert FOV from degrees to radians
        let fov_rad = self.fov_degrees * PI / 180.0;

        // Calculate the height of the view plane at distance 1.0
        let view_height = 2.0 * (fov_rad / 2.0).tan();
        let view_width = view_height * (self.width as Float) / (self.height as Float);

        let mut rays = Vec::new();
        let sub = self.subdivisions as Float;
        let sample_size = 1.0 / sub;

        for y in 0..self.height {
            let mut row = Vec::new();
            for x in 0..self.width {
                let mut pixel_samples = Vec::new();

                // Generate samples within this pixel
                for sy in 0..self.subdivisions {
                    for sx in 0..self.subdivisions {
                        // Offset within the pixel: [0, 1)
                        let offset_x = (sx as Float + 0.5) * sample_size;
                        let offset_y = (sy as Float + 0.5) * sample_size;

                        // Normalize to [-0.5, 0.5] relative to image
                        let u = (x as Float + offset_x) / (self.width as Float) - 0.5;
                        let v = (y as Float + offset_y) / (self.height as Float) - 0.5;

                        // Calculate ray direction in camera space
                        let ray_dir = forward + right * (u * view_width) - up * (v * view_height);

                        let ray = Ray::new(self.position, ray_dir);
                        pixel_samples.push(ray);
                    }
                }

                row.push(pixel_samples);
            }
            rays.push(row);
        }

        rays
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_at() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let p = ray.at(5.0);
        assert_eq!(p, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            1920,
            1080,
            1,
        );
        assert_eq!(camera.width, 1920);
        assert_eq!(camera.height, 1080);
        assert_eq!(camera.fov_degrees, 60.0);
        assert_eq!(camera.subdivisions, 1);
    }

    #[test]
    fn test_generate_rays() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            90.0,
            10,
            5,
            1,
        );

        let rays = camera.generate_rays();
        assert_eq!(rays.len(), 5); // height
        assert_eq!(rays[0].len(), 10); // width
        assert_eq!(rays[0][0].len(), 1); // samples (subdivisions=1)

        // Check that all rays originate from camera position
        for row in &rays {
            for pixel_samples in row {
                for ray in pixel_samples {
                    assert_eq!(ray.origin, camera.position);
                    // Direction should be normalized
                    let len = ray.direction.length();
                    assert!((len - 1.0).abs() < 1e-5);
                }
            }
        }
    }
}
