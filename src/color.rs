use vek::Rgba;
use image::Pixel;

/// Converts a Rgba color to a bgra u32 color
///
/// Each Rgba value should be between 0.0 and 1.0
#[inline(always)]
pub fn rgba_to_bgra_u32(Rgba {r, g, b, a}: Rgba<f32>) -> u32 {
    // Truncating conversion to u8 from f32 in range 0.0 to 1.0
    let to_u8 = |x| (x * 255.0) as u8;

    (to_u8(b) as u32) << 0 |
    (to_u8(g) as u32) << 8 |
    (to_u8(r) as u32) << 16 |
    (to_u8(a) as u32) << 24
}

/// Converts a u32 bgra color to a Rgba color
#[inline(always)]
pub fn bgra_u32_to_rgba(value: u32) -> Rgba<f32> {
    let to_rgba = |x, offset| (x >> offset) as u8 as f32 / 255.0;

    Rgba {
        r: to_rgba(value, 16),
        g: to_rgba(value, 8),
        b: to_rgba(value, 0),
        a: to_rgba(value, 24),
    }
}

#[inline(always)]
pub fn vek_rgba_to_image_rgba(Rgba {r, g, b, a}: Rgba<f32>) -> image::Rgba<u8> {
    image::Rgba::from_channels(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq_color {
        ($left:expr, $right:expr) => (
            {
                let left = $left;
                let right = $right;
                assert!((left.r - right.r).abs() < 0.005, "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`", left, right);
                assert!((left.g - right.g).abs() < 0.005, "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`", left, right);
                assert!((left.b - right.b).abs() < 0.005, "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`", left, right);
                assert!((left.a - right.a).abs() < 0.005, "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`", left, right);
            }
        );
    }

    #[test]
    fn rgba_to_bgra_u32_inverses() {
        //TODO: Use property testing instead of this...
        let n = 50;
        for i in 0..=n {
            for j in 0..=n {
                for k in 0..=n {
                    for l in 0..=n {
                        let color = Rgba {
                            r: i as f32 / n as f32,
                            g: j as f32 / n as f32,
                            b: k as f32 / n as f32,
                            a: l as f32 / n as f32,
                        };
                        let bgra = rgba_to_bgra_u32(color);
                        let rgba = bgra_u32_to_rgba(bgra);
                        assert_approx_eq_color!(color, rgba);
                    }
                }
            }
        }
    }
}
