//! Base16/Base24 scheme parsing helpers compatible with tinted-theming.

use crate::colors::{Rgb, Srgb8};

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const BASE16_KEYS: [&str; 16] = [
    "base00", "base01", "base02", "base03", "base04", "base05", "base06", "base07", "base08", "base09", "base0A",
    "base0B", "base0C", "base0D", "base0E", "base0F",
];

const BASE24_KEYS: [&str; 24] = [
    "base00", "base01", "base02", "base03", "base04", "base05", "base06", "base07", "base08", "base09", "base0A",
    "base0B", "base0C", "base0D", "base0E", "base0F", "base10", "base11", "base12", "base13", "base14", "base15",
    "base16", "base17",
];

/// Metadata shared across scheme formats.
#[derive(Debug, Clone)]
pub struct SchemeMetadata {
    pub system: String,
    pub name: String,
    pub author: Option<String>,
    pub variant: Option<String>,
}

/// Base16 scheme definition (16 canonical colors).
#[derive(Debug, Clone)]
pub struct Base16Scheme {
    pub metadata: SchemeMetadata,
    colors: [Srgb8; 16],
}

impl Base16Scheme {
    pub fn new(metadata: SchemeMetadata, colors: [Srgb8; 16]) -> Self {
        Self { metadata, colors }
    }

    pub fn colors(&self) -> &[Srgb8] {
        &self.colors
    }

    pub fn as_rgb(&self) -> Vec<Rgb> {
        self.colors.iter().copied().map(Rgb::from).collect()
    }
}

/// Base24 scheme definition (Base16 + 8 extended slots).
#[derive(Debug, Clone)]
pub struct Base24Scheme {
    pub metadata: SchemeMetadata,
    colors: [Srgb8; 24],
}

impl Base24Scheme {
    pub fn new(metadata: SchemeMetadata, colors: [Srgb8; 24]) -> Self {
        Self { metadata, colors }
    }

    pub fn colors(&self) -> &[Srgb8] {
        &self.colors
    }

    pub fn as_rgb(&self) -> Vec<Rgb> {
        self.colors.iter().copied().map(Rgb::from).collect()
    }
}

/// Errors that may occur while loading tinted-theming schemes.
#[derive(Debug)]
pub enum SchemeError {
    Io { path: PathBuf, source: std::io::Error },
    Parse { path: PathBuf, source: serde_yml::Error },
    MissingField(&'static str),
    MissingColor(String),
    InvalidHex { key: String, value: String },
    UnsupportedSystem(String),
    EmptyDirectory(PathBuf),
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeError::Io { path, source } => write!(f, "failed to read {}: {}", path.display(), source),
            SchemeError::Parse { path, source } => write!(f, "failed to parse {}: {}", path.display(), source),
            SchemeError::MissingField(field) => write!(f, "scheme is missing required field '{field}'"),
            SchemeError::MissingColor(key) => write!(f, "scheme palette missing '{key}'"),
            SchemeError::InvalidHex { key, value } => {
                write!(f, "palette entry '{key}' is not a valid hex color: {value}")
            }
            SchemeError::UnsupportedSystem(system) => write!(f, "unsupported scheme system '{system}'"),
            SchemeError::EmptyDirectory(path) => write!(f, "no YAML schemes found in {}", path.display()),
        }
    }
}

