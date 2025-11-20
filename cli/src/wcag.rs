//! WCAG (Web Content Accessibility Guidelines) color contrast utilities.
//!
//! Implements relative luminance and contrast ratio calculations per WCAG 2.1 specification.
//! Used for ensuring text and UI elements meet accessibility standards for readability.

use crate::colors::{Rgb, Srgb8};

/// WCAG AA minimum contrast ratio for normal text.
pub const WCAG_AA_NORMAL: f32 = 4.5;

/// WCAG AA minimum contrast ratio for large text (18pt+ or 14pt+ bold).
pub const WCAG_AA_LARGE: f32 = 3.0;

/// WCAG AAA minimum contrast ratio for normal text.
pub const WCAG_AAA_NORMAL: f32 = 7.0;

/// WCAG AAA minimum contrast ratio for large text (18pt+ or 14pt+ bold).
pub const WCAG_AAA_LARGE: f32 = 4.5;

/// Calculates relative luminance of an sRGB color per WCAG specification.
///
/// Relative luminance is the perceived brightness of a color normalized to [0, 1],
/// where 0 is black and 1 is white. This calculation follows the WCAG 2.1 formula:
/// L = 0.2126 * R + 0.7152 * G + 0.0722 * B
///
/// The input color is first converted to linear RGB (gamma-decoded) before applying
/// the luminance formula, as required by the WCAG specification.
///
/// # Examples
///
/// ```
/// use colorizer::colors::Srgb8;
/// use colorizer::wcag::relative_luminance;
///
/// let white = Srgb8::new(255, 255, 255);
/// assert_eq!(relative_luminance(white), 1.0);
///
/// let black = Srgb8::new(0, 0, 0);
/// assert_eq!(relative_luminance(black), 0.0);
/// ```
pub fn relative_luminance(color: Srgb8) -> f32 {
    let linear: Rgb = color.into();
    0.2126 * linear.r + 0.7152 * linear.g + 0.0722 * linear.b
}

/// Calculates the contrast ratio between two colors per WCAG specification.
///
/// The contrast ratio is calculated as (L1 + 0.05) / (L2 + 0.05), where L1 is the
/// relative luminance of the lighter color and L2 is the relative luminance of the
/// darker color. The ratio ranges from 1:1 (no contrast) to 21:1 (maximum contrast).
///
/// WCAG 2.1 requirements:
/// - AA Normal text: 4.5:1
/// - AA Large text: 3:1
/// - AAA Normal text: 7:1
/// - AAA Large text: 4.5:1
///
/// # Examples
///
/// ```
/// use colorizer::colors::Srgb8;
/// use colorizer::wcag::contrast_ratio;
///
/// let white = Srgb8::new(255, 255, 255);
/// let black = Srgb8::new(0, 0, 0);
/// let ratio = contrast_ratio(white, black);
/// assert!(ratio > 20.0); // Maximum contrast is 21:1
/// ```
pub fn contrast_ratio(c1: Srgb8, c2: Srgb8) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };

    (lighter + 0.05) / (darker + 0.05)
}

/// Checks if the contrast ratio meets WCAG AA standards for normal text (4.5:1).
///
/// # Examples
///
/// ```
/// use colorizer::colors::Srgb8;
/// use colorizer::wcag::{contrast_ratio, meets_aa_normal};
///
/// let white = Srgb8::new(255, 255, 255);
/// let dark_gray = Srgb8::new(100, 100, 100);
/// let ratio = contrast_ratio(white, dark_gray);
/// assert!(meets_aa_normal(ratio));
/// ```
pub fn meets_aa_normal(contrast: f32) -> bool {
    contrast >= WCAG_AA_NORMAL
}

/// Checks if the contrast ratio meets WCAG AA standards for large text (3:1).
///
/// Large text is defined as 18pt+ (24px+) or 14pt+ (18.66px+) bold.
pub fn meets_aa_large(contrast: f32) -> bool {
    contrast >= WCAG_AA_LARGE
}

