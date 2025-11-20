//! Color harmony generation using HSL color space.
//!
//! Provides functions to generate harmonious color palettes from a base color using
//! traditional color theory rules. All harmonies are computed by rotating the hue angle
//! in HSL space while optionally adjusting saturation and lightness.

use crate::colors::{Hsl, Rgb, clamp01};

/// Defines different types of color harmonies based on traditional color theory.
///
/// Each harmony type uses specific hue angle relationships to create pleasing color combinations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HarmonyKind {
    /// Complementary: base color + opposite color (H+180�)
    Complementary,
    /// Split-complementary: base + two colors adjacent to complement (H+150�, H+210�)
    SplitComplementary,
    /// Analogous: base + adjacent colors (H��), default �=30�
    Analogous(f32),
    /// Triadic: three evenly spaced colors (H+0�, H+120�, H+240�)
    Triadic,
    /// Tetradic (rectangle): four colors forming rectangle (H+0�, H+60�, H+180�, H+240�)
    Tetradic,
    /// Square: four evenly spaced colors (H+0�, H+90�, H+180�, H+270�)
    Square,
}

/// Converts RGB to HSL color space.
///
/// HSL (Hue, Saturation, Lightness) is a cylindrical representation where:
/// - Hue represents the color angle (0-360�)
/// - Saturation represents color intensity (0-1)
/// - Lightness represents brightness (0-1)
///
/// This implementation uses the standard RGB�HSL algorithm:
/// 1. Find max and min RGB components
/// 2. Calculate lightness as (max + min) / 2
/// 3. Calculate saturation based on lightness and delta
/// 4. Calculate hue based on which component is maximum
impl From<Rgb> for Hsl {
    fn from(rgb: Rgb) -> Self {
        let r = rgb.r;
        let g = rgb.g;
        let b = rgb.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta < 1e-10 {
            return Hsl::new(0.0, 0.0, l);
        }

        let s = if l < 0.5 { delta / (max + min) } else { delta / (2.0 - max - min) };

        let h = if (max - r).abs() < 1e-10 {
            ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) * 60.0
        } else if (max - g).abs() < 1e-10 {
            ((b - r) / delta + 2.0) * 60.0
        } else {
            ((r - g) / delta + 4.0) * 60.0
        };

        Hsl::new(h, s, l)
    }
}

/// Converts HSL to RGB color space.
///
/// Uses the standard HSL�RGB algorithm with helper function for calculating
/// RGB components from hue, chroma, and adjusted value.
impl From<Hsl> for Rgb {
    fn from(hsl: Hsl) -> Self {
        let h = hsl.h;
        let s = hsl.s;
        let l = hsl.l;

        if s < 1e-10 {
            return Rgb::new(l, l, l);
        }

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

        let (r1, g1, b1) = match h_prime as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        let m = l - c / 2.0;
        Rgb::new(r1 + m, g1 + m, b1 + m)
    }
}

/// Generates a color harmony palette from a base HSL color.
///
/// Returns a vector of HSL colors following the specified harmony pattern.
/// The base color is always included as the first element.
///
/// # Arguments
///
/// * `base` - The base HSL color to generate harmonies from
/// * `kind` - The type of harmony to generate
///
/// # Examples
///
/// ```
/// use colorizer::{colors::Hsl, HarmonyKind, harmonies};
///
/// let base = Hsl::new(180.0, 0.5, 0.5);
/// let palette = harmonies(base, HarmonyKind::Complementary);
/// // Returns [base, complement at 360�]
/// ```
pub fn harmonies(base: Hsl, kind: HarmonyKind) -> Vec<Hsl> {
    let h = base.h;
    let s = base.s;
    let l = base.l;

    match kind {
        HarmonyKind::Complementary => {
            vec![base, Hsl::new(h + 180.0, s, l)]
        }
        HarmonyKind::SplitComplementary => {
            vec![base, Hsl::new(h + 150.0, s, l), Hsl::new(h + 210.0, s, l)]
        }
        HarmonyKind::Analogous(angle) => {
            vec![Hsl::new(h - angle, s, l), base, Hsl::new(h + angle, s, l)]
        }
        HarmonyKind::Triadic => {
            vec![base, Hsl::new(h + 120.0, s, l), Hsl::new(h + 240.0, s, l)]
        }
        HarmonyKind::Tetradic => {
            vec![
                base,
                Hsl::new(h + 60.0, s, l),
                Hsl::new(h + 180.0, s, l),
                Hsl::new(h + 240.0, s, l),
            ]
        }
        HarmonyKind::Square => {
            vec![
                base,
                Hsl::new(h + 90.0, s, l),
                Hsl::new(h + 180.0, s, l),
                Hsl::new(h + 270.0, s, l),
            ]
        }
    }
}

