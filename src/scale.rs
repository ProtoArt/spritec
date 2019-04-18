use euc::{Target, buffer::Buffer2d};

/// Scales the given source buffer into the target buffer by directly copying each value. The given
/// function is applied to each value before it is written into the target buffer.
///
/// The dimensions of the target buffer must be an even multiple of the dimensions of the source.
pub fn scale_map<T, U, F>(target: &mut Buffer2d<U>, source: &Buffer2d<T>, f: F)
    where T: Clone + Copy,
          U: Clone + Copy,
          F: Fn(T) -> U,
{
    // Unsafe because we are guaranteeing that these indexes are not out of bounds
    scale_with(target.size(), source, |pos, value| unsafe { target.set(pos, f(value)); })
}

/// Scales the given source buffer into some target size by calling the given function with a
/// position on the target buffer and the value to place there. Each provided target position is
/// guaranteed to be within the bounds of target_width and target_height.
///
/// The dimensions of the target buffer must be an even multiple of the dimensions of the source.
pub fn scale_with<T: Clone + Copy, F: FnMut([usize; 2], T)>(
    [target_width, target_height]: [usize; 2],
    source: &Buffer2d<T>,
    mut f: F,
) {
    let [source_width, source_height] = source.size();
    let scale_x = target_width / source_width;
    let scale_y = target_height / source_height;

    // Check for truncating division. Should only fail if the dimensions are not even multiples
    // of each other.
    debug_assert_eq!(source_width * scale_x, target_width);
    debug_assert_eq!(source_height * scale_y, target_height);

    // Blit the pixels with no anti-aliasing
    for i in 0..source_width {
        for j in 0..source_height {
            // Unsafe because we are guaranteeing that these indexes are not out of bounds
            let color = unsafe { *source.get([i, j]) };

            // Copy the color to every pixel in the scaled box
            for sx in 0..scale_x {
                for sy in 0..scale_y {
                    f([i * scale_x + sx, j * scale_y + sy], color);
                }
            }
        }
    }
}

/// Copy the entire source buffer into the given target buffer starting at the given offset.
///
/// Unsafe because no bounds checking is performed.
pub unsafe fn copy<T: Clone + Copy>(target: &mut Buffer2d<T>, source: &Buffer2d<T>, (x, y): (usize, usize)) {
    copy_map(target, source, (x, y), |p| p)
}

/// Copy the entire source buffer into the given target buffer starting at the given offset.
/// Applies the given function to each value before writing to the target buffer.
///
/// Unsafe because no bounds checking is performed.
pub unsafe fn copy_map<T, U, F>(target: &mut Buffer2d<U>, source: &Buffer2d<T>, (x, y): (usize, usize), f: F)
    where T: Clone + Copy,
          U: Clone + Copy,
          F: Fn(T) -> U,
{
    let [source_width, source_height] = source.size();

    for i in 0..source_width {
        for j in 0..source_height {
            let value = f(*source.get([i, j]));
            target.set([x + i, y + j], value);
        }
    }
}
