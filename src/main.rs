mod raytracer;

use raytracer::camera::Camera;
use raytracer::light::Light;
use raytracer::material::{Color, Material};
use raytracer::mesh::Triangle;
use raytracer::raytracer::RayTracer;
use raytracer::sphere::Sphere;
use raytracer::vector::Vec3;

use crate::raytracer::image::ACESFilmic;

fn main() {
    // === CAMERA SETUP ===
    let camera = Camera::new(
        Vec3::new(0.0, -2.0, 2.0), // eye position
        Vec3::new(0.0, 1.0, -1.0), // forward direction
        Vec3::new(0.0, 0.0, 1.0),  // up direction
        90.0,                      // field of view (degrees)
        1920,                      // image width
        1080,                      // image height
        1,                         // subdivisions for anti-aliasing
    );

    // === SCENE ===
    let mirror = Material::mirror(Color::white(), 0.9);
    let red_glass = Material::new(
        Color::new(0.7, 0.2, 0.2),
        0.0,
        0.1,
        0.9,
        1.5,
        Color::new(0.0, 0.01, 0.01),
    );
    let green_glass = Material::new(
        Color::new(0.2, 0.7, 0.2),
        0.0,
        0.1,
        0.9,
        1.5,
        Color::new(0.01, 0.0, 0.01),
    );
    let blue_glass = Material::new(
        Color::new(0.2, 0.2, 0.7),
        0.0,
        0.1,
        0.9,
        1.5,
        Color::new(0.01, 0.01, 0.0),
    );
    let yellow_matte = Material::new(
        Color::new(1.0, 1.0, 1.0),
        0.2,
        0.6,
        0.2,
        1.0,
        Color::new(0.0, 0.0, 0.0),
    );

    let sphere1 = Sphere::new(Vec3::new(-0.5, 1.5, 0.7), 0.7, mirror);
    let sphere2 = Sphere::new(Vec3::new(0.0, 0.0, 0.5), 0.5, red_glass);
    let sphere3 = Sphere::new(Vec3::new(-1.2, 0.0, 0.5), 0.5, blue_glass);
    let sphere4 = Sphere::new(Vec3::new(1.2, 0.0, 0.5), 0.5, green_glass);

    let triangle1 = Triangle::new(
        Vec3::new(3.0, 3.0, 0.0),
        Vec3::new(3.0, -1.0, 0.0),
        Vec3::new(-3.0, -1.0, 0.0),
        yellow_matte,
    );
    let triangle2 = Triangle::new(
        Vec3::new(3.0, 3.0, 0.0),
        Vec3::new(-3.0, -1.0, 0.0),
        Vec3::new(-3.0, 3.0, 0.0),
        yellow_matte,
    );

    // Use trait objects to store mixed geometry types
    let surfaces: Vec<&dyn raytracer::Surface> = vec![
        &sphere1, &sphere2, &sphere3, &sphere4, &triangle1, &triangle2,
    ];

    // === LIGHTING SETUP ===
    let light1 = Light::new(Vec3::new(3.0, -3.0, 5.0), 3.0, Color::new(5.0, 5.0, 5.0));
    let light2 = Light::new(Vec3::new(0.0, 0.0, 10.0), 2.0, Color::new(5.8, 5.8, 5.0));
    let light3 = Light::new(Vec3::new(-10.0, -5.0, 5.0), 2.0, Color::new(9.0, 10.0, 9.5)); // Top light

    let lights = [light1, light2, light3];

    // === RAYTRACER SETUP ===
    let vacuum = Material::new(Color::black(), 0.0, 0.0, 1.0, 1.0, Color::black());
    let raytracer = RayTracer::new(
        Color::new(0.0, 0.0, 0.0), // background color (darker blue)
        8,                         // max depth
        1e-3,                      // min weight
        vacuum,
    );

    // === RENDERING ===
    println!(
        "Rendering scene with {} surfaces and {} lights...",
        surfaces.len(),
        lights.len()
    );
    let image = raytracer.render(&camera, &surfaces[..], &lights);
    println!("Render complete!");

    // === TONE MAPPING ===
    let tone_mapper = ACESFilmic::new();
    let rgb8_data = image.convert(&tone_mapper);
    println!("Tone mapping complete!");

    // === OUTPUT INFO ===
    println!(
        "Image size: {}x{} ({} pixels)",
        image.width,
        image.height,
        rgb8_data.len()
    );
    println!("Sample pixel values (first 5):");
    for (i, pixel) in rgb8_data.iter().take(5).enumerate() {
        println!("  Pixel {}: RGB({}, {}, {})", i, pixel.0, pixel.1, pixel.2);
    }

    // === SAVE TO FILE ===
    save_image_to_file(&rgb8_data, image.width, image.height, "output.png")
        .expect("Failed to save image");

    println!("Rendering complete. Image saved to output.png");
}

/// Convert RGB8 pixel data to an image and save to a PNG file
fn save_image_to_file(
    rgb8_data: &[(u8, u8, u8)],
    width: usize,
    height: usize,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert RGB tuples to flat byte array (RGBA format for the image crate)
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    for (r, g, b) in rgb8_data {
        rgba_data.push(*r);
        rgba_data.push(*g);
        rgba_data.push(*b);
        rgba_data.push(255); // Alpha channel (fully opaque)
    }

    // Create image buffer
    let imgbuf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
        width as u32,
        height as u32,
        rgba_data,
    )
    .ok_or("Failed to create image buffer")?;

    // Save as PNG
    imgbuf.save(filename)?;
    println!("Image saved to: {}", filename);

    Ok(())
}
