//! Raytracer module with geometric types, camera, mesh primitives, and rendering utilities.

pub mod camera;
pub mod image;
pub mod light;
pub mod material;
pub mod mesh;
pub mod raytracer;
pub mod sphere;
pub mod vector;

use crate::raytracer::material::Material;
use crate::raytracer::vector::{Float, Vec3};

/// A ray in 3D space, defined by an origin point and a direction vector.
#[derive(Copy, Clone, Debug)]
pub struct Ray {
    /// Origin point of the ray
    pub origin: Vec3,
    /// Direction vector of the ray (should be normalized)
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray with origin and direction.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at parameter t.
    /// point(t) = origin + t * direction
    pub fn at(&self, t: Float) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Surface intersection information returned by ray-surface intersections.
#[derive(Copy, Clone, Debug)]
pub struct Intersection {
    /// Distance from ray origin to intersection point
    pub t: Float,
    /// Position of the intersection point
    pub point: Vec3,
    /// Surface normal at the intersection point
    pub normal: Vec3,
    /// Material at the intersection point
    pub material: Material,
}

impl Intersection {
    /// Create a new intersection.
    pub fn new(t: Float, point: Vec3, normal: Vec3, material: Material) -> Self {
        Self {
            t,
            point,
            normal,
            material,
        }
    }
}

/// A branched ray generated from a ray-surface interaction.
/// Represents one of potentially multiple scattered/reflected/transmitted rays.
#[derive(Copy, Clone, Debug)]
pub struct BranchedRay {
    /// The new ray after interaction
    pub ray: Ray,
    /// Weight of this branch (reflection/transmission rate)
    pub weight: Float,
    /// Material the ray is passing through after the interaction
    pub passing_material: Material,
}

/// Trait for ray-surface intersection detection.
/// Any geometry that can be intersected by rays should implement this trait.
pub trait Surface {
    /// Calculate ray-surface intersection.
    /// Returns the closest intersection if the ray hits this surface, None otherwise.
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;

    /// Get the material of this surface.
    fn material(&self) -> Material;
}

// Implement Surface for references to trait objects
impl<'a> Surface for &'a (dyn Surface + 'a) {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        (*self).intersect(ray)
    }

    fn material(&self) -> Material {
        (*self).material()
    }
}
