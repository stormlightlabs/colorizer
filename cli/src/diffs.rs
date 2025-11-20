//! Color difference metrics (ΔE) and helper utilities.
//!
//! Implements the most common perceptual color distance formulas operating on
//! Lab colors:
//! - ΔE76 (Euclidean distance)
//! - ΔE94 (graphics/textiles variants)
//! - ΔE2000 (CIEDE2000)
//! Supporting helpers for "just noticeable difference" checks and enforcing a
//! minimum perceptual spacing within color collections.

use crate::colors::{Lab, wrap_degrees};

/// Default ΔE threshold commonly cited as the "just noticeable difference".
pub const DEFAULT_JND_THRESHOLD: f32 = 2.3;

/// Computes the original CIE76 ΔE as simple Euclidean distance in Lab space.
pub fn delta_e_76(a: Lab, b: Lab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

/// Computes the CIE94 ΔE with separate tuning constants for graphics/textiles.
pub fn delta_e_94(a: Lab, b: Lab, is_textiles: bool) -> f32 {
    let (k_l, k1, k2) = if is_textiles { (2.0, 0.048, 0.014) } else { (1.0, 0.045, 0.015) };
    let k_c = 1.0;
    let k_h = 1.0;

    let c1 = (a.a * a.a + a.b * a.b).sqrt();
    let c2 = (b.a * b.a + b.b * b.b).sqrt();
    let delta_l = b.l - a.l;
    let delta_a = b.a - a.a;
    let delta_b = b.b - a.b;
    let delta_c = c2 - c1;
    let delta_h_sq = (delta_a * delta_a + delta_b * delta_b - delta_c * delta_c).max(0.0);
    let delta_h = delta_h_sq.sqrt();

    let c_bar = (c1 + c2) * 0.5;
    let s_l = 1.0;
    let s_c = 1.0 + k1 * c_bar;
    let s_h = 1.0 + k2 * c_bar;

    let l_term = delta_l / (k_l * s_l);
    let c_term = delta_c / (k_c * s_c);
    let h_term = delta_h / (k_h * s_h);

    (l_term * l_term + c_term * c_term + h_term * h_term).sqrt()
}

/// Computes the CIEDE2000 (ΔE2000) color difference.
pub fn delta_e_2000(a: Lab, b: Lab) -> f32 {
    let k_l = 1.0;
    let k_c = 1.0;
    let k_h = 1.0;

    let c1 = (a.a * a.a + a.b * a.b).sqrt();
    let c2 = (b.a * b.a + b.b * b.b).sqrt();
    let c_bar = (c1 + c2) * 0.5;
    let c_bar_pow7 = c_bar.powi(7);
    let twenty_five_pow7 = 25_f32.powi(7);

    let g = if c_bar == 0.0 { 0.0 } else { 0.5 * (1.0 - c_bar_pow7 / (c_bar_pow7 + twenty_five_pow7)) };

    let a1_prime = (1.0 + g) * a.a;
    let a2_prime = (1.0 + g) * b.a;
    let c1_prime = (a1_prime * a1_prime + a.b * a.b).sqrt();
    let c2_prime = (a2_prime * a2_prime + b.b * b.b).sqrt();

    let h1_prime = if c1_prime == 0.0 { 0.0 } else { wrap_degrees(a.b.atan2(a1_prime).to_degrees()) };
    let h2_prime = if c2_prime == 0.0 { 0.0 } else { wrap_degrees(b.b.atan2(a2_prime).to_degrees()) };

    let delta_l_prime = b.l - a.l;
    let delta_c_prime = c2_prime - c1_prime;

    let mut delta_h_prime = h2_prime - h1_prime;
    if c1_prime * c2_prime == 0.0 {
        delta_h_prime = 0.0;
    } else if delta_h_prime > 180.0 {
        delta_h_prime -= 360.0;
    } else if delta_h_prime < -180.0 {
        delta_h_prime += 360.0;
    }

    let delta_h_prime_rad = (delta_h_prime * 0.5).to_radians();
    let delta_h_cap = 2.0 * (c1_prime * c2_prime).sqrt() * delta_h_prime_rad.sin();

    let l_bar_prime = (a.l + b.l) * 0.5;
    let c_bar_prime = (c1_prime + c2_prime) * 0.5;

    let h_bar_prime = if c1_prime * c2_prime == 0.0 {
        h1_prime + h2_prime
    } else {
        let h_diff = (h1_prime - h2_prime).abs();
        let h_sum = h1_prime + h2_prime;
        if h_diff > 180.0 {
            if h_sum < 360.0 { (h_sum + 360.0) * 0.5 } else { (h_sum - 360.0) * 0.5 }
        } else {
            h_sum * 0.5
        }
    };

    let t = 1.0 - 0.17 * (h_bar_prime - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar_prime).to_radians().cos()
        + 0.32 * (3.0 * h_bar_prime + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar_prime - 63.0).to_radians().cos();

    let delta_theta = 30.0 * (-((h_bar_prime - 275.0) / 25.0).powi(2)).exp();
    let r_c = 2.0 * (c_bar_prime.powi(7) / (c_bar_prime.powi(7) + twenty_five_pow7)).sqrt();
    let s_l = 1.0 + (0.015 * (l_bar_prime - 50.0).powi(2)) / (20.0 + (l_bar_prime - 50.0).powi(2)).sqrt();
    let s_c = 1.0 + 0.045 * c_bar_prime;
    let s_h = 1.0 + 0.015 * c_bar_prime * t;
    let r_t = -((2.0 * delta_theta.to_radians()).sin()) * r_c;

    let l_term = delta_l_prime / (k_l * s_l);
    let c_term = delta_c_prime / (k_c * s_c);
    let h_term = delta_h_cap / (k_h * s_h);

    (l_term * l_term + c_term * c_term + h_term * h_term + r_t * c_term * h_term).sqrt()
}

/// Returns true if the given ΔE exceeds the supplied or default JND threshold.
// TODO: Expose CLI options that let users enforce or inspect minimum ΔE values via this helper.
pub fn is_just_noticeable(delta_e: f32, threshold: f32) -> bool {
    delta_e >= threshold
}

/// Removes colors that are closer than the requested ΔE (using ΔE2000).
// TODO: Integrate this guard into future palette/random generators to keep sampled colors perceptually distinct.
pub fn ensure_min_distance(colors: &mut Vec<Lab>, min_delta_e: f32) {
    if colors.len() <= 1 || min_delta_e <= 0.0 {
        return;
    }

    let mut filtered: Vec<Lab> = Vec::with_capacity(colors.len());
    'outer: for &candidate in colors.iter() {
        for existing in &filtered {
            if delta_e_2000(*existing, candidate) < min_delta_e {
                continue 'outer;
            }
        }
        filtered.push(candidate);
    }

    colors.clear();
    colors.extend(filtered);
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn lab(l: f32, a: f32, b: f32) -> Lab {
        Lab::new(l, a, b)
    }

    #[test]
    fn delta_e_76_matches_euclidean_distance() {
        let a = lab(50.0, 50.0, 50.0);
        let b = lab(60.0, 60.0, 65.0);
        let expected = ((10.0_f32).powi(2) + (10.0_f32).powi(2) + (15.0_f32).powi(2)).sqrt();
        assert!((delta_e_76(a, b) - expected).abs() < 1e-6);
    }

    #[test]
    fn delta_e_94_respects_kl_factor() {
        let a = lab(50.0, 0.0, 0.0);
        let b = lab(60.0, 0.0, 0.0);
        assert!((delta_e_94(a, b, false) - 10.0).abs() < 1e-6);
        assert!((delta_e_94(a, b, true) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn delta_e_2000_matches_reference_case() {
        let a = lab(50.0, 2.6772, -79.7751);
        let b = lab(50.0, 0.0, -82.7485);
        let diff = delta_e_2000(a, b);
        assert!((diff - 2.0425).abs() < 1e-4);
    }

    #[test]
    fn just_noticeable_difference_helper() {
        assert!(is_just_noticeable(3.0, DEFAULT_JND_THRESHOLD));
        assert!(!is_just_noticeable(1.0, DEFAULT_JND_THRESHOLD));
    }

    #[test]
    fn ensure_min_distance_filters_close_colors() {
        let mut colors = vec![lab(50.0, 0.0, 0.0), lab(50.1, 0.0, 0.0), lab(60.0, 40.0, 30.0)];
        ensure_min_distance(&mut colors, 2.0);
        assert_eq!(colors.len(), 2);
        assert!(delta_e_2000(colors[0], colors[1]) >= 2.0);
    }
}
