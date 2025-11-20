use crate::colors::{Lab, Lch, Rgb, Srgb8};
use crate::diffs::delta_e_2000;
use rand::Rng;
use std::ops::Range;

/// Configuration for Poisson-disk sampling in Lch space.
#[derive(Debug, Clone)]
pub struct PoissonConfig {
    pub radius: f32,
    pub k: usize,
    pub l_range: Range<f32>,
    pub c_range: Range<f32>,
    pub h_range: Range<f32>,
}

impl Default for PoissonConfig {
    fn default() -> Self {
        Self { radius: 10.0, k: 30, l_range: 0.0..100.0, c_range: 0.0..120.0, h_range: 0.0..360.0 }
    }
}

/// Computes color distance in Lab space via ΔE2000.
pub fn distance_lab(a: Lab, b: Lab) -> f32 {
    delta_e_2000(a, b)
}

/// Generates a palette using Poisson-disk sampling in Lch space.
pub fn poisson_palette(config: PoissonConfig, max_samples: usize) -> Vec<Rgb> {
    if max_samples == 0 {
        return Vec::new();
    }

    let mut rng = rand::rng();
    let mut samples_lab: Vec<Lab> = Vec::new();
    let mut result: Vec<Rgb> = Vec::new();
    let mut active: Vec<usize> = Vec::new();

    if let Some((lab, rgb)) = random_point(&mut rng, &config) {
        samples_lab.push(lab);
        result.push(rgb);
        active.push(0);
    } else {
        return Vec::new();
    }

    while !active.is_empty() && result.len() < max_samples {
        let idx = rng.random_range(0..active.len());
        let sample_index = active[idx];
        let mut found = false;

        for _ in 0..config.k {
            if let Some(candidate_lab) = random_candidate_near(&mut rng, samples_lab[sample_index], &config) {
                if samples_lab
                    .iter()
                    .all(|&lab| distance_lab(lab, candidate_lab) >= config.radius)
                {
                    let rgb = Srgb8::from(candidate_lab);
                    samples_lab.push(candidate_lab);
                    result.push(Rgb::from(rgb));
                    active.push(samples_lab.len() - 1);
                    found = true;
                    break;
                }
            }
        }

        if !found {
            active.swap_remove(idx);
        }

        if result.len() >= max_samples {
            break;
        }
    }

    result
}

fn random_point(rng: &mut impl Rng, config: &PoissonConfig) -> Option<(Lab, Rgb)> {
    if !valid_range(&config.l_range) || !valid_range(&config.c_range) || !valid_range(&config.h_range) {
        return None;
    }
    let l = rng.random_range(config.l_range.start..config.l_range.end);
    let c = rng.random_range(config.c_range.start..config.c_range.end);
    let h = rng.random_range(config.h_range.start..config.h_range.end);
    let lch = Lch::new(l, c, h);
    let lab = Lab::from(lch);
    let rgb = Rgb::from(Srgb8::from(lab));
    Some((lab, rgb))
}

fn random_candidate_near(rng: &mut impl Rng, base: Lab, config: &PoissonConfig) -> Option<Lab> {
    if config.radius <= 0.0 {
        return None;
    }
    let dist = rng.random_range(config.radius..config.radius * 2.0);
    let theta = rng.random_range(0.0..std::f32::consts::TAU);
    let u: f32 = rng.random_range(-1.0..1.0);
    let sqrt1_minus_u2 = (1.0 - u * u).sqrt();

    let dl = dist * sqrt1_minus_u2 * theta.cos();
    let da = dist * sqrt1_minus_u2 * theta.sin();
    let db = dist * u;

    let candidate = Lab::new(base.l + dl, base.a + da, base.b + db);
    let lch = Lch::from(candidate);

    if !config.l_range.contains(&lch.l) || !config.c_range.contains(&lch.c) {
        return None;
    }

    let mut hue = lch.h;
    if hue < config.h_range.start {
        hue += 360.0;
    } else if hue > config.h_range.end {
        hue -= 360.0;
    }
    if !config.h_range.contains(&hue) {
        return None;
    }

    Some(candidate)
}

fn valid_range(range: &Range<f32>) -> bool {
    range.end > range.start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisson_palette_returns_requested_samples() {
        let config = PoissonConfig { radius: 5.0, ..Default::default() };
        let palette = poisson_palette(config, 5);
        assert!(!palette.is_empty());
        assert!(palette.len() <= 5);
    }
}
