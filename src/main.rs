mod raytracer;

use raytracer::camera::Camera;
use raytracer::light::Light;
use raytracer::material::{Color, Material};
use raytracer::mesh::Triangle;
use raytracer::raytracer::RayTracer;
use raytracer::sphere::Sphere;
use raytracer::vector::Vec3;

use crate::raytracer::image::ACESFilmic;
use crate::raytracer::vector::Float;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

fn main() {
    let fps: Float = 60.0;
    let frames: usize = fps as usize * 5;

    // Get number of available CPU cores
    let num_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    // Limit to at most 4 concurrent threads
    let max_concurrent_threads = std::cmp::min(16, num_cores);

    println!(
        "Starting multi-threaded rendering with {} available CPU cores",
        num_cores
    );
    println!(
        "Limiting to {} concurrent rendering threads",
        max_concurrent_threads
    );

    let mut handles: Vec<(usize, thread::JoinHandle<()>, Arc<AtomicBool>)> = vec![];
    let mut next_frame = 0;

    while next_frame < frames || !handles.is_empty() {
        // Scan all handles and remove finished threads
        handles.retain_mut(|(frame_idx, _handle, is_done)| {
            if is_done.load(Ordering::Relaxed) {
                println!("Frame {} completed", frame_idx);
                false // Remove this handle
            } else {
                true // Keep this handle
            }
        });

        // If we have capacity and frames left to render, spawn a new thread
        if next_frame < frames && handles.len() < max_concurrent_threads {
            let f = next_frame;
            let is_done = Arc::new(AtomicBool::new(false));
            let is_done_clone = Arc::clone(&is_done);

            let handle = thread::spawn(move || {
                let time = f as Float / fps;
                frame(time, &format!("output/frame_{:03}.png", f));
                is_done_clone.store(true, Ordering::Relaxed);
            });
            handles.push((f, handle, is_done));
            next_frame += 1;
        } else if !handles.is_empty() {
            // If no capacity and frames remain, wait a tiny bit before checking again
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    println!("All frames rendered!");
}

fn frame(time: Float, filename: &str) {
    // === CAMERA SETUP ===
    let camera = Camera::new(
        Vec3::new(0.0, -3.0, 3.0), // eye position
        Vec3::new(0.0, 3.0, -2.0), // forward direction
        Vec3::new(0.0, 0.0, 1.0),  // up direction
        60.0,                      // field of view (degrees)
        1920,                      // image width
        1080,                      // image height
        4,                         // subdivisions for anti-aliasing
    );

    // === SCENE ===
    let mirror = Material::mirror(Color::new(0.05, 0.05, 0.05), 0.9);
    let red_glass = Material::new(
        Color::new(0.3, 0.0, 0.0),
        0.0,
        0.05,
        0.9,
        1.5,
        Color::new(0.0, 1.5, 1.5),
    );
    let green_glass = Material::new(
        Color::new(0.0, 0.3, 0.0),
        0.0,
        0.05,
        0.9,
        1.5,
        Color::new(1.5, 0.0, 1.5),
    );
    let blue_glass = Material::new(
        Color::new(0.0, 0.0, 0.3),
        0.0,
        0.05,
        0.9,
        1.5,
        Color::new(1.5, 1.5, 0.0),
    );
    let yellow_matte = Material::new(
        Color::new(1.0, 1.0, 1.0),
        0.2,
        0.6,
        0.2,
        1.0,
        Color::new(0.0, 0.0, 0.0),
    );

    let x = (time * 2.0).sin() * 1.5;

    let sphere1 = Sphere::new(Vec3::new(x, 1.5, 0.7), 0.7, mirror);
    let sphere2 = Sphere::new(Vec3::new(0.0, 0.0, 0.5), 0.5, red_glass);
    let sphere3 = Sphere::new(Vec3::new(-1.2, 0.0, 0.5), 0.5, blue_glass);
    let sphere4 = Sphere::new(Vec3::new(1.2, 0.0, 0.5), 0.5, green_glass);

    let triangle1 = Triangle::new(
        Vec3::new(3.0, 3.0, 0.0),
        Vec3::new(-3.0, -1.0, 0.0),
        Vec3::new(3.0, -1.0, 0.0),
        yellow_matte,
    );
    let triangle2 = Triangle::new(
        Vec3::new(3.0, 3.0, 0.0),
        Vec3::new(-3.0, 3.0, 0.0),
        Vec3::new(-3.0, -1.0, 0.0),
        yellow_matte,
    );

    // Use trait objects to store mixed geometry types
    let surfaces: Vec<&dyn raytracer::Surface> = vec![
        &sphere1, &sphere2, &sphere3, &sphere4, &triangle1, &triangle2,
    ];

    // === LIGHTING SETUP ===
    let light1 = Light::new(Vec3::new(3.0, -3.0, 5.0), 3.0, Color::new(1.0, 1.0, 1.0));
    let light2 = Light::new(Vec3::new(0.0, 0.0, 10.0), 2.0, Color::new(1.0, 1.0, 1.0));
    let light3 = Light::new(Vec3::new(-10.0, -5.0, 5.0), 2.0, Color::new(1.0, 1.0, 1.0)); // Top light

    let lights = [light1, light2, light3];

    // === RAYTRACER SETUP ===
    let vacuum = Material::new(Color::black(), 0.0, 0.0, 1.0, 1.0, Color::black());
    let raytracer = RayTracer::new(
        Color::new(0.0, 0.0, 0.0), // background color (darker blue)
        16,                        // max depth
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
    save_image_to_file(&rgb8_data, image.width, image.height, filename)
        .expect("Failed to save image");

    println!("Rendering complete. Image saved to {}", filename);
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
