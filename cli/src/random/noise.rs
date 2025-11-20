use crate::colors::{Lab, Lch, Rgb, Srgb8, wrap_degrees};
use rand::Rng;

/// Lightweight trait representing a 1D noise source returning values in [0, 1].
pub trait NoiseSource: Send + Sync {
    fn noise(&self, x: f32) -> f32;
}

/// Hash-based gradient noise suitable for palette modulation.
#[derive(Debug, Clone)]
pub struct HashNoise {
    seed: u32,
}

impl HashNoise {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }
}

impl Default for HashNoise {
    fn default() -> Self {
        Self { seed: 0xdecafbad }
    }
}

impl NoiseSource for HashNoise {
    fn noise(&self, x: f32) -> f32 {
        let xi = x.floor() as i32;
        let xf = x - xi as f32;
        let v1 = hash(xi, self.seed);
        let v2 = hash(xi + 1, self.seed);
        let smooth = smoothstep(xf);
        v1 * (1.0 - smooth) + v2 * smooth
    }
}

fn hash(x: i32, seed: u32) -> f32 {
    let mut v = x as u32;
    v = v.wrapping_mul(0x45d9f3b);
    v = v.rotate_left(7) ^ seed;
    v = v.wrapping_mul(0x45d9f3b);
    v ^= v >> 16;
    (v & 0xffff) as f32 / 0xffff as f32
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Performs a random walk in Lch space using Gaussian perturbations.
pub fn random_walk_lch<R: Rng + ?Sized>(rng: &mut R, seed: Lch, steps: usize, sigmas: (f32, f32, f32)) -> Vec<Lch> {
    let mut colors = Vec::with_capacity(steps);
    let mut current = seed;
    for _ in 0..steps {
        current.l = (current.l + gaussian(rng, sigmas.0)).clamp(0.0, 100.0);
        current.c = (current.c + gaussian(rng, sigmas.1)).max(0.0);
        current.h = wrap_degrees(current.h + gaussian(rng, sigmas.2));
        colors.push(current);
    }
    colors
}

/// Generates a palette by modulating an Lch base with the provided noise source.
pub fn noise_palette<N: NoiseSource>(n: usize, base: Lch, spread: f32, freq: f32, noise: &N) -> Vec<Rgb> {
    if n == 0 {
        return Vec::new();
    }

    let mut colors = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / n as f32;
        let hue_noise = noise.noise(freq * t) - 0.5;
        let chroma_noise = noise.noise(freq * t + 37.0) - 0.5;
        let light_noise = noise.noise(freq * t + 73.0) - 0.5;

        let mut lch = base;
        lch.h = wrap_degrees(lch.h + hue_noise * spread * 360.0);
        lch.c = (lch.c + chroma_noise * spread * 50.0).max(0.0);
        lch.l = (lch.l + light_noise * spread * 30.0).clamp(0.0, 100.0);

        let lab = Lab::from(lch);
        colors.push(Rgb::from(Srgb8::from(lab)));
    }
    colors
}

fn gaussian<R: Rng + ?Sized>(rng: &mut R, sigma: f32) -> f32 {
    if sigma.abs() < f32::EPSILON {
        return 0.0;
    }
    let mut u1: f32 = rng.random_range(0.0f32..1.0f32);
    u1 = u1.clamp(1e-6, 0.999_999);
    let u2: f32 = rng.random_range(0.0f32..1.0f32);
    let mag = (-2.0f32 * u1.ln()).sqrt();
    let z = mag * (2.0f32 * std::f32::consts::PI * u2).cos();
    z * sigma
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_noise_outputs_unit_interval() {
        let noise = HashNoise::new(42);
        for i in 0..100 {
            let v = noise.noise(i as f32 * 0.1);
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    #[test]
    fn random_walk_has_requested_length() {
        let mut rng = rand::rng();
        let seed = Lch::new(50.0, 30.0, 120.0);
        let walk = random_walk_lch(&mut rng, seed, 10, (1.0, 1.0, 5.0));
        assert_eq!(walk.len(), 10);
    }
}