/// Normalizes saturation values in a palette to fall within a specified range.
///
/// Useful for ensuring all colors in a harmony have consistent visual intensity.
/// Values are linearly scaled from their current [0,1] range to [s_min, s_max].
///
/// # Arguments
///
/// * `colors` - Mutable slice of HSL colors to normalize
/// * `s_min` - Minimum saturation value (clamped to [0,1])
/// * `s_max` - Maximum saturation value (clamped to [0,1])
pub fn normalize_saturation(colors: &mut [Hsl], s_min: f32, s_max: f32) {
    let s_min = clamp01(s_min);
    let s_max = clamp01(s_max);

    if s_max <= s_min {
        return;
    }

    for color in colors {
        color.s = s_min + color.s * (s_max - s_min);
    }
}

/// Adjusts the lightness of all colors in a palette by a fixed offset.
///
/// Useful for creating variants suitable for different contexts (text, background, etc.).
/// The offset is added to each color's lightness and clamped to [0,1].
///
/// # Arguments
///
/// * `colors` - Mutable slice of HSL colors to adjust
/// * `offset` - Lightness offset to apply (can be negative to darken)
///
/// # Examples
///
/// ```
/// use colorizer::{colors::Hsl, shift_lightness};
///
/// let mut palette = vec![
///     Hsl::new(0.0, 0.5, 0.3),
///     Hsl::new(120.0, 0.5, 0.5),
/// ];
///
/// // Lighten colors for use on dark backgrounds
/// shift_lightness(&mut palette, 0.2);
///
/// // Darken colors for use as text
/// shift_lightness(&mut palette, -0.3);
/// ```
pub fn shift_lightness(colors: &mut [Hsl], offset: f32) {
    for color in colors {
        color.l = clamp01(color.l + offset);
    }
}

