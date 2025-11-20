//! Base16/Base24 scheme generation with semantic color role assignment.
//!
//! This module generates color schemes that adhere to the Base16/Base24 specification.

use crate::colors::{Hsl, Rgb, Srgb8};
use crate::harmonies::{HarmonyKind, harmonies};
use crate::tinted_theming::{Base16Scheme, Base24Scheme, SchemeMetadata};
use crate::wcag::contrast_ratio;

pub const NEUTRAL_MAX_SATURATION: f32 = 0.10;
pub const DEFAULT_NEUTRAL_DEPTH: f32 = 1.0;

const MIN_CONTRAST: f32 = 4.5;
const DARK_NEUTRAL_CLASSIC: [f32; 8] = [0.08, 0.13, 0.18, 0.30, 0.50, 0.90, 0.95, 0.98];
const DARK_NEUTRAL_MOODY: [f32; 8] = [0.008, 0.019, 0.033, 0.060, 0.100, 0.279, 0.456, 0.631];
const LIGHT_NEUTRAL_CLASSIC: [f32; 8] = [0.98, 0.95, 0.90, 0.70, 0.50, 0.18, 0.13, 0.08];
const LIGHT_NEUTRAL_MOODY: [f32; 8] = [0.95, 0.90, 0.80, 0.67, 0.54, 0.32, 0.20, 0.11];
const NEUTRAL_SAT_DEPTH_FACTOR: f32 = 1.0;

/// Theme variant determines background/foreground lightness progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Dark,
    Light,
}

impl Variant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Variant::Dark => "dark",
            Variant::Light => "light",
        }
    }
}

/// Configuration for Base16 scheme generation.
#[derive(Debug, Clone)]
pub struct Base16Config {
    pub name: String,
    pub author: Option<String>,
    pub variant: Variant,
    pub accent_color: Srgb8,
    pub harmony: HarmonyKind,
    pub neutral_depth: f32,
}

/// Configuration for Base24 scheme generation.
#[derive(Debug, Clone)]
pub struct Base24Config {
    pub name: String,
    pub author: Option<String>,
    pub variant: Variant,
    pub accent_color: Srgb8,
    pub harmony: HarmonyKind,
    pub neutral_depth: f32,
}

/// Generates a Base16 scheme from a single accent color using color harmonies.
///
/// The scheme follows Base16 semantic guidelines:
/// - base00-base07: neutral backgrounds and foregrounds (low saturation)
/// - base08-base0F: accent colors mapped to semantic roles (variables, strings, etc.)
pub fn generate_base16_scheme(config: Base16Config) -> Base16Scheme {
    let metadata = SchemeMetadata {
        system: "base16".to_string(),
        name: config.name,
        author: config.author,
        variant: Some(config.variant.as_str().to_string()),
    };

    let neutrals = generate_neutrals(config.variant, config.neutral_depth);
    let accent_hsl: Hsl = Rgb::from(config.accent_color).into();
    let accents = generate_accents(accent_hsl, config.harmony, neutrals[0], config.variant);

    let mut colors = [Srgb8::new(0, 0, 0); 16];
    for (i, &color) in neutrals.iter().enumerate() {
        colors[i] = color;
    }
    for (i, &color) in accents.iter().enumerate() {
        colors[i + 8] = color;
    }

    Base16Scheme::new(metadata, colors)
}

/// Generates a Base24 scheme from a single accent color using color harmonies.
///
/// Extends Base16 with base10-base17 as brighter/darker variants of accent colors.
pub fn generate_base24_scheme(config: Base24Config) -> Base24Scheme {
    let metadata = SchemeMetadata {
        system: "base24".to_string(),
        name: config.name,
        author: config.author,
        variant: Some(config.variant.as_str().to_string()),
    };

    let neutrals = generate_neutrals(config.variant, config.neutral_depth);
    let accent_hsl: Hsl = Rgb::from(config.accent_color).into();
    let accents = generate_accents(accent_hsl, config.harmony, neutrals[0], config.variant);
    let extended = generate_base24_extended(&neutrals, &accents, config.variant);

    let mut colors = [Srgb8::new(0, 0, 0); 24];
    for (i, &color) in neutrals.iter().enumerate() {
        colors[i] = color;
    }
    for (i, &color) in accents.iter().enumerate() {
        colors[i + 8] = color;
    }
    for (i, &color) in extended.iter().enumerate() {
        colors[i + 16] = color;
    }

    Base24Scheme::new(metadata, colors)
}

