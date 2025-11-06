//! Light sources for the raytracer.

use super::material::Color;
use super::vector::{Float, Vec3};
use super::{Intersection, Ray};

/// A spherical light source.
/// Emits light uniformly in all directions from its surface.
#[derive(Copy, Clone, Debug)]
pub struct Light {
    /// Center position of the light
    pub center: Vec3,
    /// Radius of the light
    pub radius: Float,
    /// Emission color and intensity
    pub emission: Color,
}

impl Light {
    /// Create a new light source.
    pub fn new(center: Vec3, radius: Float, emission: Color) -> Self {
        Self {
            center,
            radius,
            emission,
        }
    }

    /// Calculate the surface normal at a given point on the light sphere.
    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        (point - self.center).normalize()
    }

    /// Check if a point is inside the light sphere.
    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// Get the surface area of the light sphere.
    pub fn surface_area(&self) -> Float {
        4.0 * std::f32::consts::PI * self.radius * self.radius
    }

    /// Get the total luminous flux (power) of the light.
    /// Calculated as: emission_magnitude * surface_area
    pub fn luminous_flux(&self) -> Float {
        let emission_magnitude = (self.emission.r + self.emission.g + self.emission.b) / 3.0;
        emission_magnitude * self.surface_area()
    }

    /// Calculate ray-light intersection.
    /// Returns the intersection if the ray hits this light, None otherwise.
    /// Uses the quadratic formula to solve: ||O + t*D - C||^2 = r^2
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
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

        // Light intersections use a dummy material (lights aren't rendered as surfaces)
        let dummy_material = crate::raytracer::material::Material::new(
            Color::black(),
            0.0,
            0.0,
            0.0,
            1.0,
            Color::black(),
        );
        Some(Intersection::new(t, point, normal, dummy_material))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_creation() {
        let light = Light::new(Vec3::new(0.0, 0.0, 0.0), 1.0, Color::white());
        assert_eq!(light.center, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(light.radius, 1.0);
    }

    #[test]
    fn test_light_intersect_direct_hit() {
        let light = Light::new(Vec3::new(0.0, 0.0, 5.0), 1.0, Color::white());
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        if let Some(intersection) = light.intersect(&ray) {
            assert!((intersection.t - 4.0).abs() < 1e-5); // Should hit at t ≈ 4.0
            assert_eq!(intersection.point, Vec3::new(0.0, 0.0, 4.0));
        } else {
            panic!("Expected intersection");
        }
    }

    #[test]
    fn test_light_no_intersect() {
        let light = Light::new(Vec3::new(0.0, 0.0, 5.0), 1.0, Color::white());
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

        assert!(light.intersect(&ray).is_none());
    }

    #[test]
    fn test_light_normal_at() {
        let light = Light::new(Vec3::new(0.0, 0.0, 0.0), 1.0, Color::white());
        let point = Vec3::new(1.0, 0.0, 0.0);
        let normal = light.normal_at(point);

        assert_eq!(normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_light_contains_point() {
        let light = Light::new(Vec3::new(0.0, 0.0, 0.0), 1.0, Color::white());

        assert!(light.contains_point(Vec3::new(0.0, 0.0, 0.0)));
        assert!(light.contains_point(Vec3::new(0.5, 0.0, 0.0)));
        assert!(!light.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_light_surface_area() {
        let light = Light::new(Vec3::new(0.0, 0.0, 0.0), 1.0, Color::white());
        let expected = 4.0 * std::f32::consts::PI;
        assert!((light.surface_area() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_light_emission() {
        let emission = Color::new(1.0, 1.0, 1.0);
        let light = Light::new(Vec3::new(0.0, 0.0, 0.0), 1.0, emission);
        assert_eq!(light.emission, emission);
    }
}
