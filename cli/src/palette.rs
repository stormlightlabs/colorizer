//! Palette generation helpers and visualization utilities.

use crate::GoldenPalette;
use crate::colors::{Hsl, Rgb, Srgb8};
use crate::diffs::ensure_min_distance;
use crate::harmonies::{HarmonyKind, harmonies};
use crate::shades::{darken_hsl, lighten_hsl};
use crate::wcag::contrast_ratio;

use image::{Rgb as ImgRgb, RgbImage};
use rusttype::{Font, Scale, point};
use std::cmp::max;
use std::ops::Range;

const VARIATION_STEP: f32 = 0.08;
const FONT_WIDTH: u32 = 5;
const FONT_HEIGHT: u32 = 7;
const TRUETYPE_FONT_SIZE: f32 = 24.0;
const MIN_HEIGHT_WITH_TRUETYPE: u32 = 40;

/// Label styles supported during palette-to-image rendering.
#[derive(Debug, Clone, Copy)]
pub enum PaletteLabelStyle<'a> {
    Hex,
    Index,
    None,
    Custom(&'a [String]),
}

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

/// Attempts to load a TrueType font from the system.
///
/// TODO: Allow users to pass in a custom font family via CLI flag (e.g., --font "FontName").
fn load_system_font() -> Option<Font<'static>> {
    if let Some((data, _)) = font_loader::system_fonts::get(
        &font_loader::system_fonts::FontPropertyBuilder::new()
            .family("0xProto Nerd Font")
            .build(),
    ) {
        if let Some(font) = Font::try_from_vec(data) {
            return Some(font);
        }
    }

    for family in &[
        "0xProto Nerd Font Mono",
        "Monaco",
        "Menlo",
        "Consolas",
        "DejaVu Sans Mono",
    ] {
        if let Some((data, _)) = font_loader::system_fonts::get(
            &font_loader::system_fonts::FontPropertyBuilder::new()
                .family(family)
                .build(),
        ) {
            if let Some(font) = Font::try_from_vec(data) {
                return Some(font);
            }
        }
    }

    None
}

/// Renders the palette into an RGB image with vertical bars and optional labels.
pub fn palette_to_image<'a>(colors: &[Srgb8], labels: PaletteLabelStyle<'a>, size: (u32, u32)) -> RgbImage {
    let system_font = load_system_font();
    let min_height = if system_font.is_some() { MIN_HEIGHT_WITH_TRUETYPE } else { FONT_HEIGHT + 8 };

    let width = max(size.0, colors.len() as u32).max(1);
    let height = max(size.1, min_height);
    let mut image = RgbImage::from_pixel(width, height, ImgRgb([0, 0, 0]));

    if colors.is_empty() {
        return image;
    }

    let label_strings = build_labels(colors, labels);
    let segments = colors.len() as u32;
    let mut start_x = 0;
    let base_width = max(width / segments, 1);

    for (index, &color) in colors.iter().enumerate() {
        let mut end_x = start_x + base_width;
        if index == colors.len() - 1 {
            end_x = width;
        }
        fill_bar(&mut image, start_x, end_x, color);

        if let Some(text) = label_strings.get(index) {
            let text_color = pick_label_color(color);
            if let Some(ref font) = system_font {
                draw_label_truetype(&mut image, text, start_x, end_x, text_color, font);
            } else {
                draw_label_bitmap(&mut image, text, start_x, end_x, text_color);
            }
        }

        start_x = end_x;
    }

    image
}

fn build_labels<'a>(colors: &[Srgb8], labels: PaletteLabelStyle<'a>) -> Vec<String> {
    match labels {
        PaletteLabelStyle::None => Vec::new(),
        PaletteLabelStyle::Hex => colors.iter().map(|c| c.to_hex().to_uppercase()).collect(),
        PaletteLabelStyle::Index => colors.iter().enumerate().map(|(i, _)| format!("{i:02}")).collect(),
        PaletteLabelStyle::Custom(values) => colors
            .iter()
            .enumerate()
            .map(|(i, _)| values.get(i).cloned().unwrap_or_default())
            .collect(),
    }
}

fn fill_bar(image: &mut RgbImage, start_x: u32, end_x: u32, color: Srgb8) {
    for y in 0..image.height() {
        for x in start_x..end_x {
            image.put_pixel(x, y, ImgRgb([color.r, color.g, color.b]));
        }
    }
}

fn pick_label_color(bg: Srgb8) -> Srgb8 {
    let white = Srgb8::new(255, 255, 255);
    let black = Srgb8::new(0, 0, 0);
    if contrast_ratio(bg, white) >= contrast_ratio(bg, black) { white } else { black }
}

