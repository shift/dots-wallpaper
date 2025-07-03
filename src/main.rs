use image::{imageops, io::Reader as ImageReader, RgbImage};
use std::env;
use std::error::Error;
use std::process;

/// Creates a composite wallpaper by combining angled strips from multiple images without distortion.
///
/// This function operates by first resizing each source image to the full target resolution.
/// It then iterates through each pixel of the destination canvas. For each pixel, it uses
/// the angle to determine which of the pre-resized source images should be visible at that
/// location (acting like a mask). It then copies the pixel from the chosen source image
/// directly, ensuring the source images are never warped or sheared.
///
/// # Arguments
///
/// * `output_path` - The path to save the final generated wallpaper.
/// * `resolution` - A tuple `(width, height)` for the output wallpaper.
/// * `angle_degrees` - The angle of the dividing slices in degrees. 0 is vertical.
/// * `wallpaper_paths` - A slice of strings representing the paths to the input images.
///
/// # Returns
///
/// A `Result` which is `Ok(())` on success or a `Box<dyn Error>` on failure.
fn create_angled_strip_wallpaper(
    output_path: &str,
    resolution: (u32, u32),
    angle_degrees: f32,
    wallpaper_paths: &[String],
) -> Result<(), Box<dyn Error>> {
    let (width, height) = resolution;

    // --- Step 1: Load and Resize All Images to Full Target Resolution ---

    let resized_images: Vec<RgbImage> = wallpaper_paths
        .iter()
        .filter_map(|path| {
            println!("Loading and resizing: {}", path);

            // FIX: Use a nested match to handle different error types explicitly.
            // This correctly separates the `io::Error` from `ImageReader::open`
            // from the `ImageError` that can occur during decoding.
            match ImageReader::open(path) {
                Ok(reader) => {
                    match reader.with_guessed_format() {
                        Ok(guessed_reader) => match guessed_reader.decode() {
                            Ok(img) => {
                                // On success, convert to RGB and resize.
                                let rgb_img = img.to_rgb8();
                                Some(imageops::resize(
                                    &rgb_img,
                                    width,
                                    height,
                                    imageops::FilterType::Lanczos3,
                                ))
                            }
                            Err(e) => {
                                eprintln!("Warning: Skipping {} due to a decode error: {}", path, e);
                                None
                            }
                        },
                        Err(e) => {
                            eprintln!("Warning: Skipping {} due to a format error: {}", path, e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Skipping {} due to an IO error: {}", path, e);
                    None
                }
            }
        })
        .collect();

    let num_users = resized_images.len();

    // --- Edge Case Handling ---

    if num_users == 0 {
        println!("No valid wallpapers provided, creating a black image.");
        RgbImage::new(width, height).save(output_path)?;
        return Ok(());
    }

    if num_users == 1 {
        println!("Only one wallpaper provided, saving it directly.");
        resized_images[0].save(output_path)?;
        println!("Wallpaper successfully saved to {}", output_path);
        return Ok(());
    }

    // --- Step 2: Composite the Pre-Resized Images Using an Angled Mask ---

    let theta = angle_degrees.to_radians();
    let tan_theta = theta.tan();

    let p1 = 0.0 - 0.0 * tan_theta; // Top-left
    let p2 = (width - 1) as f32 - 0.0 * tan_theta; // Top-right
    let p3 = 0.0 - (height - 1) as f32 * tan_theta; // Bottom-left
    let p4 = (width - 1) as f32 - (height - 1) as f32 * tan_theta; // Bottom-right

    let min_skewed_x = p1.min(p2).min(p3).min(p4);
    let max_skewed_x = p1.max(p2).max(p3).max(p4);
    let skewed_range = max_skewed_x - min_skewed_x;

    let mut canvas = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let skewed_x = x as f32 - y as f32 * tan_theta;
            let normalized_progress = (skewed_x - min_skewed_x) / skewed_range;
            let image_index_float = normalized_progress * num_users as f32;
            let image_index = (image_index_float.floor() as usize).min(num_users - 1);

            let source_image = &resized_images[image_index];
            let pixel = source_image.get_pixel(x, y);
            canvas.put_pixel(x, y, *pixel);
        }
    }

    canvas.save(output_path)?;
    println!("Angled wallpaper successfully saved to {}", output_path);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} <output_path> <width>x<height> <angle_degrees> [wallpaper_path1] ...", args[0]);
        eprintln!("Example: {} ./output.png 1920x1080 20 ./img1.jpg ./img2.jpg", args[0]);
        process::exit(1);
    }

    let output_arg = &args[1];
    let resolution_parts: Vec<&str> = args[2].split('x').collect();
    if resolution_parts.len() != 2 {
        eprintln!("Error: Resolution must be in the format <width>x<height>");
        process::exit(1);
    }

    let width = resolution_parts[0].parse::<u32>().unwrap_or_else(|_| {
        eprintln!("Error: Invalid width provided.");
        process::exit(1);
    });
    let height = resolution_parts[1].parse::<u32>().unwrap_or_else(|_| {
        eprintln!("Error: Invalid height provided.");
        process::exit(1);
    });
    let resolution_arg = (width, height);

    let angle_arg = args[3].parse::<f32>().unwrap_or_else(|_| {
        eprintln!("Error: Invalid angle provided. Must be a number.");
        process::exit(1);
    });

    let paths_arg = &args[4..];

    if let Err(e) = create_angled_strip_wallpaper(output_arg, resolution_arg, angle_arg, paths_arg) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}


