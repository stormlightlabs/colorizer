//! Color space conversion functions.
//!
//! Implements bidirectional conversions between various color spaces:
//! - sRGB 8-bit ↔ sRGB float
//! - sRGB ↔ linear RGB (gamma correction)
//! - Linear RGB ↔ XYZ (D65 white point)
//! - XYZ ↔ Lab (perceptually uniform)
//! - Lab ↔ Lch (cylindrical representation)

use crate::colors::*;

const D65_X: f32 = 0.95047;
const D65_Y: f32 = 1.00000;
const D65_Z: f32 = 1.08883;

const LAB_EPSILON: f32 = 216.0 / 24389.0;
const LAB_KAPPA: f32 = 24389.0 / 27.0;

/// sRGB to XYZ transformation matrix (D65 white point)
const RGB_TO_XYZ: [[f32; 3]; 3] = [
    [0.4124564, 0.3575761, 0.1804375],
    [0.2126729, 0.7151522, 0.0721750],
    [0.0193339, 0.1191920, 0.9503041],
];

/// XYZ to sRGB transformation matrix (D65 white point, inverse of above)
const XYZ_TO_RGB: [[f32; 3]; 3] = [
    [3.2404542, -1.5371385, -0.4985314],
    [-0.9692660, 1.8760108, 0.0415560],
    [0.0556434, -0.2040259, 1.0572252],
];

impl From<Srgb8> for Srgb {
    /// Converts 8-bit sRGB to float sRGB by dividing by 255.
    fn from(c: Srgb8) -> Self {
        Srgb::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
    }
}

impl From<Srgb> for Srgb8 {
    /// Converts float sRGB to 8-bit sRGB by multiplying by 255 and rounding.
    fn from(c: Srgb) -> Self {
        Srgb8::new(
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        )
    }
}

/// Converts a single sRGB component to linear RGB using inverse gamma (linearization).
///
/// Uses the standard sRGB transfer function per WCAG guidelines with piecewise 2.4 exponent.
/// - For values d 0.04045: linear segment (value / 12.92)
/// - For values > 0.04045: power function ((value + 0.055) / 1.055)^2.4
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// Converts a single linear RGB component to sRGB using gamma encoding.
///
/// Applies the inverse of the sRGB transfer function:
/// - For values d 0.0031308: linear segment (value * 12.92)
/// - For values > 0.0031308: power function (1.055 * value^(1/2.4) - 0.055)
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

impl From<Srgb> for Rgb {
    /// Converts sRGB to linear RGB by applying inverse gamma correction to each component.
    fn from(c: Srgb) -> Self {
        Rgb::new(srgb_to_linear(c.r), srgb_to_linear(c.g), srgb_to_linear(c.b))
    }
}

impl From<Rgb> for Srgb {
    /// Converts linear RGB to sRGB by applying gamma correction to each component.
    fn from(c: Rgb) -> Self {
        Srgb::new(linear_to_srgb(c.r), linear_to_srgb(c.g), linear_to_srgb(c.b))
    }
}

impl From<Rgb> for Xyz {
    /// Converts linear RGB to XYZ using standard sRGB↔XYZ matrix with D65 white point.
    fn from(c: Rgb) -> Self {
        let x = RGB_TO_XYZ[0][0] * c.r + RGB_TO_XYZ[0][1] * c.g + RGB_TO_XYZ[0][2] * c.b;
        let y = RGB_TO_XYZ[1][0] * c.r + RGB_TO_XYZ[1][1] * c.g + RGB_TO_XYZ[1][2] * c.b;
        let z = RGB_TO_XYZ[2][0] * c.r + RGB_TO_XYZ[2][1] * c.g + RGB_TO_XYZ[2][2] * c.b;
        Xyz::new(x, y, z)
    }
}

impl From<Xyz> for Rgb {
    /// Converts XYZ to linear RGB using standard XYZ↔sRGB matrix with D65 white point.
    fn from(c: Xyz) -> Self {
        let r = XYZ_TO_RGB[0][0] * c.x + XYZ_TO_RGB[0][1] * c.y + XYZ_TO_RGB[0][2] * c.z;
        let g = XYZ_TO_RGB[1][0] * c.x + XYZ_TO_RGB[1][1] * c.y + XYZ_TO_RGB[1][2] * c.z;
        let b = XYZ_TO_RGB[2][0] * c.x + XYZ_TO_RGB[2][1] * c.y + XYZ_TO_RGB[2][2] * c.z;
        Rgb::new(r, g, b)
    }
}

/// Lab conversion helper function f(t).
///
/// Applies the piecewise function used in XYZ↔Lab conversion:
/// - If t > ↔: t^(1/3)
/// - Otherwise: (↔*t + 16)/116
fn lab_f(t: f32) -> f32 {
    if t > LAB_EPSILON { t.cbrt() } else { (LAB_KAPPA * t + 16.0) / 116.0 }
}