fn draw_label_truetype(image: &mut RgbImage, text: &str, start_x: u32, end_x: u32, color: Srgb8, font: &Font) {
    if text.is_empty() {
        return;
    }

    let sanitized = text.trim();
    if sanitized.is_empty() {
        return;
    }

    let scale = Scale::uniform(TRUETYPE_FONT_SIZE);
    let v_metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font.layout(sanitized, scale, point(0.0, 0.0)).collect();
    let text_width = glyphs
        .iter()
        .filter_map(|g| g.pixel_bounding_box().map(|b| b.max.x))
        .max()
        .unwrap_or(0) as u32;

    let available = end_x.saturating_sub(start_x);
    let x = start_x + available.saturating_sub(text_width) / 2;
    let y = image
        .height()
        .saturating_sub((v_metrics.ascent - v_metrics.descent) as u32 + 6);

    for glyph in font.layout(sanitized, scale, point(x as f32, y as f32 + v_metrics.ascent)) {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let px = bounding_box.min.x + gx as i32;
                let py = bounding_box.min.y + gy as i32;

                if px >= 0 && py >= 0 {
                    let px = px as u32;
                    let py = py as u32;

                    if px < image.width() && py < image.height() && v > 0.1 {
                        let bg = image.get_pixel(px, py);

                        let r = ((1.0 - v) * bg[0] as f32 + v * color.r as f32) as u8;
                        let g = ((1.0 - v) * bg[1] as f32 + v * color.g as f32) as u8;
                        let b = ((1.0 - v) * bg[2] as f32 + v * color.b as f32) as u8;

                        image.put_pixel(px, py, ImgRgb([r, g, b]));
                    }
                }
            });
        }
    }
}

fn draw_label_bitmap(image: &mut RgbImage, text: &str, start_x: u32, end_x: u32, color: Srgb8) {
    if text.is_empty() {
        return;
    }

    let sanitized = text.trim();
    if sanitized.is_empty() {
        return;
    }

    let text_width = sanitized.len() as u32 * (FONT_WIDTH + 1) - 1;
    let available = end_x.saturating_sub(start_x);
    let x = start_x + available.saturating_sub(text_width) / 2;
    let y = image.height().saturating_sub(FONT_HEIGHT + 3);

    draw_text(image, sanitized, x, y, ImgRgb([color.r, color.g, color.b]));
}

fn draw_text(image: &mut RgbImage, text: &str, mut cursor_x: u32, cursor_y: u32, color: ImgRgb<u8>) {
    for ch in text.chars() {
        if let Some(rows) = glyph_for(ch.to_ascii_uppercase()) {
            for (row_idx, row) in rows.iter().enumerate() {
                for col in 0..FONT_WIDTH {
                    if row & (1 << (FONT_WIDTH - 1 - col)) != 0 {
                        let x = cursor_x + col;
                        let y = cursor_y + row_idx as u32;
                        if x < image.width() && y < image.height() {
                            image.put_pixel(x, y, color);
                        }
                    }
                }
            }
            cursor_x += FONT_WIDTH + 1;
        } else {
            cursor_x += FONT_WIDTH;
        }
    }
}

fn glyph_for(ch: char) -> Option<&'static [u8; FONT_HEIGHT as usize]> {
    match ch {
        '0' => Some(&[0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
        '1' => Some(&[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        '2' => Some(&[0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
        '3' => Some(&[0b11110, 0b00001, 0b00001, 0b00110, 0b00001, 0b00001, 0b11110]),
        '4' => Some(&[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
        '5' => Some(&[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
        '6' => Some(&[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
        '7' => Some(&[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
        '8' => Some(&[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        '9' => Some(&[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
        'A' => Some(&[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'B' => Some(&[0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110]),
        'C' => Some(&[0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110]),
        'D' => Some(&[0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100]),
        'E' => Some(&[0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b11111]),
        'F' => Some(&[0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000]),
        '#' => Some(&[0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010]),
        ' ' => Some(&[0; FONT_HEIGHT as usize]),
        _ => None,
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

    #[test]
    fn palette_image_dimensions_match_request() {
        let colors = vec![Srgb8::new(255, 0, 0), Srgb8::new(0, 255, 0)];
        let image = palette_to_image(&colors, PaletteLabelStyle::Index, (200, 80));
        assert_eq!(image.width(), 200);
        assert_eq!(image.height(), 80);
    }
}