// This module is only compiled when running `cargo test`
#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Helper function to create a solid color image for testing.
    fn create_dummy_image(path: &PathBuf, width: u32, height: u32, color: Rgb<u8>) {
        // Explicitly save as PNG for testing purposes. The main code can handle it.
        let mut img = RgbImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = color;
        }
        img.save_with_format(path, image::ImageFormat::Png).unwrap();
    }
    
    // Helper to create an image with a feature (a cross) to test for warping.
    fn create_feature_image(path: &PathBuf, width: u32, height: u32, bg: Rgb<u8>, fg: Rgb<u8>) {
        let mut img = RgbImage::from_pixel(width, height, bg);
        let center_x = width / 2;
        let center_y = height / 2;
        // Draw a vertical line
        for y in 0..height {
            img.put_pixel(center_x, y, fg);
        }
        // Draw a horizontal line
        for x in 0..width {
            img.put_pixel(x, center_y, fg);
        }
        img.save_with_format(path, image::ImageFormat::Png).unwrap();
    }


    #[test]
    fn test_no_images_creates_black_canvas() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &[]).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
        assert_eq!(*output_img.get_pixel(50, 50), Rgb([0, 0, 0]));
    }

    #[test]
    fn test_single_image_saves_directly() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input"); // No extension
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();
        
        let red = Rgb([255, 0, 0]);
        create_dummy_image(&input_path, 200, 200, red);

        let paths = vec![input_path.to_str().unwrap().to_string()];
        create_angled_strip_wallpaper(output_path_str, (100, 50), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 50));
        assert_eq!(*output_img.get_pixel(0, 0), red);
    }

    #[test]
    fn test_vertical_strips_with_zero_angle() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        let red = Rgb([255, 0, 0]);
        let green = Rgb([0, 255, 0]);
        let path1 = dir.path().join("red");
        let path2 = dir.path().join("green");
        create_dummy_image(&path1, 100, 100, red);
        create_dummy_image(&path2, 100, 100, green);

        let paths = vec![
            path1.to_str().unwrap().to_string(),
            path2.to_str().unwrap().to_string(),
        ];
        
        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
        assert_eq!(*output_img.get_pixel(25, 50), red); // Left side
        assert_eq!(*output_img.get_pixel(75, 50), green); // Right side
    }

    #[test]
    fn test_no_warping_with_angled_strips() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        let bg = Rgb([255, 255, 255]); // white
        let fg = Rgb([0, 0, 0]); // black
        let path1 = dir.path().join("feature_image");
        let path2 = dir.path().join("solid_image");
        create_feature_image(&path1, 100, 100, bg, fg);
        create_dummy_image(&path2, 100, 100, Rgb([255,0,0]));

        let paths = vec![
            path1.to_str().unwrap().to_string(),
            path2.to_str().unwrap().to_string(),
        ];
        
        create_angled_strip_wallpaper(output_path_str, (100, 100), 45.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        
        assert_eq!(*output_img.get_pixel(50, 75), fg);
        assert_eq!(*output_img.get_pixel(51, 75), bg);
        assert_eq!(*output_img.get_pixel(25, 50), fg);
        assert_eq!(*output_img.get_pixel(25, 51), bg);
    }
}

