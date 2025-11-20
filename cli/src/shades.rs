//! Tints, shades, and tones generation for color manipulation.
//!
//! Provides functions to create color variations by mixing with white (tints), black (shades), or gray (tones).
//! Also includes HSL-based convenience functions for lightening, darkening, and desaturating colors.

use crate::colors::{Hsl, Rgb, clamp01};

/// Mixes two RGB colors using linear interpolation.
///
/// Performs linear interpolation between two colors in RGB space:
/// - `t = 0.0` returns color `a`
/// - `t = 1.0` returns color `b`
/// - `t = 0.5` returns the midpoint between `a` and `b`
///
/// The parameter `t` is clamped to [0, 1] to ensure valid results.
///
/// # Arguments
///
/// * `a` - The first RGB color
/// * `b` - The second RGB color
/// * `t` - The interpolation parameter in [0, 1]
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::shades::mix_rgb;
///
/// let red = Rgb::new(1.0, 0.0, 0.0);
/// let blue = Rgb::new(0.0, 0.0, 1.0);
/// let purple = mix_rgb(red, blue, 0.5);
/// // purple H Rgb(0.5, 0.0, 0.5)
/// ```
pub fn mix_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = clamp01(t);
    Rgb::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t)
}

/// Creates a tint by mixing a color with white.
///
/// Tints lighten a color by blending it with white:
/// - `t = 0.0` returns the original color
/// - `t = 1.0` returns pure white
/// - `t = 0.5` returns a color halfway to white
///
/// Tints are useful for creating lighter variations of a color while maintaining its hue, commonly used for hover states or backgrounds.
///
/// # Arguments
///
/// * `color` - The base RGB color to tint
/// * `t` - The amount of white to mix in [0, 1]
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::shades::tint;
///
/// let blue = Rgb::new(0.0, 0.0, 1.0);
/// let light_blue = tint(blue, 0.3);
/// // light_blue H Rgb(0.3, 0.3, 1.0)
/// ```
pub fn tint(color: Rgb, t: f32) -> Rgb {
    let white = Rgb::new(1.0, 1.0, 1.0);
    mix_rgb(color, white, t)
}

/// Creates a shade by mixing a color with black.
///
/// Shades darken a color by blending it with black:
/// - `t = 0.0` returns the original color
/// - `t = 1.0` returns pure black
/// - `t = 0.5` returns a color halfway to black
///
/// Shades are useful for creating darker variations of a color, commonly used for text, shadows, or pressed states.
///
/// # Arguments
///
/// * `color` - The base RGB color to shade
/// * `t` - The amount of black to mix in [0, 1]
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::shades::shade;
///
/// let blue = Rgb::new(0.0, 0.0, 1.0);
/// let dark_blue = shade(blue, 0.3);
/// // dark_blue H Rgb(0.0, 0.0, 0.7)
/// ```
pub fn shade(color: Rgb, t: f32) -> Rgb {
    let black = Rgb::new(0.0, 0.0, 0.0);
    mix_rgb(color, black, t)
}

/// Creates a tone by mixing a color with a gray value.
///
/// Tones reduce the intensity/saturation of a color by blending it with gray:
/// - `t = 0.0` returns the original color
/// - `t = 1.0` returns the gray value
/// - `t = 0.5` returns a color halfway to gray
///
/// The `gray` parameter controls the lightness of the gray (default 0.5 for middle gray).
/// Tones are useful for creating muted or subtle color variations.
///
/// # Arguments
///
/// * `color` - The base RGB color to tone
/// * `t` - The amount of gray to mix in [0, 1]
/// * `gray` - The lightness of the gray value [0, 1], defaults to 0.5
///
/// # Examples
///
/// ```
/// use colorizer::colors::Rgb;
/// use colorizer::shades::tone;
///
/// let blue = Rgb::new(0.0, 0.0, 1.0);
/// let muted_blue = tone(blue, 0.4, 0.5);
/// // muted_blue is a less saturated blue
/// ```
pub fn tone(color: Rgb, t: f32, gray: f32) -> Rgb {
    let gray_value = clamp01(gray);
    let gray_color = Rgb::new(gray_value, gray_value, gray_value);
    mix_rgb(color, gray_color, t)
}

