pub mod colors;
mod conversions;
mod diffs;
mod interpolation;
mod palette;
mod random;
mod shades;
mod tinted_theming;
mod vimrc;
pub mod wcag;

pub mod harmonies;
pub use harmonies::{HarmonyKind, harmonies, normalize_saturation, set_lightness, shift_lightness};