/// Generates 8 neutral colors (base00-base07) with low saturation.
///
/// Dark themes: base00 (darkest) → base07 (lightest)
/// Light themes: base00 (lightest) → base07 (darkest)
fn generate_neutrals(variant: Variant, neutral_depth: f32) -> [Srgb8; 8] {
    let depth = neutral_depth.clamp(0.0, 1.0);
    let (lightness_values, hue, saturation) = match variant {
        Variant::Dark => (
            blend_lightness_curve(&DARK_NEUTRAL_CLASSIC, &DARK_NEUTRAL_MOODY, depth),
            220.0,
            adjusted_neutral_saturation(NEUTRAL_MAX_SATURATION * 0.8, depth),
        ),
        Variant::Light => (
            blend_lightness_curve(&LIGHT_NEUTRAL_CLASSIC, &LIGHT_NEUTRAL_MOODY, depth),
            40.0,
            adjusted_neutral_saturation(NEUTRAL_MAX_SATURATION * 0.6, depth),
        ),
    };
    let saturation = saturation.min(NEUTRAL_MAX_SATURATION);

    let mut neutrals = [Srgb8::new(0, 0, 0); 8];
    for (i, &lightness) in lightness_values.iter().enumerate() {
        let hsl = Hsl::new(hue, saturation, lightness);
        let rgb: Rgb = hsl.into();
        neutrals[i] = Srgb8::from(rgb);
    }
    neutrals
}

fn blend_lightness_curve(base: &[f32; 8], moody: &[f32; 8], depth: f32) -> [f32; 8] {
    let mut result = [0.0; 8];
    for i in 0..8 {
        result[i] = base[i] + (moody[i] - base[i]) * depth;
    }
    result
}

fn adjusted_neutral_saturation(base: f32, depth: f32) -> f32 {
    let scaled = base * (1.0 - depth * NEUTRAL_SAT_DEPTH_FACTOR);
    scaled.max(0.0)
}

/// Generates 8 accent colors (base08-base0F) mapped to semantic roles.
///
/// Semantic roles:
/// - base08 (red): variables, tags
/// - base09 (orange): integers, booleans
/// - base0A (yellow): classes, search
/// - base0B (green): strings, insertions
/// - base0C (cyan): regex, escape chars
/// - base0D (blue): functions, headings
/// - base0E (magenta): keywords, storage
/// - base0F (brown): deprecated
fn generate_accents(base: Hsl, harmony: HarmonyKind, background: Srgb8, variant: Variant) -> [Srgb8; 8] {
    let harmony_colors = harmonies(base, harmony);
    let target_hues = [0.0, 30.0, 60.0, 120.0, 180.0, 220.0, 280.0, 20.0];

    let target_lightness = match variant {
        Variant::Dark => 0.65,
        Variant::Light => 0.45,
    };

    let target_saturation = match variant {
        Variant::Dark => 0.70,
        Variant::Light => 0.75,
    };

    let mut accents = [Srgb8::new(0, 0, 0); 8];

    let mut assigned = [false; 8];
    for harmony_color in harmony_colors {
        let closest_idx = find_closest_hue_index(&target_hues, harmony_color.h, &assigned);
        if let Some(idx) = closest_idx {
            let adjusted = Hsl::new(
                harmony_color.h,
                if idx == 7 { 0.35 } else { target_saturation },
                target_lightness,
            );
            accents[idx] = ensure_contrast(adjusted, background, variant);
            assigned[idx] = true;
        }
    }

    for (i, &is_assigned) in assigned.iter().enumerate() {
        if !is_assigned {
            let hsl = Hsl::new(
                target_hues[i],
                if i == 7 { 0.35 } else { target_saturation },
                target_lightness,
            );
            accents[i] = ensure_contrast(hsl, background, variant);
        }
    }

    accents
}

