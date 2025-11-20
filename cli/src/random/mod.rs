//! Random palette generation utilities.
//!
//! This module ties together several sampling strategies:
//! - Uniform/random HSL sampling helpers
//! - Contrast-aware random color pickers
//! - Constraint-driven palette construction
//! - Poisson-disk sampling in perceptual color spaces
//! - Noise-driven palette walks

use crate::colors::{Hsl, Rgb, Srgb8, clamp01};
use crate::wcag::contrast_ratio;
use rand::Rng;
use std::ops::Range;

pub mod constraints;
pub mod noise;
pub mod poisson;

pub use constraints::{PaletteConstraints, random_palette_with_constraints};
pub use noise::{HashNoise, NoiseSource, noise_palette, random_walk_lch};
pub use poisson::{PoissonConfig, poisson_palette};

/// Simple theme hint used by helpers when sampling background colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightOrDark {
    Light,
    Dark,
}

/// Generates a random HSL color using the provided saturation and lightness ranges.
pub fn random_hsl<R: Rng + ?Sized>(rng: &mut R, s_range: Range<f32>, l_range: Range<f32>) -> Hsl {
    let hue = rng.random_range(0.0..360.0);
    let saturation = sample_range_clamped(rng, s_range);
    let lightness = sample_range_clamped(rng, l_range);
    Hsl::new(hue, saturation, lightness)
}

/// Returns a saturated UI accent color sampled from visually pleasing ranges.
pub fn random_ui_accent_color() -> Rgb {
    let mut rng = rand::rng();
    let hsl = random_hsl(&mut rng, 0.4..0.9, 0.35..0.7);
    Rgb::from(hsl)
}

/// Samples a background color biased toward the requested theme brightness.
pub fn random_background_color(theme: LightOrDark) -> Rgb {
    let mut rng = rand::rng();
    match theme {
        LightOrDark::Light => Rgb::from(random_hsl(&mut rng, 0.05..0.25, 0.75..0.95)),
        LightOrDark::Dark => Rgb::from(random_hsl(&mut rng, 0.1..0.4, 0.05..0.25)),
    }
}

/// Samples random colors until one meets the requested contrast ratio with `bg`.
pub fn sample_contrasting_color(bg: Srgb8, min_ratio: f32, max_attempts: usize) -> Option<Srgb8> {
    if min_ratio <= 0.0 {
        return Some(bg);
    }
    let mut rng = rand::rng();
    for _ in 0..max_attempts.max(1) {
        let hsl = random_hsl(&mut rng, 0.2..0.9, 0.1..0.9);
        let candidate = Srgb8::from(Rgb::from(hsl));
        if contrast_ratio(bg, candidate) >= min_ratio {
            return Some(candidate);
        }
    }
    None
}

fn sample_range_clamped<R: Rng + ?Sized>(rng: &mut R, range: Range<f32>) -> f32 {
    if range.end <= range.start {
        return clamp01(range.start);
    }
    clamp01(rng.random_range(range.start..range.end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_hsl_respects_ranges() {
        let mut rng = rand::rng();
        let color = random_hsl(&mut rng, 0.2..0.3, 0.4..0.5);
        assert!(color.s >= 0.2 && color.s <= 0.3);
        assert!(color.l >= 0.4 && color.l <= 0.5);
    }

    #[test]
    fn contrasting_color_respects_ratio() {
        let bg = Srgb8::new(250, 250, 250);
        let fg = sample_contrasting_color(bg, 3.0, 100).expect("should find color");
        assert!(contrast_ratio(bg, fg) >= 3.0);
    }
}
