use colors::{Hsl, Hsv, Rgb, clamp01};
use std::ops::Range;

mod conversions;
pub mod tinted_theming;
mod vimrc;

pub mod colors;
pub mod diffs;
pub mod palette;
pub mod random;
pub mod wcag;

pub mod harmonies;
pub use harmonies::{HarmonyKind, harmonies, normalize_saturation, set_lightness, shift_lightness};

pub mod shades;
pub use shades::{darken_hsl, desaturate_hsl, lighten_hsl, mix_rgb, shade, tint, tone};

pub mod interpolation;
pub use interpolation::{gradient_lab, gradient_lch, lerp_lab, lerp_lch, lerp_rgb};

/// Golden ratio conjugate used for hue stepping.
pub const GOLDEN_RATIO_CONJUGATE: f32 = 0.618_034;

/// Iterator-style helper that distributes hues by repeatedly adding the golden ratio conjugate.
#[derive(Debug, Clone, Copy, Default)]
pub struct GoldenHue {
    h: f32,
}

impl GoldenHue {
    /// Creates a new generator with the provided seed fraction (0–1).
    pub fn new(seed: f32) -> Self {
        Self { h: wrap_unit_interval(seed) }
    }

    /// Returns the next hue fraction in [0, 1) while advancing the internal state.
    pub fn next_hf(&mut self) -> f32 {
        let current = self.h;
        self.h = wrap_unit_interval(self.h + GOLDEN_RATIO_CONJUGATE);
        current
    }

    /// Returns the current hue fraction without advancing.
    pub fn peek(&self) -> f32 {
        self.h
    }
}

#[derive(Debug, Clone, Copy)]
enum GoldenValueSpec {
    Fixed(f32),
    Range { min: f32, max: f32 },
}

impl GoldenValueSpec {
    fn fixed(value: f32) -> Self {
        GoldenValueSpec::Fixed(clamp01(value))
    }

    fn from_range(range: Range<f32>) -> Self {
        let min = clamp01(range.start);
        let max = clamp01(range.end);
        if max <= min { GoldenValueSpec::Fixed(min) } else { GoldenValueSpec::Range { min, max } }
    }