impl std::error::Error for SchemeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SchemeError::Io { source, .. } => Some(source),
            SchemeError::Parse { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawScheme {
    system: Option<String>,
    name: Option<String>,
    author: Option<String>,
    variant: Option<String>,
    palette: HashMap<String, String>,
}

/// Loads Base16 schemes from a file or directory path.
pub fn load_base16_schemes(path: impl AsRef<Path>) -> Result<Vec<Base16Scheme>, SchemeError> {
    load_schemes(path.as_ref(), "base16", parse_base16)
}

/// Loads Base24 schemes from a file or directory path.
pub fn load_base24_schemes(path: impl AsRef<Path>) -> Result<Vec<Base24Scheme>, SchemeError> {
    load_schemes(path.as_ref(), "base24", parse_base24)
}

fn load_schemes<T, F>(path: &Path, expected: &str, parser: F) -> Result<Vec<T>, SchemeError>
where
    F: Fn(RawScheme, PathBuf) -> Result<T, SchemeError>,
{
    if path.is_dir() {
        let mut schemes = Vec::new();
        for entry in fs::read_dir(path).map_err(|source| SchemeError::Io { path: path.to_path_buf(), source })? {
            let entry = entry.map_err(|source| SchemeError::Io { path: path.to_path_buf(), source })?;
            let file_path = entry.path();
            if is_yaml(&file_path) {
                let raw = parse_file(&file_path)?;
                schemes.push(parser(raw, file_path)?);
            }
        }
        if schemes.is_empty() {
            return Err(SchemeError::EmptyDirectory(path.to_path_buf()));
        }
        Ok(schemes)
    } else {
        let raw = parse_file(path)?;
        if let Some(system) = raw.system.as_deref() {
            if system != expected {
                return Err(SchemeError::UnsupportedSystem(system.to_string()));
            }
        }
        parser(raw, path.to_path_buf()).map(|scheme| vec![scheme])
    }
}

fn parse_file(path: &Path) -> Result<RawScheme, SchemeError> {
    let contents = fs::read_to_string(path).map_err(|source| SchemeError::Io { path: path.to_path_buf(), source })?;
    serde_yml::from_str(&contents).map_err(|source| SchemeError::Parse { path: path.to_path_buf(), source })
}

fn parse_base16(raw: RawScheme, _: PathBuf) -> Result<Base16Scheme, SchemeError> {
    let metadata = metadata(&raw, "base16")?;
    let colors = build_palette(&raw.palette, &BASE16_KEYS)?;
    let mut array = [Srgb8::new(0, 0, 0); 16];
    array.copy_from_slice(&colors);
    Ok(Base16Scheme { metadata, colors: array })
}

fn parse_base24(raw: RawScheme, _: PathBuf) -> Result<Base24Scheme, SchemeError> {
    let metadata = metadata(&raw, "base24")?;
    let colors = build_palette(&raw.palette, &BASE24_KEYS)?;
    let mut array = [Srgb8::new(0, 0, 0); 24];
    array.copy_from_slice(&colors);
    Ok(Base24Scheme { metadata, colors: array })
}

fn metadata(raw: &RawScheme, expected_system: &str) -> Result<SchemeMetadata, SchemeError> {
    let name = raw.name.clone().ok_or(SchemeError::MissingField("name"))?;
    let system = raw.system.clone().unwrap_or_else(|| expected_system.to_string());
    if system != expected_system {
        return Err(SchemeError::UnsupportedSystem(system));
    }
    Ok(SchemeMetadata { system, name, author: raw.author.clone(), variant: raw.variant.clone() })
}

fn build_palette(palette: &HashMap<String, String>, keys: &[&str]) -> Result<Vec<Srgb8>, SchemeError> {
    let mut colors = Vec::with_capacity(keys.len());
    for key in keys {
        let raw = palette
            .get(*key)
            .ok_or_else(|| SchemeError::MissingColor((*key).to_string()))?;
        let trimmed = raw.trim();
        let color = Srgb8::from_hex(trimmed)
            .ok_or_else(|| SchemeError::InvalidHex { key: key.to_string(), value: trimmed.to_string() })?;
        colors.push(color);
    }
    Ok(colors)
}

fn is_yaml(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some(ext) if ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_base16_example() {
        let raw: RawScheme = serde_yml::from_str(include_str!("../../examples/base16/oxocarbon-dark.yml")).unwrap();
        let scheme = parse_base16(raw, PathBuf::new()).unwrap();
        assert_eq!(scheme.metadata.name, "Oxocarbon Dark");
        assert_eq!(scheme.colors()[0], Srgb8::from_hex("#161616").unwrap());
        assert_eq!(scheme.colors().len(), 16);
    }

    #[test]
    fn parse_base24_example() {
        let raw: RawScheme =
            serde_yml::from_str(include_str!("../../examples/base24/catppuccin-macchiato.yml")).unwrap();
        let scheme = parse_base24(raw, PathBuf::new()).unwrap();
        assert_eq!(scheme.metadata.name, "Catppuccin Macchiato");
        assert_eq!(scheme.colors().len(), 24);
        assert_eq!(scheme.colors()[23], Srgb8::from_hex("#f5bde6").unwrap());
    }
}
