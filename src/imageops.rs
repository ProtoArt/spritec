//! Additional image utilities that extend the ones provided by the image crate

use image::RgbaImage;

/// Scales the given source image to fit into the target image.
///
/// The target image dimensions must be a multiple of the source image dimensions. No interpolation
/// is performed during the scaling operation.
pub fn scale(source: &RgbaImage, target: &mut RgbaImage) {
    let source_width = source.width();
    let source_height = source.height();

    let target_width = target.width();
    let target_height = target.height();

    let scale_x = target_width / source_width;
    let scale_y = target_height / source_height;

    // Check for truncating division. Should only fail if the dimensions are not even multiples
    // of each other.
    assert_eq!(source_width * scale_x, target_width);
    assert_eq!(source_height * scale_y, target_height);

    // Blit the pixels with no anti-aliasing
    for x in 0..source_width {
        for y in 0..source_height {
            let pixel = *source.get_pixel(x, y);

            // Copy the color to every pixel in the scaled box
            for i in 0..scale_x {
                for j in 0..scale_y {
                    let sx = x * scale_x + i;
                    let sy = y * scale_y + j;
                    target.put_pixel(sx, sy, pixel);
                }
            }
        }
    }
}

/// Copy the entire source buffer into the given target buffer starting at the given offset.
pub fn copy(source: &RgbaImage, target: &mut RgbaImage, (offset_x, offset_y): (u32, u32)) {
    let source_width = source.width();
    let source_height = source.height();

    assert!(offset_x + source_width <= target.width());
    assert!(offset_y + source_height <= target.height());

    for x in 0..source_width {
        for y in 0..source_height {
            let pixel = *source.get_pixel(x, y);

            // Copy the color to every pixel in the scaled box
            let cx = x + offset_x;
            let cy = y + offset_y;
            target.put_pixel(cx, cy, pixel);
        }
    }
}