/// Lightens an HSL color by increasing its lightness.
///
/// Increases the lightness component by the specified amount, clamped to [0, 1].
/// This is more intuitive than RGB-based tinting for predictable lightening.
///
/// # Arguments
///
/// * `color` - The HSL color to lighten
/// * `amount` - The amount to increase lightness (can be negative to darken)
///
/// # Examples
///
/// ```
/// use colorizer::colors::Hsl;
/// use colorizer::shades::lighten_hsl;
///
/// let blue = Hsl::new(240.0, 1.0, 0.3);
/// let light_blue = lighten_hsl(blue, 0.2);
/// // light_blue has lightness = 0.5
/// ```
pub fn lighten_hsl(color: Hsl, amount: f32) -> Hsl {
    Hsl::new(color.h, color.s, clamp01(color.l + amount))
}

/// Darkens an HSL color by decreasing its lightness.
///
/// Decreases the lightness component by the specified amount, clamped to [0, 1].
/// Equivalent to `lighten_hsl(color, -amount)` but more semantically clear.
///
/// # Arguments
///
/// * `color` - The HSL color to darken
/// * `amount` - The amount to decrease lightness (positive values darken)
///
/// # Examples
///
/// ```
/// use colorizer::colors::Hsl;
/// use colorizer::shades::darken_hsl;
///
/// let blue = Hsl::new(240.0, 1.0, 0.7);
/// let dark_blue = darken_hsl(blue, 0.3);
/// // dark_blue has lightness = 0.4
/// ```
pub fn darken_hsl(color: Hsl, amount: f32) -> Hsl {
    Hsl::new(color.h, color.s, clamp01(color.l - amount))
}

