pub mod colors;
mod conversions;
mod diffs;
mod interpolation;
mod palette;
mod random;
mod tinted_theming;
mod vimrc;
pub mod wcag;

pub mod harmonies;
pub use harmonies::{HarmonyKind, harmonies, normalize_saturation, set_lightness, shift_lightness};

pub mod shades;
pub use shades::{darken_hsl, desaturate_hsl, lighten_hsl, mix_rgb, shade, tint, tone};
