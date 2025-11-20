pub mod colors;
mod conversions;
pub mod diffs;
mod palette;
mod random;
mod tinted_theming;
mod vimrc;
pub mod wcag;

pub mod harmonies;
pub use harmonies::{HarmonyKind, harmonies, normalize_saturation, set_lightness, shift_lightness};

pub mod shades;
pub use shades::{darken_hsl, desaturate_hsl, lighten_hsl, mix_rgb, shade, tint, tone};

pub mod interpolation;
pub use interpolation::{gradient_lab, gradient_lch, lerp_lab, lerp_lch, lerp_rgb};