    fn sample(&self, t: f32) -> f32 {
        match *self {
            GoldenValueSpec::Fixed(value) => value,
            GoldenValueSpec::Range { min, max } => min + (max - min) * t,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GoldenSpace {
    Hsl,
    Hsv,
}

/// Generates evenly distributed colors by repeatedly stepping hue via the golden ratio conjugate.
#[derive(Debug, Clone)]
pub struct GoldenPalette {
    hue: GoldenHue,
    saturation: GoldenValueSpec,
    lum_or_value: GoldenValueSpec,
    space: GoldenSpace,
}

impl GoldenPalette {
    /// Creates an HSL palette with fixed saturation/lightness values.
    pub fn hsl_fixed(seed: f32, saturation: f32, lightness: f32) -> Self {
        Self {
            hue: GoldenHue::new(seed),
            saturation: GoldenValueSpec::fixed(saturation),
            lum_or_value: GoldenValueSpec::fixed(lightness),
            space: GoldenSpace::Hsl,
        }
    }

    /// Creates an HSL palette that samples saturation/lightness from ranges using the hue fraction as a t parameter.
    pub fn hsl_ranged(seed: f32, saturation_range: Range<f32>, lightness_range: Range<f32>) -> Self {
        Self {
            hue: GoldenHue::new(seed),
            saturation: GoldenValueSpec::from_range(saturation_range),
            lum_or_value: GoldenValueSpec::from_range(lightness_range),
            space: GoldenSpace::Hsl,
        }
    }

    /// Creates an HSV palette with fixed saturation/value values.
    pub fn hsv_fixed(seed: f32, saturation: f32, value: f32) -> Self {
        Self {
            hue: GoldenHue::new(seed),
            saturation: GoldenValueSpec::fixed(saturation),
            lum_or_value: GoldenValueSpec::fixed(value),
            space: GoldenSpace::Hsv,
        }
    }

    /// Creates an HSV palette that samples saturation/value from ranges.
    pub fn hsv_ranged(seed: f32, saturation_range: Range<f32>, value_range: Range<f32>) -> Self {
        Self {
            hue: GoldenHue::new(seed),
            saturation: GoldenValueSpec::from_range(saturation_range),
            lum_or_value: GoldenValueSpec::from_range(value_range),
            space: GoldenSpace::Hsv,
        }
    }

    /// Returns the next color as HSL; panics if the palette was constructed for HSV.
    pub fn next_hsl(&mut self) -> Hsl {
        assert!(
            matches!(self.space, GoldenSpace::Hsl),
            "GoldenPalette::next_hsl called on HSV palette"
        );
        let (hue, saturation, lightness) = self.advance();
        Hsl::new(hue * 360.0, saturation, lightness)
    }

    /// Returns the next color as HSV; panics if the palette was constructed for HSL.
    pub fn next_hsv(&mut self) -> Hsv {
        assert!(
            matches!(self.space, GoldenSpace::Hsv),
            "GoldenPalette::next_hsv called on HSL palette"
        );
        let (hue, saturation, value) = self.advance();
        Hsv::new(hue * 360.0, saturation, value)
    }

    fn advance(&mut self) -> (f32, f32, f32) {
        let hue = self.hue.next_hf();
        let s = self.saturation.sample(hue);
        let lv = self.lum_or_value.sample(hue);
        (hue, s, lv)
    }
}

/// Convenience helper that returns `n` evenly spaced RGB colors for a fixed HSL saturation/lightness.
// TODO: Extend this helper to accept CLI-provided seeds and ranges once palette commands are wired up.
pub fn golden_palette(n: usize, saturation: f32, lightness: f32) -> Vec<Rgb> {
    let mut generator = GoldenPalette::hsl_fixed(0.0, saturation, lightness);
    let mut result = Vec::with_capacity(n);
    for _ in 0..n {
        let hsl = generator.next_hsl();
        result.push(Rgb::from(hsl));
    }
    result
}

fn wrap_unit_interval(x: f32) -> f32 {
    let mut value = x % 1.0;
    if value < 0.0 {
        value += 1.0;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_hue_wraps_into_unit_interval() {
        let mut golden = GoldenHue::new(0.95);
        assert!(golden.peek() >= 0.0 && golden.peek() < 1.0);
        let first = golden.next_hf();
        assert!((0.0..1.0).contains(&first));
        let second = golden.next_hf();
        assert!((0.0..1.0).contains(&second));
        assert_ne!(first, second);
    }

    #[test]
    fn golden_palette_fixed_hsl_keeps_sl_constant() {
        let mut palette = GoldenPalette::hsl_fixed(0.0, 0.6, 0.4);
        let first = palette.next_hsl();
        let second = palette.next_hsl();
        assert_eq!(first.s, 0.6);
        assert_eq!(first.l, 0.4);
        assert_eq!(second.s, 0.6);
        assert_eq!(second.l, 0.4);
        assert_ne!(first.h, second.h);
    }

    #[test]
    fn golden_palette_ranges_follow_hue_fraction() {
        let mut palette = GoldenPalette::hsl_ranged(0.0, 0.4..0.8, 0.3..0.7);
        let color = palette.next_hsl();
        assert!(color.s >= 0.4 && color.s <= 0.8);
        assert!(color.l >= 0.3 && color.l <= 0.7);
    }

    #[test]
    fn golden_palette_hsv_mode_produces_hsv_colors() {
        let mut palette = GoldenPalette::hsv_fixed(0.2, 0.5, 0.8);
        let hsv = palette.next_hsv();
        assert_eq!(hsv.s, 0.5);
        assert_eq!(hsv.v, 0.8);
        assert!(hsv.h >= 0.0 && hsv.h < 360.0);
    }

    #[test]
    fn golden_palette_helper_returns_rgb_values() {
        let colors = golden_palette(5, 0.6, 0.5);
        assert_eq!(colors.len(), 5);

        for color in colors {
            assert!(color.r >= 0.0 && color.r <= 1.0);
            assert!(color.g >= 0.0 && color.g <= 1.0);
            assert!(color.b >= 0.0 && color.b <= 1.0);
        }
    }
}
