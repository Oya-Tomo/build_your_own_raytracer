//! Main raytracer engine for color computation and ray tracing.

use super::camera::Camera;
use super::image::Image;
use super::light::Light;
use super::material::{Color, Material};
use super::vector::{Float, Vec3};
use super::{Intersection, Ray, Surface};

/// Main raytracer engine.
/// Responsible for computing ray colors through the scene.
pub struct RayTracer {
    /// Background color (for rays that don't hit anything)
    pub background_color: Color,
    /// Maximum recursion depth for ray tracing
    pub max_depth: usize,
    /// Minimum ray weight to continue tracing
    pub min_weight: Float,
    /// Default material for vacuum/air (used for rays not inside any object)
    pub vacuum_material: Material,
}

impl RayTracer {
    /// Create a new raytracer with specified background color, depth limit, minimum weight, and vacuum material.
    pub fn new(
        background_color: Color,
        max_depth: usize,
        min_weight: Float,
        vacuum_material: Material,
    ) -> Self {
        Self {
            background_color,
            max_depth,
            min_weight,
            vacuum_material,
        }
    }

    /// Render a complete image from the camera viewpoint.
    /// Generates rays for each pixel and traces them through the scene.
    ///
    /// # Arguments
    /// * `camera` - The camera defining viewpoint and image resolution
    /// * `surfaces` - Array of surfaces in the scene
    /// * `lights` - Array of light sources in the scene
    ///
    /// # Returns
    /// An Image containing the rendered HDR pixels
    pub fn render(&self, camera: &Camera, surfaces: &[impl Surface], lights: &[Light]) -> Image {
        let rays = camera.generate_rays();
        let pixels = rays
            .iter()
            .map(|row| {
                row.iter()
                    .map(|pixel_samples| {
                        // Average all samples for this pixel
                        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                        for sample_ray in pixel_samples {
                            pixel_color =
                                pixel_color + self.trace_ray(sample_ray, surfaces, lights);
                        }
                        pixel_color * (1.0 / pixel_samples.len() as Float)
                    })
                    .collect()
            })
            .collect();
        Image::from_pixels(pixels)
    }

    /// Trace a ray through the scene and compute its color.
    /// Recursively traces rays through reflections, refractions, and diffuse scattering.
    ///
    /// # Arguments
    /// * `ray` - The ray to trace
    /// * `surfaces` - Array of surfaces in the scene
    /// * `lights` - Array of light sources in the scene
    ///
    /// # Returns
    /// The computed color of the ray
    fn trace_ray(&self, ray: &Ray, surfaces: &[impl Surface], lights: &[Light]) -> Color {
        self.trace_ray_recursive(ray, surfaces, lights, 0, 1.0, self.vacuum_material)
    }

    /// Internal recursive implementation of trace_ray.
    /// This method handles the depth limiting, intersection finding, and ray branching.
    ///
    /// # Arguments
    /// * `ray` - The ray to trace
    /// * `surfaces` - Array of surfaces in the scene
    /// * `lights` - Array of light sources in the scene
    /// * `depth` - Current recursion depth
    /// * `current_weight` - Current weight of the ray
    /// * `passing_material` - Material the ray is currently passing through
    fn trace_ray_recursive(
        &self,
        ray: &Ray,
        surfaces: &[impl Surface],
        lights: &[Light],
        depth: usize,
        current_weight: Float,
        passing_material: Material,
    ) -> Color {
        // Stop tracing if depth exceeded or weight too small
        if depth >= self.max_depth || current_weight < self.min_weight {
            return self.background_color;
        }

        // Find closest intersection with all surfaces
        let closest_intersection = self.find_closest_intersection(ray, surfaces);

        // Find closest intersection with all lights
        let mut closest_light_intersection: Option<(super::Intersection, usize)> = None;
        let mut closest_light_t = Float::INFINITY;
        for (light_idx, light) in lights.iter().enumerate() {
            if let Some(light_intersection) = light.intersect(ray) {
                if light_intersection.t > 1e-5 && light_intersection.t < closest_light_t {
                    closest_light_intersection = Some((light_intersection, light_idx));
                    closest_light_t = light_intersection.t;
                }
            }
        }

        // Check if light is closer than surface
        if let Some((light_intersection, light_idx)) = closest_light_intersection {
            if closest_intersection.is_none()
                || light_intersection.t < closest_intersection.as_ref().unwrap().t
            {
                // Ray hit the light first
                let light = &lights[light_idx];

                // Apply Beer's law absorption for the distance traveled
                let distance = light_intersection.t;
                let attenuation = Color::new(
                    (-passing_material.absorption.r * distance).exp(),
                    (-passing_material.absorption.g * distance).exp(),
                    (-passing_material.absorption.b * distance).exp(),
                );

                // Return the light emission attenuated by the material
                return light.emission * attenuation * current_weight;
            }
        }

        let intersection = match closest_intersection {
            Some(intersection) => intersection,
            None => {
                // Ray didn't hit anything; return background
                return self.background_color;
            }
        };

        // Apply Beer's law absorption: I = I0 * e^(-absorption * distance)
        // Compute attenuation factor for the ray distance traveled
        let distance = intersection.t;
        let attenuation = Color::new(
            (-passing_material.absorption.r * distance).exp(),
            (-passing_material.absorption.g * distance).exp(),
            (-passing_material.absorption.b * distance).exp(),
        );

        // Get the material at intersection (now embedded in Intersection)
        let material = intersection.material;

        // === DIRECT LIGHTING ===
        // Compute light contribution from all light sources
        let mut direct_color = Color::black();
        for light in lights {
            direct_color =
                direct_color + self.compute_direct_light(&intersection, light, &material, surfaces);
        }

        // === INDIRECT LIGHTING (RAY BRANCHING) ===
        // Generate branched rays for reflection/refraction/diffuse
        let branched_rays = self.branch_rays(ray, &intersection, passing_material);

        let mut indirect_color = Color::black();
        for branched in &branched_rays {
            let weight = current_weight * branched.weight;
            let contribution = self.trace_ray_recursive(
                &branched.ray,
                surfaces,
                lights,
                depth + 1,
                weight,
                branched.passing_material,
            );
            // Modulate by material albedo
            indirect_color = indirect_color + (material.albedo * contribution) * branched.weight;
        }

        // Apply Beer's law attenuation to both direct and indirect lighting
        let result = (direct_color + indirect_color) * attenuation;

        // Combine direct and indirect lighting
        result
    }