/// Sets all colors in a palette to a specific lightness value.
///
/// Useful for creating palettes with uniform brightness, which can be important
/// for accessibility and visual consistency.
///
/// # Arguments
///
/// * `colors` - Mutable slice of HSL colors to adjust
/// * `lightness` - Target lightness value (clamped to [0,1])
pub fn set_lightness(colors: &mut [Hsl], lightness: f32) {
    let l = clamp01(lightness);
    for color in colors {
        color.l = l;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.01;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_rgb_to_hsl_primary_colors() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let hsl = Hsl::from(red);
        assert!(approx_eq(hsl.h, 0.0));
        assert!(approx_eq(hsl.s, 1.0));
        assert!(approx_eq(hsl.l, 0.5));

        let green = Rgb::new(0.0, 1.0, 0.0);
        let hsl = Hsl::from(green);
        assert!(approx_eq(hsl.h, 120.0));
        assert!(approx_eq(hsl.s, 1.0));
        assert!(approx_eq(hsl.l, 0.5));

        let blue = Rgb::new(0.0, 0.0, 1.0);
        let hsl = Hsl::from(blue);
        assert!(approx_eq(hsl.h, 240.0));
        assert!(approx_eq(hsl.s, 1.0));
        assert!(approx_eq(hsl.l, 0.5));
    }

    #[test]
    fn test_rgb_to_hsl_grayscale() {
        let black = Rgb::new(0.0, 0.0, 0.0);
        let hsl = Hsl::from(black);
        assert!(approx_eq(hsl.s, 0.0));
        assert!(approx_eq(hsl.l, 0.0));

        let white = Rgb::new(1.0, 1.0, 1.0);
        let hsl = Hsl::from(white);
        assert!(approx_eq(hsl.s, 0.0));
        assert!(approx_eq(hsl.l, 1.0));

        let gray = Rgb::new(0.5, 0.5, 0.5);
        let hsl = Hsl::from(gray);
        assert!(approx_eq(hsl.s, 0.0));
        assert!(approx_eq(hsl.l, 0.5));
    }

    #[test]
    fn test_hsl_to_rgb_primary_colors() {
        let red = Hsl::new(0.0, 1.0, 0.5);
        let rgb = Rgb::from(red);
        assert!(approx_eq(rgb.r, 1.0));
        assert!(approx_eq(rgb.g, 0.0));
        assert!(approx_eq(rgb.b, 0.0));

        let green = Hsl::new(120.0, 1.0, 0.5);
        let rgb = Rgb::from(green);
        assert!(approx_eq(rgb.r, 0.0));
        assert!(approx_eq(rgb.g, 1.0));
        assert!(approx_eq(rgb.b, 0.0));

        let blue = Hsl::new(240.0, 1.0, 0.5);
        let rgb = Rgb::from(blue);
        assert!(approx_eq(rgb.r, 0.0));
        assert!(approx_eq(rgb.g, 0.0));
        assert!(approx_eq(rgb.b, 1.0));
    }

    #[test]
    fn test_hsl_to_rgb_grayscale() {
        let black = Hsl::new(0.0, 0.0, 0.0);
        let rgb = Rgb::from(black);
        assert!(approx_eq(rgb.r, 0.0));
        assert!(approx_eq(rgb.g, 0.0));
        assert!(approx_eq(rgb.b, 0.0));

        let white = Hsl::new(0.0, 0.0, 1.0);
        let rgb = Rgb::from(white);
        assert!(approx_eq(rgb.r, 1.0));
        assert!(approx_eq(rgb.g, 1.0));
        assert!(approx_eq(rgb.b, 1.0));

        let gray = Hsl::new(180.0, 0.0, 0.5);
        let rgb = Rgb::from(gray);
        assert!(approx_eq(rgb.r, 0.5));
        assert!(approx_eq(rgb.g, 0.5));
        assert!(approx_eq(rgb.b, 0.5));
    }

    #[test]
    fn test_rgb_hsl_round_trip() {
        let test_colors = vec![
            Rgb::new(1.0, 0.0, 0.0),
            Rgb::new(0.0, 1.0, 0.0),
            Rgb::new(0.0, 0.0, 1.0),
            Rgb::new(1.0, 1.0, 0.0),
            Rgb::new(0.5, 0.3, 0.8),
            Rgb::new(0.2, 0.7, 0.4),
        ];

        for rgb in test_colors {
            let hsl = Hsl::from(rgb);
            let back = Rgb::from(hsl);
            assert!(approx_eq(back.r, rgb.r), "R mismatch: {} != {}", back.r, rgb.r);
            assert!(approx_eq(back.g, rgb.g), "G mismatch: {} != {}", back.g, rgb.g);
            assert!(approx_eq(back.b, rgb.b), "B mismatch: {} != {}", back.b, rgb.b);
        }
    }

    #[test]
    fn test_complementary_harmony() {
        let base = Hsl::new(180.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::Complementary);

        assert_eq!(palette.len(), 2);
        assert!(approx_eq(palette[0].h, 180.0));
        assert!(approx_eq(palette[1].h, 0.0));
    }

    #[test]
    fn test_split_complementary_harmony() {
        let base = Hsl::new(0.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::SplitComplementary);

        assert_eq!(palette.len(), 3);
        assert!(approx_eq(palette[0].h, 0.0));
        assert!(approx_eq(palette[1].h, 150.0));
        assert!(approx_eq(palette[2].h, 210.0));
    }

    #[test]
    fn test_analogous_harmony() {
        let base = Hsl::new(180.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::Analogous(30.0));

        assert_eq!(palette.len(), 3);
        assert!(approx_eq(palette[0].h, 150.0));
        assert!(approx_eq(palette[1].h, 180.0));
        assert!(approx_eq(palette[2].h, 210.0));
    }

    #[test]
    fn test_triadic_harmony() {
        let base = Hsl::new(0.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::Triadic);

        assert_eq!(palette.len(), 3);
        assert!(approx_eq(palette[0].h, 0.0));
        assert!(approx_eq(palette[1].h, 120.0));
        assert!(approx_eq(palette[2].h, 240.0));
    }

    #[test]
    fn test_tetradic_harmony() {
        let base = Hsl::new(0.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::Tetradic);

        assert_eq!(palette.len(), 4);
        assert!(approx_eq(palette[0].h, 0.0));
        assert!(approx_eq(palette[1].h, 60.0));
        assert!(approx_eq(palette[2].h, 180.0));
        assert!(approx_eq(palette[3].h, 240.0));
    }

    #[test]
    fn test_square_harmony() {
        let base = Hsl::new(0.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::Square);

        assert_eq!(palette.len(), 4);
        assert!(approx_eq(palette[0].h, 0.0));
        assert!(approx_eq(palette[1].h, 90.0));
        assert!(approx_eq(palette[2].h, 180.0));
        assert!(approx_eq(palette[3].h, 270.0));
    }

    #[test]
    fn test_hue_wrapping_in_harmonies() {
        let base = Hsl::new(350.0, 0.5, 0.5);
        let palette = harmonies(base, HarmonyKind::Complementary);
        assert!(approx_eq(palette[1].h, 170.0));
    }

    #[test]
    fn test_normalize_saturation() {
        let mut palette = vec![
            Hsl::new(0.0, 0.0, 0.5),
            Hsl::new(120.0, 0.5, 0.5),
            Hsl::new(240.0, 1.0, 0.5),
        ];

        normalize_saturation(&mut palette, 0.4, 0.8);

        assert!(approx_eq(palette[0].s, 0.4));
        assert!(approx_eq(palette[1].s, 0.6));
        assert!(approx_eq(palette[2].s, 0.8));
    }

    #[test]
    fn test_shift_lightness() {
        let mut palette = vec![
            Hsl::new(0.0, 0.5, 0.3),
            Hsl::new(120.0, 0.5, 0.5),
            Hsl::new(240.0, 0.5, 0.7),
        ];

        shift_lightness(&mut palette, 0.2);

        assert!(approx_eq(palette[0].l, 0.5));
        assert!(approx_eq(palette[1].l, 0.7));
        assert!(approx_eq(palette[2].l, 0.9));

        shift_lightness(&mut palette, 0.5);
        assert!(approx_eq(palette[2].l, 1.0));
    }

    #[test]
    fn test_set_lightness() {
        let mut palette = vec![
            Hsl::new(0.0, 0.5, 0.3),
            Hsl::new(120.0, 0.5, 0.5),
            Hsl::new(240.0, 0.5, 0.7),
        ];

        set_lightness(&mut palette, 0.6);

        assert!(approx_eq(palette[0].l, 0.6));
        assert!(approx_eq(palette[1].l, 0.6));
        assert!(approx_eq(palette[2].l, 0.6));
    }

    #[test]
    fn test_normalize_saturation_edge_cases() {
        let mut palette = vec![Hsl::new(0.0, 0.5, 0.5)];

        normalize_saturation(&mut palette, 0.8, 0.4);
        assert!(approx_eq(palette[0].s, 0.5));

        normalize_saturation(&mut palette, -0.1, 1.5);
        assert!(palette[0].s >= 0.0 && palette[0].s <= 1.0);
    }
}
