//! Color interpolation and gradient generation.
//!
//! Provides functions for smoothly transitioning between colors in different color spaces.
//! RGB interpolation is simple but may produce unexpected colors;
//! Lab and Lch interpolation are perceptually uniform and produce more natural gradients.

use crate::colors::{Lab, Lch, Rgb, clamp01, wrap_degrees};

/// Linearly interpolates between two RGB colors.
///
/// Performs component-wise linear interpolation in RGB space:
/// - `t = 0.0` returns color `a`
/// - `t = 1.0` returns color `b`
/// - `t = 0.5` returns the midpoint between `a` and `b`
///
/// Note: RGB interpolation may produce unexpected intermediate colors (e.g., brown when
/// mixing red and green). For perceptually uniform gradients, consider `lerp_lab` or `lerp_lch`.
///
/// # Arguments
///
/// * `a` - The starting RGB color
/// * `b` - The ending RGB color
/// * `t` - The interpolation parameter in [0, 1]
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::interpolation::lerp_rgb;
///
/// let red = Rgb::new(1.0, 0.0, 0.0);
/// let blue = Rgb::new(0.0, 0.0, 1.0);
/// let purple = lerp_rgb(red, blue, 0.5);
/// ```
pub fn lerp_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = clamp01(t);
    Rgb::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t)
}

/// Linearly interpolates between two Lab colors.
///
/// Performs component-wise linear interpolation in Lab color space, which is perceptually
/// uniform. This produces more natural-looking gradients than RGB interpolation.
///
/// # Arguments
///
/// * `a` - The starting Lab color
/// * `b` - The ending Lab color
/// * `t` - The interpolation parameter in [0, 1]
///
/// # Examples
///
/// ```
/// use colorizer::colors::Lab;
/// use colorizer::interpolation::lerp_lab;
///
/// let color1 = Lab::new(50.0, 20.0, 30.0);
/// let color2 = Lab::new(70.0, -10.0, 40.0);
/// let mid = lerp_lab(color1, color2, 0.5);
/// ```
pub fn lerp_lab(a: Lab, b: Lab, t: f32) -> Lab {
    let t = clamp01(t);
    Lab::new(a.l + (b.l - a.l) * t, a.a + (b.a - a.a) * t, a.b + (b.b - a.b) * t)
}

/// Linearly interpolates between two Lch colors with circular hue interpolation.
///
/// Interpolates L and C components linearly, while the hue component is interpolated
/// circularly to take the shortest path around the color wheel. This produces smooth,
/// perceptually uniform gradients that avoid unexpected hue shifts.
///
/// The hue interpolation computes the shortest angular difference and wraps correctly
/// around the 0/360 degree boundary.
///
/// # Arguments
///
/// * `a` - The starting Lch color
/// * `b` - The ending Lch color
/// * `t` - The interpolation parameter in [0, 1]
///
/// # Examples
///
/// ```
/// use colorizer::colors::Lch;
/// use colorizer::interpolation::lerp_lch;
///
/// let red = Lch::new(50.0, 60.0, 40.0);  // Reddish
/// let blue = Lch::new(50.0, 60.0, 280.0); // Bluish
/// let mid = lerp_lch(red, blue, 0.5);     // Smooth transition via purple
/// ```
pub fn lerp_lch(a: Lch, b: Lch, t: f32) -> Lch {
    let t = clamp01(t);
    let l = a.l + (b.l - a.l) * t;
    let c = a.c + (b.c - a.c) * t;
    let mut delta_h = b.h - a.h;

    if delta_h > 180.0 {
        delta_h -= 360.0;
    } else if delta_h < -180.0 {
        delta_h += 360.0;
    }

    let h = wrap_degrees(a.h + delta_h * t);

    Lch::new(l, c, h)
}

/// Generates a gradient of colors in Lab space.
///
/// Creates a smooth gradient between two RGB colors by converting to Lab space,
/// interpolating, and converting back. Lab is perceptually uniform, so gradients
/// appear smooth and natural to human vision.
///
/// # Arguments
///
/// * `a` - The starting RGB color
/// * `b` - The ending RGB color
/// * `steps` - The number of colors to generate (must be at least 2)
///
/// # Returns
///
/// A vector of RGB colors representing the gradient. Returns empty vector if steps < 2.
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::interpolation::gradient_lab;
///
/// let red = Rgb::new(1.0, 0.0, 0.0);
/// let blue = Rgb::new(0.0, 0.0, 1.0);
/// let gradient = gradient_lab(red, blue, 5);
/// // Returns [red, ..., blue] with 5 smooth steps
/// ```
pub fn gradient_lab(a: Rgb, b: Rgb, steps: usize) -> Vec<Rgb> {
    if steps < 2 {
        return Vec::new();
    }

    let a_lab: Lab = {
        let xyz = crate::colors::Xyz::from(a);
        Lab::from(xyz)
    };

    let b_lab: Lab = {
        let xyz = crate::colors::Xyz::from(b);
        Lab::from(xyz)
    };

    let mut result = Vec::with_capacity(steps);

    for i in 0..steps {
        let t = if steps == 1 { 0.0 } else { i as f32 / (steps - 1) as f32 };
        let lab = lerp_lab(a_lab, b_lab, t);
        let xyz = crate::colors::Xyz::from(lab);
        let rgb = Rgb::from(xyz);
        result.push(rgb);
    }

    result
}