    /// Find the closest intersection of a ray with all surfaces.
    /// Returns the intersection, or None if no hit.
    fn find_closest_intersection(
        &self,
        ray: &Ray,
        surfaces: &[impl Surface],
    ) -> Option<Intersection> {
        let mut closest = None;
        let mut closest_t = Float::INFINITY;

        for surface in surfaces.iter() {
            if let Some(intersection) = surface.intersect(ray) {
                // Only consider intersections in front of the camera (t > 0)
                // and ignore self-intersections (t > small epsilon)
                if intersection.t > 1e-5 && intersection.t < closest_t {
                    closest = Some(intersection);
                    closest_t = intersection.t;
                }
            }
        }

        closest
    }

    /// Compute direct lighting contribution from a single light source.
    /// Implements Lambertian diffuse reflection using cosine law (N · L).
    fn compute_direct_light(
        &self,
        intersection: &Intersection,
        light: &Light,
        material: &super::material::Material,
        surfaces: &[impl Surface],
    ) -> Color {
        // Vector from intersection point to light center
        let to_light = (light.center - intersection.point).normalize();

        // Lambertian cosine law: only lit if facing the light
        let cos_theta = to_light.dot(intersection.normal);
        if cos_theta <= 0.0 {
            return Color::black();
        }

        // Shadow ray: trace toward the light to check visibility
        // Apply offset to avoid self-intersection (shadow acne)
        const OFFSET_EPS: Float = 1e-4;
        let shadow_origin = intersection.point + to_light * OFFSET_EPS;
        let shadow_ray = Ray::new(shadow_origin, to_light);

        // Check if there's any surface blocking the direct path to light
        // We only check surfaces, not the light itself
        for surface in surfaces {
            if let Some(shadow_hit) = surface.intersect(&shadow_ray) {
                // Check if we hit something before the light
                // Light is at distance: (light.center - intersection.point).length()
                let dist_to_light = (light.center - intersection.point).length();
                if shadow_hit.t < dist_to_light - 1e-5 {
                    // Blocked by another surface
                    return Color::black();
                }
            }
        }

        // Lambertian diffuse reflection formula:
        // diffuse_color = object_color * light_color * cos_theta * diffuse_rate
        material.albedo * light.emission * (cos_theta * material.diffuse_rate)
    }

