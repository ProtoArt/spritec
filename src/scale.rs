use euc::{Target, buffer::Buffer2d};

/// Scales the given source buffer into the target buffer by directly copying each value.
///
/// The dimensions of the target buffer must be an even multiple of the dimensions of the source.
pub fn scale_buffer<T: Clone + Copy>(target: &mut Buffer2d<T>, source: &Buffer2d<T>) {
    scale_buffer_map(target, source, |x| x)
}

/// Same as scale_buffer but also runs the given function for each color
pub fn scale_buffer_map<T, U, F>(target: &mut Buffer2d<U>, source: &Buffer2d<T>, f: F)
    where T: Clone + Copy,
          U: Clone + Copy,
          F: Fn(T) -> U,
{
    let target_size = target.size();
    let source_size = source.size();
    let scale_x = target_size[0] / source_size[0];
    let scale_y = target_size[1] / source_size[1];

    // Check for truncating division. Should only fail if the dimensions are not even multiples
    // of each other.
    debug_assert_eq!(source_size[0] * scale_x, target_size[0]);
    debug_assert_eq!(source_size[1] * scale_y, target_size[1]);

    // Blit the pixels with no anti-aliasing
    for i in 0..source_size[0] {
        for j in 0..source_size[1] {
            // Unsafe because we are guaranteeing that these indexes are not out of bounds
            let color = f(unsafe { *source.get([i, j]) });

            // Copy the color to every pixel in the scaled box
            for sx in 0..scale_x {
                for sy in 0..scale_y {
                    // Unsafe because we are guaranteeing that these indexes are not out of bounds
                    unsafe {
                        target.set([i * scale_x + sx, j * scale_y + sy], color);
                    }
                }
            }
        }
    }
}