/// Generates 8 extended colors for Base24 (base10-base17).
///
/// Per Base24 spec:
/// - base10-base11: darker/darkest backgrounds
/// - base12-base17: brighter versions of base08-base0D
fn generate_base24_extended(neutrals: &[Srgb8; 8], accents: &[Srgb8; 8], variant: Variant) -> [Srgb8; 8] {
    let mut extended = [Srgb8::new(0, 0, 0); 8];

    match variant {
        Variant::Dark => {
            let base_hsl: Hsl = Rgb::from(neutrals[0]).into();
            extended[0] = Srgb8::from(Rgb::from(Hsl::new(
                base_hsl.h,
                base_hsl.s,
                (base_hsl.l - 0.03).max(0.0),
            )));
            extended[1] = Srgb8::from(Rgb::from(Hsl::new(
                base_hsl.h,
                base_hsl.s,
                (base_hsl.l - 0.05).max(0.0),
            )));
        }
        Variant::Light => {
            let base_hsl: Hsl = Rgb::from(neutrals[0]).into();
            extended[0] = Srgb8::from(Rgb::from(Hsl::new(
                base_hsl.h,
                base_hsl.s,
                (base_hsl.l + 0.02).min(1.0),
            )));
            extended[1] = Srgb8::from(Rgb::from(Hsl::new(
                base_hsl.h,
                base_hsl.s,
                (base_hsl.l + 0.03).min(1.0),
            )));
        }
    }

    for i in 0..6 {
        let accent_hsl: Hsl = Rgb::from(accents[i]).into();
        let brighter_l = match variant {
            Variant::Dark => (accent_hsl.l + 0.15).min(0.85),
            Variant::Light => (accent_hsl.l - 0.15).max(0.30),
        };
        extended[i + 2] = Srgb8::from(Rgb::from(Hsl::new(accent_hsl.h, accent_hsl.s, brighter_l)));
    }

    extended
}

/// Finds the closest unassigned target hue index.
fn find_closest_hue_index(targets: &[f32], hue: f32, assigned: &[bool]) -> Option<usize> {
    let mut best_idx = None;
    let mut best_distance = f32::MAX;

    for (i, &target) in targets.iter().enumerate() {
        if assigned[i] {
            continue;
        }
        let distance = hue_distance(hue, target);
        if distance < best_distance {
            best_distance = distance;
            best_idx = Some(i);
        }
    }

    best_idx
}

/// Calculates circular distance between hues (0-360 degrees).
fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

