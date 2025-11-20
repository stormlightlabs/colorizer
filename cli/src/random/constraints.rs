use crate::colors::{Hsl, Lab, Rgb, Srgb8, clamp01};
use crate::diffs::delta_e_2000;
use crate::harmonies::{HarmonyKind, harmonies};
use crate::random::random_hsl;
use crate::wcag::contrast_ratio;
use rand::seq::IndexedRandom;
use rand::{Rng, rngs::ThreadRng};
use std::ops::Range;

/// Constraints applied during random palette construction.
#[derive(Debug, Clone)]
pub struct PaletteConstraints {
    pub base: Option<Srgb8>,
    pub harmony: Option<HarmonyKind>,
    pub min_contrast: Option<f32>,
    pub background: Option<Srgb8>,
    pub min_delta_e: Option<f32>,
    pub saturation_range: Range<f32>,
    pub lightness_range: Range<f32>,
    pub max_iterations: usize,
}

impl Default for PaletteConstraints {
    fn default() -> Self {
        Self {
            base: None,
            harmony: None,
            min_contrast: None,
            background: None,
            min_delta_e: None,
            saturation_range: 0.2..0.9,
            lightness_range: 0.25..0.75,
            max_iterations: 1000,
        }
    }
}

/// Generates a palette that satisfies the provided constraints (best effort).
pub fn random_palette_with_constraints(count: usize, constraints: PaletteConstraints) -> Vec<Srgb8> {
    if count == 0 {
        return Vec::new();
    }

    let mut rng = rand::rng();
    let mut accepted: Vec<Srgb8> = Vec::with_capacity(count);
    let mut labs: Vec<Lab> = Vec::with_capacity(count);

    if let Some(base) = constraints.base {
        if passes_filters(base, &labs, &constraints) {
            labs.push(Lab::from(base));
            accepted.push(base);
        }
    }

    let mut iterations = 0;
    while accepted.len() < count && iterations < constraints.max_iterations {
        iterations += 1;
        let candidate = sample_candidate(&mut rng, &constraints);
        if passes_filters(candidate, &labs, &constraints) {
            labs.push(Lab::from(candidate));
            accepted.push(candidate);
        }
    }

    accepted
}

fn passes_filters(candidate: Srgb8, labs: &[Lab], constraints: &PaletteConstraints) -> bool {
    if let (Some(bg), Some(min_ratio)) = (constraints.background, constraints.min_contrast) {
        if contrast_ratio(bg, candidate) < min_ratio {
            return false;
        }
    }

    if let Some(threshold) = constraints.min_delta_e {
        if threshold > 0.0 {
            let candidate_lab = Lab::from(candidate);
            if labs.iter().any(|&lab| delta_e_2000(lab, candidate_lab) < threshold) {
                return false;
            }
        }
    }

    true
}

fn sample_candidate(rng: &mut ThreadRng, constraints: &PaletteConstraints) -> Srgb8 {
    if let (Some(base), Some(harmony_kind)) = (constraints.base, constraints.harmony) {
        let base_rgb = Rgb::from(base);
        let base_hsl: Hsl = base_rgb.into();
        let palette = harmonies(base_hsl, harmony_kind);
        if let Some(choice) = palette.choose(rng) {
            let jittered = jitter_hsl(*choice, rng);
            return Srgb8::from(Rgb::from(jittered));
        }
    }

    let hsl = random_hsl(
        rng,
        constraints.saturation_range.clone(),
        constraints.lightness_range.clone(),
    );
    Srgb8::from(Rgb::from(hsl))
}

fn jitter_hsl(mut color: Hsl, rng: &mut ThreadRng) -> Hsl {
    let sat_jitter = rng.random_range(-0.1..0.1);
    let light_jitter = rng.random_range(-0.1..0.1);
    color.s = clamp01(color.s + sat_jitter);
    color.l = clamp01(color.l + light_jitter);
    color
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_respects_delta_e() {
        let constraints = PaletteConstraints { min_delta_e: Some(5.0), ..Default::default() };
        let palette = random_palette_with_constraints(3, constraints);
        assert!(palette.len() <= 3);
        let labs: Vec<_> = palette.iter().copied().map(Lab::from).collect();
        for i in 0..labs.len() {
            for j in i + 1..labs.len() {
                assert!(delta_e_2000(labs[i], labs[j]) >= 5.0);
            }
        }
    }

    #[test]
    fn palette_respects_contrast() {
        let bg = Srgb8::new(240, 240, 240);
        let constraints = PaletteConstraints { background: Some(bg), min_contrast: Some(4.5), ..Default::default() };
        let palette = random_palette_with_constraints(2, constraints);
        for color in palette {
            assert!(contrast_ratio(bg, color) >= 4.5);
        }
    }
}
