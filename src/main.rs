use image::{imageops, ImageReader, RgbImage};
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

    if args.len() < 4 {
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

    let paths_arg = if args.len() > 4 { &args[4..] } else { &[] };

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

    // ===== ROBUSTNESS TESTS =====

    #[test]
    fn test_invalid_image_paths() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Test with non-existent files
        let paths = vec![
            "/non/existent/path.jpg".to_string(),
            "/another/invalid/path.png".to_string(),
        ];

        // Should succeed and create a black image (no valid images)
        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
        assert_eq!(*output_img.get_pixel(50, 50), Rgb([0, 0, 0]));
    }

    #[test]
    fn test_corrupt_image_files() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Create corrupt files
        let corrupt1 = dir.path().join("corrupt1.jpg");
        let corrupt2 = dir.path().join("corrupt2.png");
        let empty_file = dir.path().join("empty.gif");
        let text_file = dir.path().join("notimage.bmp");

        // Write invalid data
        std::fs::write(&corrupt1, b"This is not an image").unwrap();
        std::fs::write(&corrupt2, b"\x89PNG\r\n\x1a\nINVALID").unwrap(); // Invalid PNG header
        std::fs::write(&empty_file, b"").unwrap(); // Empty file
        std::fs::write(&text_file, b"Just some text pretending to be an image").unwrap();

        let paths = vec![
            corrupt1.to_str().unwrap().to_string(),
            corrupt2.to_str().unwrap().to_string(),
            empty_file.to_str().unwrap().to_string(),
            text_file.to_str().unwrap().to_string(),
        ];

        // Should succeed and create a black image (no valid images)
        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
        assert_eq!(*output_img.get_pixel(50, 50), Rgb([0, 0, 0]));
    }

    #[test]
    fn test_major_image_formats() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        let red = Rgb([255, 0, 0]);
        let green = Rgb([0, 255, 0]);
        let blue = Rgb([0, 0, 255]);

        // Create images in different formats
        let png_path = dir.path().join("test.png");
        let jpg_path = dir.path().join("test.jpg"); 
        let bmp_path = dir.path().join("test.bmp");

        // Create test images in different formats
        let img = RgbImage::from_pixel(50, 50, red);
        img.save_with_format(&png_path, image::ImageFormat::Png).unwrap();
        
        let img2 = RgbImage::from_pixel(50, 50, green);
        img2.save_with_format(&jpg_path, image::ImageFormat::Jpeg).unwrap();
        
        let img3 = RgbImage::from_pixel(50, 50, blue);
        img3.save_with_format(&bmp_path, image::ImageFormat::Bmp).unwrap();

        // Note: GIF and TIFF need specific handling for RGB, so we'll test with what we can create
        let paths = vec![
            png_path.to_str().unwrap().to_string(),
            jpg_path.to_str().unwrap().to_string(),
            bmp_path.to_str().unwrap().to_string(),
        ];

        create_angled_strip_wallpaper(output_path_str, (150, 150), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (150, 150));
        // Should have successfully processed at least the first image
        // We don't test exact pixel values due to JPEG compression artifacts
    }

    #[test]
    fn test_edge_case_image_sizes() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Test very small image (1x1)
        let tiny_path = dir.path().join("tiny.png");
        let mut tiny_img = RgbImage::new(1, 1);
        tiny_img.put_pixel(0, 0, Rgb([255, 0, 0]));
        tiny_img.save_with_format(&tiny_path, image::ImageFormat::Png).unwrap();

        // Test non-square image
        let rect_path = dir.path().join("rect.png");
        let rect_img = RgbImage::from_pixel(200, 50, Rgb([0, 255, 0]));
        rect_img.save_with_format(&rect_path, image::ImageFormat::Png).unwrap();

        let paths = vec![
            tiny_path.to_str().unwrap().to_string(),
            rect_path.to_str().unwrap().to_string(),
        ];

        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
    }

    #[test]
    fn test_large_image() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Test reasonably large image (not too large to avoid memory issues in CI)
        let large_path = dir.path().join("large.png");
        let large_img = RgbImage::from_pixel(1000, 1000, Rgb([128, 128, 128]));
        large_img.save_with_format(&large_path, image::ImageFormat::Png).unwrap();

        let paths = vec![large_path.to_str().unwrap().to_string()];

        create_angled_strip_wallpaper(output_path_str, (200, 200), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (200, 200));
    }

    #[test]
    fn test_images_with_transparency() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Create an RGBA image with transparency
        let rgba_path = dir.path().join("transparent.png");
        let rgba_img = image::RgbaImage::from_pixel(100, 100, image::Rgba([255, 0, 0, 128])); // Semi-transparent red
        rgba_img.save_with_format(&rgba_path, image::ImageFormat::Png).unwrap();

        let paths = vec![rgba_path.to_str().unwrap().to_string()];

        // Should handle transparency by converting to RGB
        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
        // Transparency should be handled (converted to RGB)
    }

    #[test]
    fn test_duplicate_images() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        let image_path = dir.path().join("test.png");
        create_dummy_image(&image_path, 100, 100, Rgb([255, 0, 0]));

        // Use the same image multiple times
        let paths = vec![
            image_path.to_str().unwrap().to_string(),
            image_path.to_str().unwrap().to_string(),
            image_path.to_str().unwrap().to_string(),
        ];

        create_angled_strip_wallpaper(output_path_str, (150, 150), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (150, 150));
        // Should handle duplicates fine
    }

    #[test]
    fn test_mixed_valid_invalid_images() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Create one valid image
        let valid_path = dir.path().join("valid.png");
        create_dummy_image(&valid_path, 100, 100, Rgb([0, 255, 0]));

        // Create one invalid file
        let invalid_path = dir.path().join("invalid.jpg");
        std::fs::write(&invalid_path, b"Not an image").unwrap();

        let paths = vec![
            invalid_path.to_str().unwrap().to_string(), // Invalid first
            valid_path.to_str().unwrap().to_string(),   // Valid second
            "/non/existent.png".to_string(),            // Non-existent third
        ];

        create_angled_strip_wallpaper(output_path_str, (100, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (100, 100));
        // Should process the one valid image successfully
        assert_eq!(*output_img.get_pixel(50, 50), Rgb([0, 255, 0]));
    }

    #[test]
    fn test_ordering_preservation() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.png");
        let output_path_str = output_path.to_str().unwrap();

        // Create images with distinct colors
        let red_path = dir.path().join("red.png");
        let green_path = dir.path().join("green.png");
        let blue_path = dir.path().join("blue.png");

        create_dummy_image(&red_path, 100, 100, Rgb([255, 0, 0]));
        create_dummy_image(&green_path, 100, 100, Rgb([0, 255, 0]));
        create_dummy_image(&blue_path, 100, 100, Rgb([0, 0, 255]));

        let paths = vec![
            red_path.to_str().unwrap().to_string(),
            green_path.to_str().unwrap().to_string(),
            blue_path.to_str().unwrap().to_string(),
        ];

        create_angled_strip_wallpaper(output_path_str, (300, 100), 0.0, &paths).unwrap();

        let output_img = image::open(&output_path).unwrap().to_rgb8();
        assert_eq!(output_img.dimensions(), (300, 100));
        
        // Check that ordering is preserved (left to right for 0-degree angle)
        assert_eq!(*output_img.get_pixel(50, 50), Rgb([255, 0, 0]));  // Left: red
        assert_eq!(*output_img.get_pixel(150, 50), Rgb([0, 255, 0])); // Middle: green  
        assert_eq!(*output_img.get_pixel(250, 50), Rgb([0, 0, 255])); // Right: blue
    }
}

