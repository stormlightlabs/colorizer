//! Core color type definitions and helper utilities.
//!
//! Provides fundamental color representations used throughout the colorizer library:
//! - sRGB (8-bit and float)
//! - Linear RGB
//! - HSL and HSV (cylindrical color spaces)
//! - CIE Lab and Lch (perceptually uniform spaces)

use std::fmt;

/// Linear RGB color with components in [0, 1] range.
///
/// This represents color in linear light space, commonly used for physically-based rendering and blending operations.
/// Each component represents the actual light intensity without gamma correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    /// Creates a new linear RGB color, clamping components to [0, 1].
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: clamp01(r),
            g: clamp01(g),
            b: clamp01(b),
        }
    }
}

/// sRGB color with 8-bit components in [0, 255] range.
///
/// Standard RGB color space used in web and most display devices.
/// Values represent gamma-corrected intensities suitable for direct display without additional transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Srgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Srgb8 {
    /// Creates a new 8-bit sRGB color.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parses a hex color string in format "#RRGGBB" or "RRGGBB".
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self::new(r, g, b))
    }

    /// Converts to hex string format "#RRGGBB".
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl fmt::Display for Srgb8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// HSL (Hue, Saturation, Lightness) color representation.
///
/// Cylindrical color space where:
/// - `h` is hue in degrees [0, 360)
/// - `s` is saturation in [0, 1] (0 = gray, 1 = full color)
/// - `l` is lightness in [0, 1] (0 = black, 0.5 = pure color, 1 = white)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Hsl {
    /// Creates a new HSL color with normalized values.
    ///
    /// Hue is wrapped to [0, 360) and saturation/lightness are clamped to [0, 1].
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: wrap_degrees(h),
            s: clamp01(s),
            l: clamp01(l),
        }
    }
}

/// HSV (Hue, Saturation, Value) color representation.
///
/// Cylindrical color space where:
/// - `h` is hue in degrees [0, 360)
/// - `s` is saturation in [0, 1] (0 = white, 1 = full color)
/// - `v` is value/brightness in [0, 1] (0 = black, 1 = full brightness)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    /// Creates a new HSV color with normalized values.
    ///
    /// Hue is wrapped to [0, 360) and saturation/value are clamped to [0, 1].
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self {
            h: wrap_degrees(h),
            s: clamp01(s),
            v: clamp01(v),
        }
    }
}

/// CIE Lab color representation (perceptually uniform).
///
/// Device-independent color space designed to approximate human vision:
/// - `l` is lightness [0, 100] (0 = black, 100 = white)
/// - `a` is green-red axis (negative = green, positive = red)
/// - `b` is blue-yellow axis (negative = blue, positive = yellow)
///
/// Lab is perceptually uniform, meaning equal distances in Lab space correspond to roughly equal perceived color differences.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Lab {
    /// Creates a new CIE Lab color.
    ///
    /// NOTE: No clamping is performed as a and b can have wide ranges depending on the color.
    /// L is typically in [0, 100] but can exceed this range for very bright colors.
    pub const fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }
}

/// CIE Lch color representation (cylindrical Lab).
///
/// Cylindrical transformation of Lab where:
/// - `l` is lightness [0, 100] (same as Lab)
/// - `c` is chroma/saturation [0, ∞) (distance from gray axis)
/// - `h` is hue angle in degrees [0, 360) (color angle)
///
/// Lch is useful for operations like hue rotation while maintaining perceptual uniformity.
/// Chroma represents colorfulness independent of lightness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Lch {
    /// Creates a new Lch color with normalized hue.
    ///
    /// Hue is wrapped to [0, 360).
    /// L and C are not clamped to allow for out-of-gamut colors that may be brought into gamut later.
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self {
            l,
            c,
            h: wrap_degrees(h),
        }
    }
}

/// Wraps an angle in degrees to the range [0, 360).
///
/// Handles negative angles and angles greater than 360 by using modulo arithmetic to bring them into the standard range.
///
/// # Examples
///
/// ```
/// assert_eq!(wrap_degrees(370.0), 10.0);
/// assert_eq!(wrap_degrees(-10.0), 350.0);
/// ```
pub fn wrap_degrees(h: f32) -> f32 {
    let mut h = h % 360.0;
    if h < 0.0 {
        h += 360.0;
    }
    h
}

/// Clamps a value to the range [0, 1].
///
/// Values below 0 are clamped to 0, values above 1 are clamped to 1.
/// NaN values are clamped to 0.
///
/// # Examples
///
/// ```
/// assert_eq!(clamp01(0.5), 0.5);
/// assert_eq!(clamp01(-0.1), 0.0);
/// assert_eq!(clamp01(1.5), 1.0);
/// ```
pub fn clamp01(x: f32) -> f32 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else if x.is_nan() {
        0.0
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_degrees() {
        assert_eq!(wrap_degrees(0.0), 0.0);
        assert_eq!(wrap_degrees(180.0), 180.0);
        assert_eq!(wrap_degrees(359.0), 359.0);
        assert_eq!(wrap_degrees(360.0), 0.0);
        assert_eq!(wrap_degrees(370.0), 10.0);
        assert_eq!(wrap_degrees(-10.0), 350.0);
        assert_eq!(wrap_degrees(-370.0), 350.0);
    }

    #[test]
    fn test_clamp01() {
        assert_eq!(clamp01(0.0), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(1.0), 1.0);
        assert_eq!(clamp01(-0.1), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(clamp01(f32::NAN), 0.0);
    }

    #[test]
    fn test_srgb8_hex_parsing() {
        let color = Srgb8::from_hex("#ff0000").unwrap();
        assert_eq!(color, Srgb8::new(255, 0, 0));

        let color = Srgb8::from_hex("00ff00").unwrap();
        assert_eq!(color, Srgb8::new(0, 255, 0));

        assert!(Srgb8::from_hex("invalid").is_none());
        assert!(Srgb8::from_hex("#fff").is_none());
    }

    #[test]
    fn test_srgb8_hex_formatting() {
        let color = Srgb8::new(255, 128, 0);
        assert_eq!(color.to_hex(), "#ff8000");
        assert_eq!(format!("{color}"), "#ff8000");
    }

    #[test]
    fn test_rgb_clamping() {
        let color = Rgb::new(-0.1, 0.5, 1.5);
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 1.0);
    }

    #[test]
    fn test_hsl_normalization() {
        let color = Hsl::new(370.0, 1.5, -0.1);
        assert_eq!(color.h, 10.0);
        assert_eq!(color.s, 1.0);
        assert_eq!(color.l, 0.0);
    }

    #[test]
    fn test_hsv_normalization() {
        let color = Hsv::new(-10.0, 1.5, -0.1);
        assert_eq!(color.h, 350.0);
        assert_eq!(color.s, 1.0);
        assert_eq!(color.v, 0.0);
    }

    #[test]
    fn test_lch_hue_wrapping() {
        let color = Lch::new(50.0, 30.0, 400.0);
        assert_eq!(color.h, 40.0);
    }
}