/// Checks if the contrast ratio meets WCAG AAA standards for normal text (7:1).
///
/// # Examples
///
/// ```
/// use colorizer::colors::Srgb8;
/// use colorizer::wcag::{contrast_ratio, meets_aaa_normal};
///
/// let white = Srgb8::new(255, 255, 255);
/// let black = Srgb8::new(0, 0, 0);
/// let ratio = contrast_ratio(white, black);
/// assert!(meets_aaa_normal(ratio));
/// ```
pub fn meets_aaa_normal(contrast: f32) -> bool {
    contrast >= WCAG_AAA_NORMAL
}

/// Checks if the contrast ratio meets WCAG AAA standards for large text (4.5:1).
///
/// Large text is defined as 18pt+ (24px+) or 14pt+ (18.66px+) bold.
pub fn meets_aaa_large(contrast: f32) -> bool {
    contrast >= WCAG_AAA_LARGE
}

/// Selects the best foreground color from candidates that meets minimum contrast with background.
///
/// Returns the first candidate color that achieves at least `min_ratio` contrast
/// with the background. If no candidate meets the requirement, returns None.
///
/// This is useful for automatically choosing accessible text colors from a palette.
///
/// # Examples
///
/// ```
/// use colorizer::colors::Srgb8;
/// use colorizer::wcag::{choose_accessible_foreground, WCAG_AA_NORMAL};
///
/// let bg = Srgb8::new(30, 30, 30); // Dark background
/// let candidates = vec![
///     Srgb8::new(100, 100, 100), // Low contrast
///     Srgb8::new(200, 200, 200), // Medium contrast
///     Srgb8::new(255, 255, 255), // High contrast
/// ];
///
/// let chosen = choose_accessible_foreground(bg, &candidates, WCAG_AA_NORMAL);
/// assert!(chosen.is_some());
/// ```
pub fn choose_accessible_foreground(bg: Srgb8, candidates: &[Srgb8], min_ratio: f32) -> Option<Srgb8> {
    candidates
        .iter()
        .find(|&&fg| contrast_ratio(bg, fg) >= min_ratio)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.01;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_relative_luminance_extremes() {
        let white = Srgb8::new(255, 255, 255);
        assert!(approx_eq(relative_luminance(white), 1.0));

        let black = Srgb8::new(0, 0, 0);
        assert!(approx_eq(relative_luminance(black), 0.0));
    }

    #[test]
    fn test_relative_luminance_gray() {
        let gray = Srgb8::new(128, 128, 128);
        let lum = relative_luminance(gray);
        assert!(lum > 0.18 && lum < 0.22, "Gray luminance: {lum}");
    }

    #[test]
    fn test_relative_luminance_colors() {
        let red = Srgb8::new(255, 0, 0);
        let green = Srgb8::new(0, 255, 0);
        let blue = Srgb8::new(0, 0, 255);

        let lum_r = relative_luminance(red);
        let lum_g = relative_luminance(green);
        let lum_b = relative_luminance(blue);

        assert!(
            lum_g > lum_r,
            "Green ({lum_g}) should be brighter than red ({lum_r})"
        );
        assert!(
            lum_g > lum_b,
            "Green ({lum_g}) should be brighter than blue ({lum_b})"
        );
        assert!(
            lum_r > lum_b,
            "Red ({lum_r}) should be brighter than blue ({lum_b})"
        );
    }

    #[test]
    fn test_contrast_ratio_extremes() {
        let white = Srgb8::new(255, 255, 255);
        let black = Srgb8::new(0, 0, 0);
        let ratio = contrast_ratio(white, black);
        assert!(approx_eq(ratio, 21.0), "Max contrast ratio: {ratio}");

        let ratio_reversed = contrast_ratio(black, white);
        assert!(approx_eq(ratio, ratio_reversed));
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let color = Srgb8::new(128, 128, 128);
        let ratio = contrast_ratio(color, color);
        assert!(approx_eq(ratio, 1.0), "Same color contrast: {ratio}");
    }

    #[test]
    fn test_contrast_ratio_symmetry() {
        let c1 = Srgb8::new(100, 150, 200);
        let c2 = Srgb8::new(50, 75, 100);
        assert!(approx_eq(contrast_ratio(c1, c2), contrast_ratio(c2, c1)));
    }

    #[test]
    fn test_wcag_aa_normal_threshold() {
        let white = Srgb8::new(255, 255, 255);
        let light_gray = Srgb8::new(180, 180, 180);
        let dark_gray = Srgb8::new(118, 118, 118);
        let black = Srgb8::new(0, 0, 0);
        assert!(meets_aa_normal(contrast_ratio(white, black)));
        assert!(!meets_aa_normal(contrast_ratio(white, light_gray)));

        let ratio = contrast_ratio(white, dark_gray);
        assert!((4.3..=4.7).contains(&ratio), "Ratio: {ratio}");
    }

    #[test]
    fn test_wcag_aa_large_threshold() {
        let white = Srgb8::new(255, 255, 255);
        let medium_gray = Srgb8::new(145, 145, 145);
        let ratio = contrast_ratio(white, medium_gray);

        assert!(meets_aa_large(ratio), "Ratio {ratio} should meet AA large (3:1)");
        assert!(
            !meets_aa_normal(ratio),
            "Ratio {ratio} should not meet AA normal (4.5:1)"
        );
    }

    #[test]
    fn test_wcag_aaa_normal_threshold() {
        let white = Srgb8::new(255, 255, 255);
        let gray = Srgb8::new(89, 89, 89);
        let ratio = contrast_ratio(white, gray);
        assert!((6.8..=7.5).contains(&ratio), "Ratio: {ratio}");
    }

    #[test]
    fn test_wcag_aaa_large_threshold() {
        let white = Srgb8::new(255, 255, 255);
        let gray = Srgb8::new(118, 118, 118);
        let ratio = contrast_ratio(white, gray);

        assert!(meets_aaa_large(ratio));
        assert!(meets_aa_normal(ratio));
        assert!(!meets_aaa_normal(ratio));
    }

    #[test]
    fn test_choose_accessible_foreground_finds_suitable() {
        let bg = Srgb8::new(30, 30, 30);
        let candidates = vec![
            Srgb8::new(50, 50, 50),
            Srgb8::new(100, 100, 100),
            Srgb8::new(200, 200, 200),
            Srgb8::new(255, 255, 255),
        ];

        let chosen = choose_accessible_foreground(bg, &candidates, WCAG_AA_NORMAL);
        assert!(chosen.is_some());

        let chosen_color = chosen.unwrap();
        assert!(contrast_ratio(bg, chosen_color) >= WCAG_AA_NORMAL);
    }

    #[test]
    fn test_choose_accessible_foreground_returns_none() {
        let bg = Srgb8::new(128, 128, 128);
        let candidates = vec![
            Srgb8::new(120, 120, 120),
            Srgb8::new(135, 135, 135),
            Srgb8::new(140, 140, 140),
        ];

        let chosen = choose_accessible_foreground(bg, &candidates, WCAG_AA_NORMAL);
        assert!(chosen.is_none());
    }

    #[test]
    fn test_choose_accessible_foreground_aaa_standard() {
        let bg = Srgb8::new(255, 255, 255);
        let candidates = vec![Srgb8::new(120, 120, 120), Srgb8::new(90, 90, 90), Srgb8::new(0, 0, 0)];
        let chosen = choose_accessible_foreground(bg, &candidates, WCAG_AAA_NORMAL);
        assert!(chosen.is_some());

        let chosen_color = chosen.unwrap();
        assert!(contrast_ratio(bg, chosen_color) >= WCAG_AAA_NORMAL);
    }

    #[test]
    fn test_choose_accessible_foreground_empty_candidates() {
        let bg = Srgb8::new(128, 128, 128);
        let candidates: Vec<Srgb8> = vec![];
        let chosen = choose_accessible_foreground(bg, &candidates, WCAG_AA_NORMAL);
        assert!(chosen.is_none());
    }

    #[test]
    /// Test against known WCAG reference values from https://www.w3.org/TR/WCAG20-TECHS/G17.html
    fn test_wcag_known_reference_values() {
        let white = Srgb8::new(255, 255, 255);
        let gray = Srgb8::from_hex("#777777").unwrap();
        let ratio = contrast_ratio(white, gray);

        assert!((4.4..=4.6).contains(&ratio), "Ratio: {ratio}");
        assert!(meets_aa_large(ratio));
        assert!(!meets_aa_normal(ratio));
    }
}