/// Inverse Lab conversion helper function f{↔(t).
///
/// Applies the inverse of the Lab conversion function:
/// - If t^3 > ↔: t^3
/// - Otherwise: (116*t - 16)/↔
fn lab_f_inv(t: f32) -> f32 {
    let t3 = t * t * t;
    if t3 > LAB_EPSILON { t3 } else { (116.0 * t - 16.0) / LAB_KAPPA }
}

impl From<Xyz> for Lab {
    /// Converts XYZ to Lab using CIE formulas with D65 white reference.
    fn from(c: Xyz) -> Self {
        let fx = lab_f(c.x / D65_X);
        let fy = lab_f(c.y / D65_Y);
        let fz = lab_f(c.z / D65_Z);

        let l = 116.0 * fy - 16.0;
        let a = 500.0 * (fx - fy);
        let b = 200.0 * (fy - fz);

        Lab::new(l, a, b)
    }
}

impl From<Lab> for Xyz {
    /// Converts Lab to XYZ using inverse CIE transform with D65 white reference.
    fn from(c: Lab) -> Self {
        let fy = (c.l + 16.0) / 116.0;
        let fx = c.a / 500.0 + fy;
        let fz = fy - c.b / 200.0;

        let x = D65_X * lab_f_inv(fx);
        let y = D65_Y * lab_f_inv(fy);
        let z = D65_Z * lab_f_inv(fz);

        Xyz::new(x, y, z)
    }
}

impl From<Lab> for Lch {
    /// Converts Lab to Lch using cylindrical coordinates.
    ///
    /// - L is copied directly
    /// - C (chroma) = sqrt(a↔ + b↔)
    /// - h (hue) = atan2(b, a) converted to degrees [0, 360)
    fn from(c: Lab) -> Self {
        let l = c.l;
        let chroma = (c.a * c.a + c.b * c.b).sqrt();
        let hue = c.b.atan2(c.a).to_degrees();

        Lch::new(l, chroma, hue)
    }
}

impl From<Lch> for Lab {
    /// Converts Lch to Lab using inverse cylindrical transform.
    ///
    /// - L is copied directly
    /// - a = C * cos(h)
    /// - b = C * sin(h)
    fn from(c: Lch) -> Self {
        let l = c.l;
        let h_rad = c.h.to_radians();
        let a = c.c * h_rad.cos();
        let b = c.c * h_rad.sin();

        Lab::new(l, a, b)
    }
}

impl From<Srgb8> for Rgb {
    /// Direct conversion from 8-bit sRGB to linear RGB (via float sRGB).
    fn from(c: Srgb8) -> Self {
        Srgb::from(c).into()
    }
}

impl From<Rgb> for Srgb8 {
    /// Direct conversion from linear RGB to 8-bit sRGB (via float sRGB).
    fn from(c: Rgb) -> Self {
        Srgb::from(c).into()
    }
}

impl From<Srgb8> for Lab {
    /// Direct conversion from 8-bit sRGB to Lab (via Srgb↔Rgb↔Xyz↔Lab).
    fn from(c: Srgb8) -> Self {
        let srgb = Srgb::from(c);
        let rgb = Rgb::from(srgb);
        let xyz = Xyz::from(rgb);
        Lab::from(xyz)
    }
}

impl From<Lab> for Srgb8 {
    /// Direct conversion from Lab to 8-bit sRGB (via Xyz↔Rgb↔Srgb↔Srgb8).
    fn from(c: Lab) -> Self {
        let xyz = Xyz::from(c);
        let rgb = Rgb::from(xyz);
        let srgb = Srgb::from(rgb);
        Srgb8::from(srgb)
    }
}

impl From<Srgb8> for Lch {
    /// Direct conversion from 8-bit sRGB to Lch (via Lab).
    fn from(c: Srgb8) -> Self {
        Lch::from(Lab::from(c))
    }
}