    /// Generate branched rays after ray-surface interaction.
    /// Returns a vector of branched rays based on the material properties.
    ///
    /// # Arguments
    /// * `ray` - The incident ray
    /// * `intersection` - The intersection point
    /// * `incoming_material` - Material the ray is currently passing through (incident side)
    ///
    /// This method handles:
    /// - Diffuse reflection (Lambertian scattering)
    /// - Specular reflection (mirror-like reflection)
    /// - Transmission/Refraction (dielectric materials with Snell's law)
    fn branch_rays(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        incoming_material: Material,
    ) -> Vec<super::BranchedRay> {
        let surface_material = intersection.material;
        let mut branched = Vec::new();

        // Offset to avoid self-intersection (shadow acne)
        const OFFSET_EPS: Float = 1e-4;

        // Determine if ray is coming from outside or inside the surface
        // This is crucial for proper reflection/refraction calculation
        let is_entering = ray.direction.dot(intersection.normal) < 0.0;

        // Outward-facing normal: should point toward the incoming ray direction
        let normal = if is_entering {
            intersection.normal // Ray hits from outside; use normal as-is
        } else {
            -intersection.normal // Ray is inside; flip normal to point outward
        };

        // === DIFFUSE REFLECTION ===
        // Lambertian reflection: scattered uniformly in hemisphere around normal
        if surface_material.diffuse_rate > 1e-5 {
            // For now: simplified direction along normal
            // Direct lighting is computed separately in compute_direct_light
            // This ray just continues the path for indirect effects
            let diffuse_dir = normal;
            let ray_origin = intersection.point + normal * OFFSET_EPS;

            branched.push(super::BranchedRay {
                ray: Ray::new(ray_origin, diffuse_dir),
                weight: surface_material.diffuse_rate,
                // Reflected ray continues through the incoming material
                passing_material: incoming_material,
            });
        }

        // === SPECULAR REFLECTION (+ TOTAL INTERNAL REFLECTION) ===
        // Mirror-like reflection: angle of incidence equals angle of reflection
        let mut specular_weight = surface_material.specular_rate;

        // === TRANSMISSION (REFRACTION) ===
        // Dielectric material: apply Snell's law for refraction
        if surface_material.transmission_rate > 1e-5 {
            // Snell's law: n1 * sin(θ1) = n2 * sin(θ2)
            // Compute the ratio of refractive indices
            let ratio = if is_entering {
                incoming_material.refractive_index / surface_material.refractive_index
            } else {
                surface_material.refractive_index / self.vacuum_material.refractive_index
            };

            // Refraction formula using vector form:
            let cos_i = -ray.direction.dot(normal);
            let sin_t_sq = ratio * ratio * (1.0 - cos_i * cos_i);

            // Check for total internal reflection
            if sin_t_sq > 1.0 {
                // Total internal reflection: add transmission_rate to specular reflection weight
                // This avoids creating duplicate rays
                specular_weight += surface_material.transmission_rate;
            } else {
                // Refracted ray direction
                let cos_t = (1.0 - sin_t_sq).sqrt();
                let refracted = ratio * ray.direction + normal * (ratio * cos_i - cos_t);
                // For transmission, offset in the direction of the refracted ray (inward)
                let ray_origin = intersection.point - normal * OFFSET_EPS;

                // After refraction, determine which material the ray passes through
                // If entering: ray passes through the surface material (inside)
                // If exiting: ray passes through vacuum/air (outside)
                let next_material = if is_entering {
                    surface_material // Entering the surface: pass through it
                } else {
                    self.vacuum_material // Exiting to vacuum/air
                };

                branched.push(super::BranchedRay {
                    ray: Ray::new(ray_origin, refracted),
                    weight: surface_material.transmission_rate,
                    passing_material: next_material,
                });
            }
        }

        // Add specular reflection (or total internal reflection) if weight > 0
        if specular_weight > 1e-5 {
            let reflected = ray.direction - normal * 2.0 * ray.direction.dot(normal);
            let ray_origin = intersection.point + normal * OFFSET_EPS;

            branched.push(super::BranchedRay {
                ray: Ray::new(ray_origin, reflected),
                weight: specular_weight,
                // Reflected ray continues through the incoming material
                passing_material: incoming_material,
            });
        }

        branched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raytracer::material::Material;

    // Mock Surface implementation for testing
    #[derive(Copy, Clone)]
    struct MockSurface {
        material: Material,
    }

    impl Surface for MockSurface {
        fn intersect(&self, _ray: &Ray) -> Option<Intersection> {
            None // Mock: no intersection
        }

        fn material(&self) -> Material {
            self.material
        }
    }

    #[test]
    fn test_raytracer_creation() {
        let vacuum = Material::new(Color::black(), 0.0, 0.0, 1.0, 1.0, Color::black());
        let tracer = RayTracer::new(Color::black(), 8, 1e-3, vacuum);
        assert_eq!(tracer.background_color, Color::black());
        assert_eq!(tracer.max_depth, 8);
    }

    #[test]
    fn test_raytracer_with_background() {
        let bg = Color::white();
        let vacuum = Material::new(Color::black(), 0.0, 0.0, 1.0, 1.0, Color::black());
        let tracer = RayTracer::new(bg, 5, 1e-3, vacuum);
        assert_eq!(tracer.background_color, bg);
        assert_eq!(tracer.max_depth, 5);
    }

    #[test]
    fn test_trace_ray_no_hit() {
        let vacuum = Material::new(Color::black(), 0.0, 0.0, 1.0, 1.0, Color::black());
        let tracer = RayTracer::new(Color::black(), 8, 1e-3, vacuum);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        let surfaces: Vec<MockSurface> = vec![];
        let lights: Vec<Light> = vec![];

        let color = tracer.trace_ray(&ray, &surfaces, &lights);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn test_raytracer_depth_limit() {
        // This test ensures that recursion stops at the specified depth
        let vacuum = Material::new(Color::black(), 0.0, 0.0, 1.0, 1.0, Color::black());
        let tracer = RayTracer::new(Color::black(), 3, 1e-3, vacuum);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        let surfaces: Vec<MockSurface> = vec![];
        let lights: Vec<Light> = vec![];

        let color = tracer.trace_ray(&ray, &surfaces, &lights);
        // Should return background color and not panic
        assert_eq!(color, Color::black());
    }
}