/// Desaturates an HSL color by decreasing its saturation.
///
/// Decreases the saturation component by the specified amount, clamped to [0, 1].
/// A saturation of 0 produces a pure gray, while 1 is fully saturated.
///
/// # Arguments
///
/// * `color` - The HSL color to desaturate
/// * `amount` - The amount to decrease saturation (positive values desaturate)
///
/// # Examples
///
/// ```
/// use colorizer::colors::Hsl;
/// use colorizer::shades::desaturate_hsl;
///
/// let vibrant_blue = Hsl::new(240.0, 1.0, 0.5);
/// let muted_blue = desaturate_hsl(vibrant_blue, 0.4);
/// // muted_blue has saturation = 0.6
/// ```
pub fn desaturate_hsl(color: Hsl, amount: f32) -> Hsl {
    Hsl::new(color.h, clamp01(color.s - amount), color.l)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_mix_rgb_endpoints() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 1.0);

        let result = mix_rgb(red, blue, 0.0);
        assert!(approx_eq(result.r, 1.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 0.0));

        let result = mix_rgb(red, blue, 1.0);
        assert!(approx_eq(result.r, 0.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 1.0));
    }

    #[test]
    fn test_mix_rgb_midpoint() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 1.0);

        let result = mix_rgb(red, blue, 0.5);
        assert!(approx_eq(result.r, 0.5));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 0.5));
    }

    #[test]
    fn test_mix_rgb_clamping() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 1.0);

        let result = mix_rgb(red, blue, -0.5);
        assert!(approx_eq(result.r, 1.0));

        let result = mix_rgb(red, blue, 1.5);
        assert!(approx_eq(result.b, 1.0));
    }

    #[test]
    fn test_tint_pure_color() {
        let red = Rgb::new(1.0, 0.0, 0.0);

        let result = tint(red, 0.0);
        assert!(approx_eq(result.r, 1.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 0.0));

        let result = tint(red, 1.0);
        assert!(approx_eq(result.r, 1.0));
        assert!(approx_eq(result.g, 1.0));
        assert!(approx_eq(result.b, 1.0));

        let result = tint(red, 0.5);
        assert!(approx_eq(result.r, 1.0));
        assert!(approx_eq(result.g, 0.5));
        assert!(approx_eq(result.b, 0.5));
    }

    #[test]
    fn test_shade_pure_color() {
        let green = Rgb::new(0.0, 1.0, 0.0);

        let result = shade(green, 0.0);
        assert!(approx_eq(result.r, 0.0));
        assert!(approx_eq(result.g, 1.0));
        assert!(approx_eq(result.b, 0.0));

        let result = shade(green, 1.0);
        assert!(approx_eq(result.r, 0.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 0.0));

        let result = shade(green, 0.3);
        assert!(approx_eq(result.r, 0.0));
        assert!(approx_eq(result.g, 0.7));
        assert!(approx_eq(result.b, 0.0));
    }

    #[test]
    fn test_tone_with_middle_gray() {
        let blue = Rgb::new(0.0, 0.0, 1.0);

        let result = tone(blue, 0.0, 0.5);
        assert!(approx_eq(result.r, 0.0));
        assert!(approx_eq(result.g, 0.0));
        assert!(approx_eq(result.b, 1.0));

        let result = tone(blue, 1.0, 0.5);
        assert!(approx_eq(result.r, 0.5));
        assert!(approx_eq(result.g, 0.5));
        assert!(approx_eq(result.b, 0.5));

        let result = tone(blue, 0.5, 0.5);
        assert!(approx_eq(result.r, 0.25));
        assert!(approx_eq(result.g, 0.25));
        assert!(approx_eq(result.b, 0.75));
    }

    #[test]
    fn test_tone_with_different_grays() {
        let red = Rgb::new(1.0, 0.0, 0.0);
        let result = tone(red, 0.5, 0.2);
        assert!(approx_eq(result.r, 0.6));
        assert!(approx_eq(result.g, 0.1));
        assert!(approx_eq(result.b, 0.1));

        let result = tone(red, 0.5, 0.8);
        assert!(approx_eq(result.r, 0.9));
        assert!(approx_eq(result.g, 0.4));
        assert!(approx_eq(result.b, 0.4));
    }

    #[test]
    fn test_lighten_hsl() {
        let color = Hsl::new(240.0, 1.0, 0.3);
        let result = lighten_hsl(color, 0.2);
        assert!(approx_eq(result.h, 240.0));
        assert!(approx_eq(result.s, 1.0));
        assert!(approx_eq(result.l, 0.5));

        let result = lighten_hsl(color, 0.9);
        assert!(approx_eq(result.l, 1.0));
    }

    #[test]
    fn test_darken_hsl() {
        let color = Hsl::new(120.0, 0.8, 0.6);
        let result = darken_hsl(color, 0.2);
        assert!(approx_eq(result.h, 120.0));
        assert!(approx_eq(result.s, 0.8));
        assert!(approx_eq(result.l, 0.4));

        let result = darken_hsl(color, 0.9);
        assert!(approx_eq(result.l, 0.0));
    }

    #[test]
    fn test_desaturate_hsl() {
        let color = Hsl::new(60.0, 0.9, 0.5);

        let result = desaturate_hsl(color, 0.3);
        assert!(approx_eq(result.h, 60.0));
        assert!(approx_eq(result.s, 0.6));
        assert!(approx_eq(result.l, 0.5));

        let result = desaturate_hsl(color, 1.0);
        assert!(approx_eq(result.s, 0.0));
    }

    #[test]
    fn test_lighten_preserves_hue_and_saturation() {
        let color = Hsl::new(180.0, 0.7, 0.4);
        let result = lighten_hsl(color, 0.1);

        assert!(approx_eq(result.h, color.h));
        assert!(approx_eq(result.s, color.s));
    }

    #[test]
    fn test_darken_preserves_hue_and_saturation() {
        let color = Hsl::new(300.0, 0.5, 0.7);
        let result = darken_hsl(color, 0.2);

        assert!(approx_eq(result.h, color.h));
        assert!(approx_eq(result.s, color.s));
    }

    #[test]
    fn test_desaturate_preserves_hue_and_lightness() {
        let color = Hsl::new(45.0, 0.8, 0.6);
        let result = desaturate_hsl(color, 0.3);

        assert!(approx_eq(result.h, color.h));
        assert!(approx_eq(result.l, color.l));
    }

    #[test]
    fn test_gray_clamping_in_tone() {
        let color = Rgb::new(1.0, 0.0, 0.0);
        let result = tone(color, 0.5, -0.2);
        assert!(result.r >= 0.0 && result.r <= 1.0);

        let result = tone(color, 0.5, 1.5);
        assert!(result.r >= 0.0 && result.r <= 1.0);
    }
}
