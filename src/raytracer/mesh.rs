//! Mesh objects and primitives for the raytracer.

use super::material::Material;
use super::vector::{Float, Vec3};
use super::{Intersection, Ray, Surface};

const EPSILON: Float = 1e-8;

/// A triangle defined by three vertices.
/// Used as the fundamental face primitive in mesh objects.
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    /// First vertex position
    pub v0: Vec3,
    /// Second vertex position
    pub v1: Vec3,
    /// Third vertex position
    pub v2: Vec3,
    /// Material of the triangle
    pub material: Material,
}

impl Triangle {
    /// Create a new triangle from three vertices and a material.
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: Material) -> Self {
        Self {
            v0,
            v1,
            v2,
            material,
        }
    }

    /// Calculate the surface normal without normalization (faster if only direction matters).
    pub fn normal_unnormalized(&self) -> Vec3 {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        edge1.cross(edge2)
    }

    /// Calculate the surface normal of the triangle (pointing outward).
    /// Returns a normalized normal vector.
    pub fn normal(&self) -> Vec3 {
        self.normal_unnormalized().normalize()
    }

    /// Calculate the area of the triangle.
    /// Uses: Area = 0.5 * ||(v1 - v0) × (v2 - v0)||
    pub fn area(&self) -> Float {
        self.normal_unnormalized().length() * 0.5
    }

    /// Calculate the centroid (geometric center) of the triangle.
    pub fn centroid(&self) -> Vec3 {
        (self.v0 + self.v1 + self.v2) * (1.0 / 3.0)
    }

    /// Check if a point is inside the triangle using barycentric coordinates.
    /// Returns true if the point lies on the triangle plane and within the triangle bounds.
    pub fn contains_point(&self, point: Vec3) -> bool {
        let v0 = self.v0;
        let v1 = self.v1;
        let v2 = self.v2;

        let edge0 = v1 - v0;
        let edge1 = v2 - v1;
        let edge2 = v0 - v2;

        let c0 = point - v0;
        let c1 = point - v1;
        let c2 = point - v2;

        let normal = self.normal();

        let d0 = edge0.cross(c0).dot(normal);
        let d1 = edge1.cross(c1).dot(normal);
        let d2 = edge2.cross(c2).dot(normal);

        // All should have the same sign for point to be inside
        (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
    }

    /// Get the bounding box (AABB) of the triangle.
    /// Returns (min_point, max_point).
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let min_x = self.v0.x.min(self.v1.x).min(self.v2.x);
        let min_y = self.v0.y.min(self.v1.y).min(self.v2.y);
        let min_z = self.v0.z.min(self.v1.z).min(self.v2.z);

        let max_x = self.v0.x.max(self.v1.x).max(self.v2.x);
        let max_y = self.v0.y.max(self.v1.y).max(self.v2.y);
        let max_z = self.v0.z.max(self.v1.z).max(self.v2.z);

        (
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        )
    }
}

impl Surface for Triangle {
    /// Calculate ray-triangle intersection using the Möller-Trumbore algorithm.
    /// Returns the intersection if the ray hits this triangle, None otherwise.
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let ray_cross_edge2 = ray.direction.cross(edge2);
        let det = edge1.dot(ray_cross_edge2);

        // If determinant is near zero, ray lies in the plane of the triangle
        if det.abs() < EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let s = ray.origin - self.v0;
        let u = inv_det * s.dot(ray_cross_edge2);

        // u should be in [0, 1] for intersection
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let s_cross_edge1 = s.cross(edge1);
        let v = inv_det * ray.direction.dot(s_cross_edge1);

        // v should be in [0, 1] and u + v should be <= 1 for intersection
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = inv_det * edge2.dot(s_cross_edge1);

        if t > 0.0 {
            let point = ray.at(t);
            let normal = self.normal();
            Some(Intersection::new(t, point, normal, self.material))
        } else {
            None
        }
    }

    /// Get the material of this triangle.
    fn material(&self) -> Material {
        self.material
    }
}

#[cfg(test)]
mod tests {
    use super::super::material::Color;
    use super::*;

    #[test]
    fn test_triangle_creation() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        assert_eq!(triangle.v0, v0);
        assert_eq!(triangle.v1, v1);
        assert_eq!(triangle.v2, v2);
    }

    #[test]
    fn test_triangle_normal() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        let normal = triangle.normal();
        assert_eq!(normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_triangle_area() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(2.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 2.0, 0.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        let expected = 2.0; // Area of right triangle with legs of length 2
        assert!((triangle.area() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_centroid() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(3.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 3.0, 0.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        let centroid = triangle.centroid();
        let expected = Vec3::new(1.0, 1.0, 0.0);
        assert_eq!(centroid, expected);
    }

    #[test]
    fn test_triangle_intersect() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 5.0);
        let v1 = Vec3::new(1.0, 0.0, 5.0);
        let v2 = Vec3::new(0.0, 1.0, 5.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        // Ray pointing in +Z direction from origin
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));

        if let Some(intersection) = triangle.intersect(&ray) {
            assert!((intersection.t - 5.0).abs() < 1e-5);
        } else {
            panic!("Expected intersection");
        }
    }

    #[test]
    fn test_triangle_no_intersect() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 5.0);
        let v1 = Vec3::new(1.0, 0.0, 5.0);
        let v2 = Vec3::new(0.0, 1.0, 5.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        // Ray pointing in +X direction, should miss the triangle
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

        assert!(triangle.intersect(&ray).is_none());
    }

    #[test]
    fn test_triangle_material() {
        let material = Material::matte(Color::white(), 0.8);
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let triangle = Triangle::new(v0, v1, v2, material);

        let retrieved_material = triangle.material();
        assert_eq!(retrieved_material.albedo, Color::white());
    }
}