impl From<Lch> for Srgb8 {
    /// Direct conversion from Lch to 8-bit sRGB (via Lab).
    fn from(c: Lch) -> Self {
        Srgb8::from(Lab::from(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_srgb8_to_srgb_conversion() {
        let c8 = Srgb8::new(255, 128, 0);
        let cf = Srgb::from(c8);
        assert!(approx_eq(cf.r, 1.0));
        assert!(approx_eq(cf.g, 128.0 / 255.0));
        assert!(approx_eq(cf.b, 0.0));

        let c8_back = Srgb8::from(cf);
        assert_eq!(c8_back, c8);
    }

    #[test]
    fn test_srgb_to_linear_rgb_conversion() {
        let black = Srgb::new(0.0, 0.0, 0.0);
        let linear = Rgb::from(black);
        assert!(approx_eq(linear.r, 0.0));
        assert!(approx_eq(linear.g, 0.0));
        assert!(approx_eq(linear.b, 0.0));

        let white = Srgb::new(1.0, 1.0, 1.0);
        let linear = Rgb::from(white);
        assert!(approx_eq(linear.r, 1.0));
        assert!(approx_eq(linear.g, 1.0));
        assert!(approx_eq(linear.b, 1.0));

        let color = Srgb::new(0.5, 0.75, 0.25);
        let linear = Rgb::from(color);
        let back = Srgb::from(linear);
        assert!(approx_eq(back.r, color.r));
        assert!(approx_eq(back.g, color.g));
        assert!(approx_eq(back.b, color.b));
    }

    #[test]
    fn test_rgb_to_xyz_conversion() {
        let white = Rgb::new(1.0, 1.0, 1.0);
        let xyz = Xyz::from(white);
        assert!(approx_eq(xyz.x, D65_X));
        assert!(approx_eq(xyz.y, D65_Y));
        assert!(approx_eq(xyz.z, D65_Z));

        let color = Rgb::new(0.5, 0.3, 0.7);
        let xyz = Xyz::from(color);
        let back = Rgb::from(xyz);
        assert!(approx_eq(back.r, color.r));
        assert!(approx_eq(back.g, color.g));
        assert!(approx_eq(back.b, color.b));
    }

    #[test]
    fn test_xyz_to_lab_conversion() {
        let white_xyz = Xyz::new(D65_X, D65_Y, D65_Z);
        let lab = Lab::from(white_xyz);
        assert!(approx_eq(lab.l, 100.0));
        assert!(approx_eq(lab.a, 0.0));
        assert!(approx_eq(lab.b, 0.0));

        let xyz = Xyz::new(0.5, 0.6, 0.4);
        let lab = Lab::from(xyz);
        let back = Xyz::from(lab);
        assert!(approx_eq(back.x, xyz.x));
        assert!(approx_eq(back.y, xyz.y));
        assert!(approx_eq(back.z, xyz.z));
    }

    #[test]
    fn test_lab_to_lch_conversion() {
        let gray = Lab::new(50.0, 0.0, 0.0);
        let lch = Lch::from(gray);
        assert!(approx_eq(lch.l, 50.0));
        assert!(approx_eq(lch.c, 0.0));

        let lab = Lab::new(60.0, 30.0, -20.0);
        let lch = Lch::from(lab);
        let back = Lab::from(lch);
        assert!(approx_eq(back.l, lab.l));
        assert!(approx_eq(back.a, lab.a));
        assert!(approx_eq(back.b, lab.b));
    }

    #[test]
    fn test_full_round_trip_srgb8_to_lab() {
        let colors = vec![
            Srgb8::new(255, 0, 0),
            Srgb8::new(0, 255, 0),
            Srgb8::new(0, 0, 255),
            Srgb8::new(255, 255, 0),
            Srgb8::new(128, 128, 128),
        ];

        for c8 in colors {
            let lab = Lab::from(c8);
            let back = Srgb8::from(lab);
            let tolerance = 2;
            assert!(
                (back.r as i32 - c8.r as i32).abs() <= tolerance,
                "Red mismatch: {} != {}",
                back.r,
                c8.r
            );
            assert!(
                (back.g as i32 - c8.g as i32).abs() <= tolerance,
                "Green mismatch: {} != {}",
                back.g,
                c8.g
            );
            assert!(
                (back.b as i32 - c8.b as i32).abs() <= tolerance,
                "Blue mismatch: {} != {}",
                back.b,
                c8.b
            );
        }
    }

    #[test]
    fn test_full_round_trip_srgb8_to_lch() {
        let c8 = Srgb8::new(200, 100, 50);
        let lch = Lch::from(c8);
        let back = Srgb8::from(lch);
        let tolerance = 2;
        assert!(
            (back.r as i32 - c8.r as i32).abs() <= tolerance,
            "Red mismatch: {} != {}",
            back.r,
            c8.r
        );
        assert!(
            (back.g as i32 - c8.g as i32).abs() <= tolerance,
            "Green mismatch: {} != {}",
            back.g,
            c8.g
        );
        assert!(
            (back.b as i32 - c8.b as i32).abs() <= tolerance,
            "Blue mismatch: {} != {}",
            back.b,
            c8.b
        );
    }

    #[test]
    fn test_known_reference_colors() {
        let red = Srgb8::new(255, 0, 0);
        let lab = Lab::from(red);
        assert!(lab.l > 50.0 && lab.l < 55.0, "L should be around 53");
        assert!(lab.a > 75.0 && lab.a < 85.0, "a should be around 80");
        assert!(lab.b > 65.0 && lab.b < 75.0, "b should be around 67");

        let green = Srgb8::new(0, 255, 0);
        let lab = Lab::from(green);
        assert!(lab.l > 85.0 && lab.l < 90.0, "L should be around 87, got {}", lab.l);
        assert!(lab.a < -75.0 && lab.a > -87.0, "a should be around -86, got {}", lab.a);
        assert!(lab.b > 80.0 && lab.b < 90.0, "b should be around 85, got {}", lab.b);

        let blue = Srgb8::new(0, 0, 255);
        let lab = Lab::from(blue);
        assert!(lab.l > 30.0 && lab.l < 35.0, "L should be around 32");
        assert!(lab.a > 75.0 && lab.a < 85.0, "a should be around 79");
        assert!(lab.b < -105.0 && lab.b > -115.0, "b should be around -108");
    }
}
