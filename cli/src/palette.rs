//! Palette generation helpers

use crate::GoldenPalette;
use crate::colors::{Hsl, Rgb, Srgb8};
use crate::diffs::ensure_min_distance;
use crate::harmonies::{HarmonyKind, harmonies};
use crate::shades::{darken_hsl, lighten_hsl};
use crate::wcag::contrast_ratio;
use std::ops::Range;

const VARIATION_STEP: f32 = 0.08;

/// Generates a palette derived from `base` using the requested harmony.
///
/// The harmony colors are expanded by iteratively lightening/darkening rounds until `count` colors are produced.
/// Optional perceptual distance and contrast filters are applied if requested.
pub fn palette_from_base(
    base: Srgb8, harmony: HarmonyKind, count: usize, min_delta_e: Option<f32>, background: Option<Srgb8>,
    min_contrast: Option<f32>,
) -> Vec<Srgb8> {
    let base_hsl: Hsl = Rgb::from(base).into();
    let harmony_colors = harmonies(base_hsl, harmony);

    let mut generated: Vec<Srgb8> = Vec::with_capacity(count);
    let mut round = 0;

    while generated.len() < count {
        for &color in &harmony_colors {
            if generated.len() == count {
                break;
            }
            let adjusted = apply_variation(color, round);
            let rgb: Rgb = adjusted.into();
            generated.push(Srgb8::from(rgb));
        }
        round += 1;
    }

    let filtered = enforce_min_delta_e(generated, min_delta_e);
    filter_by_contrast(filtered, background, min_contrast)
}

/// Creates a palette by stepping hue via the golden ratio conjugate and sampling saturation/lightness from provided ranges.
pub fn golden_ratio_palette(
    count: usize, saturation_range: Range<f32>, lightness_range: Range<f32>, min_delta_e: Option<f32>,
) -> Vec<Srgb8> {
    let mut generator = GoldenPalette::hsl_ranged(0.0, saturation_range, lightness_range);
    let mut colors = Vec::with_capacity(count);
    for _ in 0..count {
        let hsl = generator.next_hsl();
        let rgb: Rgb = hsl.into();
        colors.push(Srgb8::from(rgb));
    }
    enforce_min_delta_e(colors, min_delta_e)
}

fn apply_variation(color: Hsl, round: usize) -> Hsl {
    if round == 0 {
        return color;
    }
    let amount = VARIATION_STEP * round as f32;
    if round % 2 == 0 { lighten_hsl(color, amount) } else { darken_hsl(color, amount) }
}

fn enforce_min_delta_e(colors: Vec<Srgb8>, min_delta_e: Option<f32>) -> Vec<Srgb8> {
    let Some(threshold) = min_delta_e else {
        return colors;
    };

    if threshold <= 0.0 || colors.len() <= 1 {
        return colors;
    }

    let mut labs: Vec<_> = colors.iter().copied().map(crate::colors::Lab::from).collect();
    ensure_min_distance(&mut labs, threshold);
    labs.into_iter().map(Srgb8::from).collect()
}

fn filter_by_contrast(colors: Vec<Srgb8>, background: Option<Srgb8>, min_contrast: Option<f32>) -> Vec<Srgb8> {
    match (background, min_contrast) {
        (Some(bg), Some(required)) if required > 0.0 => colors
            .into_iter()
            .filter(|&color| contrast_ratio(bg, color) >= required)
            .collect(),
        _ => colors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::Srgb8;

    #[test]
    fn palette_from_base_generates_requested_count() {
        let base = Srgb8::new(255, 128, 0);
        let palette = palette_from_base(base, HarmonyKind::Complementary, 5, None, None, None);
        assert_eq!(palette.len(), 5);
    }

    #[test]
    fn palette_from_base_enforces_contrast_when_requested() {
        let base = Srgb8::new(200, 200, 200);
        let background = Srgb8::new(180, 180, 180);
        let palette = palette_from_base(base, HarmonyKind::Analogous(20.0), 4, None, Some(background), Some(4.5));
        for color in palette {
            assert!(contrast_ratio(background, color) >= 4.5);
        }
    }

    #[test]
    fn golden_ratio_palette_respects_min_delta_e() {
        let palette = golden_ratio_palette(6, 0.5..0.8, 0.4..0.6, Some(2.0));
        assert!(palette.len() <= 6);
        if palette.len() > 1 {
            let labs: Vec<_> = palette.iter().copied().map(crate::colors::Lab::from).collect();
            for pair in labs.windows(2) {
                let delta = crate::diffs::delta_e_2000(pair[0], pair[1]);
                assert!(delta >= 2.0);
            }
        }
    }
}