/// Generates a gradient of colors in Lch space.
///
/// Creates a smooth gradient between two RGB colors by converting to Lch space,
/// interpolating with circular hue handling, and converting back. Lch is perceptually
/// uniform and handles hue rotation naturally, producing beautiful gradients.
///
/// # Arguments
///
/// * `a` - The starting RGB color
/// * `b` - The ending RGB color
/// * `steps` - The number of colors to generate (must be at least 2)
///
/// # Returns
///
/// A vector of RGB colors representing the gradient. Returns empty vector if steps < 2.
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::interpolation::gradient_lch;
///
/// let yellow = Rgb::new(1.0, 1.0, 0.0);
/// let cyan = Rgb::new(0.0, 1.0, 1.0);
/// let gradient = gradient_lch(yellow, cyan, 10);
/// // Returns smooth gradient through green hues
/// ```
pub fn gradient_lch(a: Rgb, b: Rgb, steps: usize) -> Vec<Rgb> {
    if steps < 2 {
        return Vec::new();
    }

    let a_lch: Lch = {
        let xyz = crate::colors::Xyz::from(a);
        let lab = Lab::from(xyz);
        Lch::from(lab)
    };

    let b_lch: Lch = {
        let xyz = crate::colors::Xyz::from(b);
        let lab = Lab::from(xyz);
        Lch::from(lab)
    };

    let mut result = Vec::with_capacity(steps);

    for i in 0..steps {
        let t = if steps == 1 { 0.0 } else { i as f32 / (steps - 1) as f32 };
        let lch = lerp_lch(a_lch, b_lch, t);
        let lab = Lab::from(lch);
        let xyz = crate::colors::Xyz::from(lab);
        let rgb = Rgb::from(xyz);
        result.push(rgb);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_lerp_rgb_endpoints() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 1.0);
        let result = lerp_rgb(red, blue, 0.0);
        assert!(approx_eq(result.r, 1.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 0.0));

        let result = lerp_rgb(red, blue, 1.0);
        assert!(approx_eq(result.r, 0.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 1.0));
    }

    #[test]
    fn test_lerp_rgb_midpoint() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 1.0);
        let result = lerp_rgb(red, blue, 0.5);
        assert!(approx_eq(result.r, 0.5));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 0.5));
    }

    #[test]
    fn test_lerp_rgb_clamping() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 1.0);
        let result = lerp_rgb(red, blue, -0.5);
        assert!(approx_eq(result.r, 1.0));

        let result = lerp_rgb(red, blue, 1.5);
        assert!(approx_eq(result.b, 1.0));
    }

    #[test]
    fn test_lerp_lab_endpoints() {
        let a = Lab::new(50.0, 20.0, 30.0);
        let b = Lab::new(70.0, -10.0, 40.0);
        let result = lerp_lab(a, b, 0.0);
        assert!(approx_eq(result.l, 50.0));
        assert!(approx_eq(result.a, 20.0));
        assert!(approx_eq(result.b, 30.0));

        let result = lerp_lab(a, b, 1.0);
        assert!(approx_eq(result.l, 70.0));
        assert!(approx_eq(result.a, -10.0));
        assert!(approx_eq(result.b, 40.0));
    }

    #[test]
    fn test_lerp_lab_midpoint() {
        let a = Lab::new(50.0, 20.0, 30.0);
        let b = Lab::new(70.0, -10.0, 40.0);
        let result = lerp_lab(a, b, 0.5);
        assert!(approx_eq(result.l, 60.0));
        assert!(approx_eq(result.a, 5.0));
        assert!(approx_eq(result.b, 35.0));
    }

    #[test]
    fn test_lerp_lch_linear_components() {
        let a = Lch::new(50.0, 30.0, 45.0);
        let b = Lch::new(70.0, 50.0, 45.0);
        let result = lerp_lch(a, b, 0.5);
        assert!(approx_eq(result.l, 60.0));
        assert!(approx_eq(result.c, 40.0));
        assert!(approx_eq(result.h, 45.0));
    }

    #[test]
    fn test_lerp_lch_circular_hue_short_path() {
        let a = Lch::new(50.0, 30.0, 10.0);
        let b = Lch::new(50.0, 30.0, 50.0);
        let result = lerp_lch(a, b, 0.5);
        assert!(approx_eq(result.h, 30.0));
    }

    #[test]
    fn test_lerp_lch_circular_hue_wrapping_forward() {
        let a = Lch::new(50.0, 30.0, 350.0);
        let b = Lch::new(50.0, 30.0, 10.0);
        let result = lerp_lch(a, b, 0.5);
        assert!(approx_eq(result.h, 0.0) || approx_eq(result.h, 360.0));
    }

    #[test]
    fn test_lerp_lch_circular_hue_wrapping_backward() {
        let a = Lch::new(50.0, 30.0, 10.0);
        let b = Lch::new(50.0, 30.0, 350.0);
        let result = lerp_lch(a, b, 0.5);
        assert!(approx_eq(result.h, 0.0) || approx_eq(result.h, 360.0));
    }

    #[test]
    fn test_lerp_lch_circular_hue_long_path() {
        let a = Lch::new(50.0, 30.0, 0.0);
        let b = Lch::new(50.0, 30.0, 180.0);
        let result = lerp_lch(a, b, 0.5);
        assert!(approx_eq(result.h, 90.0) || approx_eq(result.h, 270.0));
    }

    #[test]
    fn test_lerp_lch_endpoints() {
        let a = Lch::new(50.0, 30.0, 45.0);
        let b = Lch::new(70.0, 50.0, 135.0);
        let result = lerp_lch(a, b, 0.0);
        assert!(approx_eq(result.l, 50.0));
        assert!(approx_eq(result.c, 30.0));
        assert!(approx_eq(result.h, 45.0));

        let result = lerp_lch(a, b, 1.0);
        assert!(approx_eq(result.l, 70.0));
        assert!(approx_eq(result.c, 50.0));
        assert!(approx_eq(result.h, 135.0));
    }

    #[test]
    fn test_gradient_lab_length() {
        let a = Rgb::new(1.0, 0.0, 0.0);
        let b = Rgb::new(0.0, 0.0, 1.0);
        let gradient = gradient_lab(a, b, 5);
        assert_eq!(gradient.len(), 5);

        let gradient = gradient_lab(a, b, 2);
        assert_eq!(gradient.len(), 2);

        let gradient = gradient_lab(a, b, 1);
        assert_eq!(gradient.len(), 0);

        let gradient = gradient_lab(a, b, 0);
        assert_eq!(gradient.len(), 0);
    }

    #[test]
    fn test_gradient_lab_endpoints() {
        let a = Rgb::new(1.0, 0.0, 0.0);
        let b = Rgb::new(0.0, 0.0, 1.0);
        let gradient = gradient_lab(a, b, 5);
        assert!(approx_eq(gradient[0].r, a.r));
        assert!(approx_eq(gradient[0].g, a.g));
        assert!(approx_eq(gradient[0].b, a.b));

        assert!(approx_eq(gradient[4].r, b.r));
        assert!(approx_eq(gradient[4].g, b.g));
        assert!(approx_eq(gradient[4].b, b.b));
    }

    #[test]
    fn test_gradient_lch_length() {
        let a = Rgb::new(1.0, 0.0, 0.0);
        let b = Rgb::new(0.0, 1.0, 0.0);
        let gradient = gradient_lch(a, b, 5);
        assert_eq!(gradient.len(), 5);

        let gradient = gradient_lch(a, b, 2);
        assert_eq!(gradient.len(), 2);

        let gradient = gradient_lch(a, b, 1);
        assert_eq!(gradient.len(), 0);

        let gradient = gradient_lch(a, b, 0);
        assert_eq!(gradient.len(), 0);
    }

    #[test]
    fn test_gradient_lch_endpoints() {
        let a = Rgb::new(1.0, 0.0, 0.0);
        let b = Rgb::new(0.0, 1.0, 0.0);
        let gradient = gradient_lch(a, b, 5);
        assert!(approx_eq(gradient[0].r, a.r));
        assert!(approx_eq(gradient[0].g, a.g));
        assert!(approx_eq(gradient[0].b, a.b));

        assert!(approx_eq(gradient[4].r, b.r));
        assert!(approx_eq(gradient[4].g, b.g));
        assert!(approx_eq(gradient[4].b, b.b));
    }

    #[test]
    fn test_gradient_lab_smooth_transition() {
        let a = Rgb::new(1.0, 0.0, 0.0);
        let b = Rgb::new(0.0, 0.0, 1.0);
        let gradient = gradient_lab(a, b, 10);

        for i in 0..gradient.len() - 1 {
            assert!(gradient[i].r >= gradient[i + 1].r - EPSILON);
            assert!(gradient[i].b <= gradient[i + 1].b + EPSILON);
        }
    }

    #[test]
    fn test_gradient_lch_smooth_transition() {
        let a = Rgb::new(1.0, 0.0, 0.0);
        let b = Rgb::new(0.0, 1.0, 0.0);
        let gradient = gradient_lch(a, b, 10);

        assert_eq!(gradient.len(), 10);

        for color in &gradient {
            assert!(color.r >= -EPSILON && color.r <= 1.0 + EPSILON);
            assert!(color.g >= -EPSILON && color.g <= 1.0 + EPSILON);
            assert!(color.b >= -EPSILON && color.b <= 1.0 + EPSILON);
        }
    }
}
