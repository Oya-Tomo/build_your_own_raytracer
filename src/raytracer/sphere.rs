//! Sphere primitive for the raytracer.

use super::material::Material;
use super::vector::{Float, Vec3};
use super::{Intersection, Ray, Surface};

/// A sphere defined by a center position and radius.
#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    /// Center position of the sphere
    pub center: Vec3,
    /// Radius of the sphere
    pub radius: Float,
    /// Material of the sphere
    pub material: Material,
}

impl Sphere {
    /// Create a new sphere with material.
    pub fn new(center: Vec3, radius: Float, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    /// Calculate the surface normal at a given point on the sphere.
    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        (point - self.center).normalize()
    }

    /// Check if a point is inside the sphere.
    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// Get the surface area of the sphere.
    pub fn surface_area(&self) -> Float {
        4.0 * std::f32::consts::PI * self.radius * self.radius
    }

    /// Get the volume of the sphere.
    pub fn volume(&self) -> Float {
        4.0 / 3.0 * std::f32::consts::PI * self.radius * self.radius * self.radius
    }
}

impl Surface for Sphere {
    /// Calculate ray-sphere intersection.
    /// Returns the closest intersection if the ray hits this sphere, None otherwise.
    /// Uses the quadratic formula to solve: ||O + t*D - C||^2 = r^2
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let oc = ray.origin - self.center;
        let d = ray.direction;

        // Coefficients of the quadratic equation: a*t^2 + b*t + c = 0
        let a = d.dot(d);
        let b = 2.0 * oc.dot(d);
        let c = oc.dot(oc) - self.radius * self.radius;

        // Discriminant
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            // No intersection
            return None;
        }

        // Two solutions: t1 and t2
        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        // Find the closest intersection in front of the camera (t > 0)
        let t = if t1 > 0.0 {
            t1
        } else if t2 > 0.0 {
            t2
        } else {
            return None;
        };

        let point = ray.at(t);
        let normal = self.normal_at(point);

        Some(Intersection::new(t, point, normal, self.material))
    }

    /// Get the material of this sphere.
    fn material(&self) -> Material {
        self.material
    }
}

#[cfg(test)]
mod tests {
    use super::super::material::Color;
    use super::*;

    #[test]
    fn test_sphere_creation() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0, material);
        assert_eq!(sphere.center, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(sphere.radius, 1.0);
    }

    #[test]
    fn test_sphere_intersect_direct_hit() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 5.0), 1.0, material);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        if let Some(intersection) = sphere.intersect(&ray) {
            assert!((intersection.t - 4.0).abs() < 1e-5); // Should hit at t ≈ 4.0
            assert_eq!(intersection.point, Vec3::new(0.0, 0.0, 4.0));
        } else {
            panic!("Expected intersection");
        }
    }

    #[test]
    fn test_sphere_no_intersect() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 5.0), 1.0, material);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

        assert!(sphere.intersect(&ray).is_none());
    }

    #[test]
    fn test_sphere_normal_at() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0, material);
        let point = Vec3::new(1.0, 0.0, 0.0);
        let normal = sphere.normal_at(point);

        assert_eq!(normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_sphere_contains_point() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0, material);

        assert!(sphere.contains_point(Vec3::new(0.0, 0.0, 0.0)));
        assert!(sphere.contains_point(Vec3::new(0.5, 0.0, 0.0)));
        assert!(!sphere.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere_surface_area() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0, material);
        let expected = 4.0 * std::f32::consts::PI;
        assert!((sphere.surface_area() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_sphere_volume() {
        let material = Material::matte(Color::white(), 0.8);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0, material);
        let expected = 4.0 / 3.0 * std::f32::consts::PI;
        assert!((sphere.volume() - expected).abs() < 1e-5);
    }
}