/// Ensures color meets minimum contrast ratio against background.
fn ensure_contrast(color: Hsl, background: Srgb8, variant: Variant) -> Srgb8 {
    let mut adjusted = color;
    let rgb: Rgb = adjusted.into();
    let mut current = Srgb8::from(rgb);

    let mut iterations = 0;
    while contrast_ratio(background, current) < MIN_CONTRAST && iterations < 20 {
        adjusted.l = match variant {
            Variant::Dark => (adjusted.l + 0.05).min(0.95),
            Variant::Light => (adjusted.l - 0.05).max(0.15),
        };
        let rgb: Rgb = adjusted.into();
        current = Srgb8::from(rgb);
        iterations += 1;
    }

    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base16_scheme_has_16_colors() {
        let config = Base16Config {
            name: "Test Dark".to_string(),
            author: None,
            variant: Variant::Dark,
            accent_color: Srgb8::new(229, 108, 117),
            harmony: HarmonyKind::Triadic,
            neutral_depth: DEFAULT_NEUTRAL_DEPTH,
        };
        let scheme = generate_base16_scheme(config);
        assert_eq!(scheme.colors().len(), 16);
    }

    #[test]
    fn base24_scheme_has_24_colors() {
        let config = Base24Config {
            name: "Test Light".to_string(),
            author: None,
            variant: Variant::Light,
            accent_color: Srgb8::new(52, 152, 219),
            harmony: HarmonyKind::Complementary,
            neutral_depth: DEFAULT_NEUTRAL_DEPTH,
        };
        let scheme = generate_base24_scheme(config);
        assert_eq!(scheme.colors().len(), 24);
    }

    #[test]
    fn neutrals_are_low_saturation() {
        let neutrals = generate_neutrals(Variant::Dark, DEFAULT_NEUTRAL_DEPTH);
        for color in neutrals {
            let hsl: Hsl = Rgb::from(color).into();
            assert!(
                hsl.s <= NEUTRAL_MAX_SATURATION + 0.02,
                "Neutral saturation {} exceeds max",
                hsl.s
            );
        }
    }

    #[test]
    fn dark_theme_base00_darker_than_base07() {
        let neutrals = generate_neutrals(Variant::Dark, DEFAULT_NEUTRAL_DEPTH);
        let base00: Hsl = Rgb::from(neutrals[0]).into();
        let base07: Hsl = Rgb::from(neutrals[7]).into();
        assert!(base00.l < base07.l, "Dark theme: base00 should be darker than base07");
    }

    #[test]
    fn light_theme_base00_lighter_than_base07() {
        let neutrals = generate_neutrals(Variant::Light, DEFAULT_NEUTRAL_DEPTH);
        let base00: Hsl = Rgb::from(neutrals[0]).into();
        let base07: Hsl = Rgb::from(neutrals[7]).into();
        assert!(base00.l > base07.l, "Light theme: base00 should be lighter than base07");
    }

    #[test]
    fn accents_meet_contrast_requirements() {
        let neutrals = generate_neutrals(Variant::Dark, DEFAULT_NEUTRAL_DEPTH);
        let base_hsl = Hsl::new(0.0, 0.7, 0.6);
        let accents = generate_accents(base_hsl, HarmonyKind::Triadic, neutrals[0], Variant::Dark);

        for accent in accents {
            let ratio = contrast_ratio(neutrals[0], accent);
            assert!(
                ratio >= MIN_CONTRAST,
                "Accent {:?} contrast ratio {} is below minimum {}",
                accent,
                ratio,
                MIN_CONTRAST
            );
        }
    }

    #[test]
    fn neutral_depth_controls_darkness() {
        let shallow = generate_neutrals(Variant::Dark, 0.0);
        let deep = generate_neutrals(Variant::Dark, 1.0);
        let shallow_l: Hsl = Rgb::from(shallow[0]).into();
        let deep_l: Hsl = Rgb::from(deep[0]).into();
        assert!(
            deep_l.l < shallow_l.l,
            "Expected deeper neutral depth to lower lightness ({} vs {})",
            deep_l.l,
            shallow_l.l
        );
    }

    #[test]
    fn neutral_depth_extreme_matches_expected_hexes() {
        let config_deep = Base16Config {
            name: "Depth Test".into(),
            author: None,
            variant: Variant::Dark,
            accent_color: Srgb8::new(97, 175, 239),
            harmony: HarmonyKind::Triadic,
            neutral_depth: 1.0,
        };
        let scheme = generate_base16_scheme(config_deep.clone());
        assert_eq!(scheme.colors()[0], Srgb8::new(0x16, 0x16, 0x16));
        assert_eq!(scheme.colors()[1], Srgb8::new(0x26, 0x26, 0x26));

        let mut config_light = config_deep;
        config_light.neutral_depth = 0.0;
        let scheme_light = generate_base16_scheme(config_light);
        assert_eq!(scheme_light.colors()[0], Srgb8::new(0x4d, 0x4f, 0x53));
    }
}
