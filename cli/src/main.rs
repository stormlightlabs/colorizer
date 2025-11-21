use clap::{Parser, Subcommand};
use colorizer::{
    HarmonyKind,
    base16_builder::{self, Base16Config, Base24Config, Variant},
    colors::Srgb8,
    palette::{PaletteLabelStyle, golden_ratio_palette, palette_from_base, palette_to_image},
    random::{self, PaletteConstraints, PoissonConfig},
    syntax,
    tinted_theming::{self, SchemeMetadata},
};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::ops::Range;

const NEUTRAL_SATURATION_TOLERANCE: f32 = 0.02;

#[derive(Parser)]
#[command(name = "colorizer")]
#[command(about = "Generate color schemes and palettes with semantic Base16/Base24 support", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate, validate, and visualize Base16/Base24 color schemes
    #[command(visible_alias = "s")]
    Scheme {
        #[command(subcommand)]
        action: SchemeAction,
    },
    /// Generate color palettes (use 'scheme' for Base16/Base24 generation)
    Palette {
        #[command(subcommand)]
        action: PaletteAction,
    },
    /// Generate palette visualization images
    Image {
        /// Color values as hex codes (comma-separated, e.g., "#ff0000,#00ff00,#0000ff")
        #[arg(long, conflicts_with = "scheme_yaml")]
        colors: Option<String>,
        /// Base16/Base24 scheme YAML file
        #[arg(long, conflicts_with = "colors")]
        scheme_yaml: Option<String>,
        /// Output image file path
        #[arg(short, long, default_value = "palette.png")]
        out: String,
        /// Image width in pixels
        #[arg(long)]
        width: Option<u32>,
        /// Image height in pixels
        #[arg(long)]
        height: Option<u32>,
        /// Label style for color bars
        #[arg(long, value_parser = ["hex", "base16", "index", "none"], default_value = "index")]
        label: String,
        /// Show palette in terminal after generating image
        #[arg(long)]
        viz: bool,
    },
    /// Generate Vim colorscheme files
    VimScheme {
        /// Base16/Base24 scheme YAML file
        #[arg(long)]
        scheme_yaml: String,
        /// Name for the colorscheme
        #[arg(long)]
        name: String,
        /// Output directory for colors/<name>.vim
        #[arg(long)]
        output_colors: String,
        /// Optional: update .vimrc with the new colorscheme
        #[arg(long)]
        update_vimrc: Option<String>,
    },
    /// Show syntax-highlighted code samples in terminal
    Demo {
        #[command(subcommand)]
        demo_type: DemoType,
    },
}

#[derive(Subcommand)]
enum SchemeAction {
    /// Generate a Base16 or Base24 color scheme from a single accent color
    #[command(visible_alias = "gen")]
    Generate {
        /// Scheme format (base16 or base24)
        #[arg(value_parser = ["base16", "base24"])]
        format: String,
        /// Scheme name
        #[arg(long)]
        name: String,
        /// Author name (optional)
        #[arg(long)]
        author: Option<String>,
        /// Theme variant: dark or light
        #[arg(long, value_parser = ["dark", "light"], default_value = "dark")]
        variant: String,
        /// Accent color as hex (e.g., "#ff5500")
        #[arg(long)]
        accent: String,
        /// Color harmony for accent generation
        #[arg(long, value_parser = ["complementary", "split-complementary", "analogous", "triadic", "tetradic", "square"], default_value = "triadic")]
        harmony: String,
        /// Neutral darkness (0 = classic bright neutrals, 1 = moody/dark neutrals)
        #[arg(long, default_value_t = base16_builder::DEFAULT_NEUTRAL_DEPTH)]
        neutral_depth: f32,
        /// Output YAML file path (defaults to <name>.yml)
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Visualize a scheme (terminal preview, image, or syntax demo)
    #[command(visible_alias = "preview")]
    Show {
        /// Base16/Base24 scheme YAML file
        scheme: String,
        /// Output format
        #[arg(long, value_parser = ["terminal", "image"], default_value = "terminal")]
        format: String,
        /// Output file path (required for 'image' format)
        #[arg(long, short)]
        output: Option<String>,
        /// Image width in pixels (for image format)
        #[arg(long, default_value = "960")]
        width: u32,
        /// Image height in pixels (for image format)
        #[arg(long, default_value = "320")]
        height: u32,
        /// Show syntax-highlighted code demo
        #[arg(long)]
        demo: Option<String>,
        /// Code file for syntax demo
        #[arg(long, requires = "demo")]
        file: Option<String>,
    },
    /// Validate a scheme (contrast, neutrals, color roles)
    Validate {
        /// Base16/Base24 scheme YAML file
        scheme: String,
    },
}

#[derive(Subcommand)]
enum PaletteAction {
    /// Generate palette from a base color using color harmonies
    FromBase {
        /// Base color as hex code (e.g., "#ff5500")
        #[arg(long)]
        base: String,
        /// Harmony type to generate
        #[arg(long, value_parser = ["complementary", "split-complementary", "analogous", "triadic", "tetradic", "square"])]
        harmony: String,
        /// Number of colors to generate
        #[arg(long, default_value = "5")]
        count: usize,
        /// Minimum contrast ratio against background (optional)
        #[arg(long)]
        min_contrast: Option<f32>,
        /// Background color for contrast checking (optional)
        #[arg(long)]
        background: Option<String>,
        /// Output format
        #[arg(long, value_parser = ["json", "yaml", "hex"], default_value = "hex")]
        format: String,
        /// Generate and save palette image to this path
        #[arg(long)]
        save_image: Option<String>,
        /// Image width in pixels
        #[arg(long, default_value = "960")]
        image_width: u32,
        /// Image height in pixels
        #[arg(long, default_value = "320")]
        image_height: u32,
        /// Image label style
        #[arg(long, value_parser = ["hex", "base16", "index", "none"], default_value = "index")]
        image_label: String,
    },
    /// Generate random color palettes
    Random {
        /// Number of colors to generate
        #[arg(long, default_value = "5")]
        count: usize,
        /// Generation method
        #[arg(long, value_parser = ["uniform", "golden", "poisson"], default_value = "golden")]
        method: String,
        /// Minimum color difference (Delta E)
        #[arg(long)]
        min_delta_e: Option<f32>,
        /// Theme preference
        #[arg(long, value_parser = ["light", "dark"])]
        theme: Option<String>,
        /// Output format
        #[arg(long, value_parser = ["json", "yaml", "hex"], default_value = "hex")]
        format: String,
        /// Generate and save palette image to this path
        #[arg(long)]
        save_image: Option<String>,
        /// Image width in pixels
        #[arg(long, default_value = "960")]
        image_width: u32,
        /// Image height in pixels
        #[arg(long, default_value = "320")]
        image_height: u32,
        /// Image label style
        #[arg(long, value_parser = ["hex", "base16", "index", "none"], default_value = "index")]
        image_label: String,
    },
    /// Export Base16 palette from scheme
    Base16 {
        /// Base16 scheme YAML file
        #[arg(long)]
        scheme_yaml: String,
        /// Output format
        #[arg(long, value_parser = ["json", "yaml", "hex"], default_value = "hex")]
        format: String,
    },
    /// Export Base24 palette from scheme
    Base24 {
        /// Base24 scheme YAML file
        #[arg(long)]
        scheme_yaml: String,
        /// Output format
        #[arg(long, value_parser = ["json", "yaml", "hex"], default_value = "hex")]
        format: String,
    },
}

#[derive(Subcommand)]
enum DemoType {
    /// Show palette as colored terminal output
    Palette {
        /// Color values as hex codes (comma-separated)
        #[arg(long, conflicts_with = "scheme_yaml")]
        colors: Option<String>,
        /// Base16/Base24 scheme YAML file
        #[arg(long, conflicts_with = "colors")]
        scheme_yaml: Option<String>,
    },
    /// Show syntax-highlighted code sample
    Code {
        /// Programming language
        #[arg(long, default_value = "rust")]
        language: String,
        /// Base16/Base24 scheme YAML file
        #[arg(long, conflicts_with = "base")]
        theme_yaml: Option<String>,
        /// Base color for theme generation
        #[arg(long, conflicts_with = "theme_yaml")]
        base: Option<String>,
        /// Harmony type (when using --base)
        #[arg(long, requires = "base")]
        harmony: Option<String>,
        /// Source code file to highlight (reads from stdin if not provided)
        #[arg(long)]
        file: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scheme { action } => handle_scheme(action),
        Commands::Palette { action } => handle_palette(action),
        Commands::Image { colors, scheme_yaml, out, width, height, label, viz } => {
            handle_image(colors, scheme_yaml, out, width, height, label, viz)
        }
        Commands::VimScheme { scheme_yaml, name, output_colors, update_vimrc } => {
            handle_vim_scheme(scheme_yaml, name, output_colors, update_vimrc)
        }
        Commands::Demo { demo_type } => handle_demo(demo_type),
    }
}

fn handle_scheme(action: SchemeAction) {
    match action {
        SchemeAction::Generate { format, name, author, variant, accent, harmony, neutral_depth, output } => {
            let accent_color = match parse_hex_color(&accent) {
                Ok(color) => color,
                Err(err) => {
                    eprintln!("{err}");
                    return;
                }
            };

            let variant = match variant.as_str() {
                "dark" => Variant::Dark,
                "light" => Variant::Light,
                _ => {
                    eprintln!("Invalid variant: {variant}");
                    return;
                }
            };

            let harmony_kind = match parse_harmony_kind(&harmony) {
                Some(kind) => kind,
                None => {
                    eprintln!("Unsupported harmony: {harmony}");
                    return;
                }
            };

            let output_path = output.unwrap_or_else(|| {
                let sanitized = name.to_lowercase().replace(' ', "-");
                format!("{sanitized}.yml")
            });
            let neutral_depth = neutral_depth.clamp(0.0, 1.0);

            match format.as_str() {
                "base16" => {
                    let config =
                        Base16Config { name, author, variant, accent_color, harmony: harmony_kind, neutral_depth };
                    let scheme = base16_builder::generate_base16_scheme(config);

                    if let Err(err) = tinted_theming::write_base16_scheme(&scheme, &output_path) {
                        eprintln!("Failed to write scheme: {err}");
                        return;
                    }

                    println!("Generated Base16 scheme: {}", scheme.metadata.name);
                    println!("  Variant: {}", scheme.metadata.variant.as_deref().unwrap_or("unknown"));
                    println!("  Output: {output_path}");
                    println!("\nPreview:");
                    syntax::display_palette_in_terminal(scheme.colors(), Some(&base16_labels(16)));
                }
                "base24" => {
                    let config =
                        Base24Config { name, author, variant, accent_color, harmony: harmony_kind, neutral_depth };
                    let scheme = base16_builder::generate_base24_scheme(config);

                    if let Err(err) = tinted_theming::write_base24_scheme(&scheme, &output_path) {
                        eprintln!("Failed to write scheme: {err}");
                        return;
                    }

                    println!("Generated Base24 scheme: {}", scheme.metadata.name);
                    println!("  Variant: {}", scheme.metadata.variant.as_deref().unwrap_or("unknown"));
                    println!("  Output: {output_path}");
                    println!("\nPreview:");
                    syntax::display_palette_in_terminal(scheme.colors(), Some(&base16_labels(24)));
                }
                _ => {
                    eprintln!("Invalid format: {format}");
                }
            }
        }
        SchemeAction::Show { scheme, format, output, width, height, demo, file } => {
            let schemes_base16 = tinted_theming::load_base16_schemes(&scheme);
            let schemes_base24 = tinted_theming::load_base24_schemes(&scheme);

            let (colors, scheme_name) = if let Ok(schemes) = schemes_base16 {
                (schemes[0].colors().to_vec(), schemes[0].metadata.name.clone())
            } else if let Ok(schemes) = schemes_base24 {
                (schemes[0].colors().to_vec(), schemes[0].metadata.name.clone())
            } else {
                eprintln!("Failed to load scheme: {scheme}");
                return;
            };

            match format.as_str() {
                "terminal" => {
                    println!("Scheme: {scheme_name}");
                    let labels: Vec<String> = (0..colors.len()).map(|i| format!("{i:02X}")).collect();
                    syntax::display_palette_in_terminal(&colors, Some(&labels));

                    if let Some(lang) = demo {
                        if let Some(file_path) = file {
                            println!("\nSyntax demo ({lang}):");

                            let theme = if colors.len() == 16 {
                                if let Ok(schemes) = tinted_theming::load_base16_schemes(&scheme) {
                                    syntax::base16_to_theme(&schemes[0])
                                } else {
                                    eprintln!("Failed to load Base16 scheme");
                                    return;
                                }
                            } else {
                                if let Ok(schemes) = tinted_theming::load_base24_schemes(&scheme) {
                                    syntax::base24_to_theme(&schemes[0])
                                } else {
                                    eprintln!("Failed to load Base24 scheme");
                                    return;
                                }
                            };

                            let syntax_set = syntax::load_syntax_set();
                            if let Some(syntax_ref) = syntax::find_syntax_by_name(&syntax_set, &lang) {
                                if let Ok(file_handle) = File::open(&file_path) {
                                    let reader = BufReader::new(file_handle);
                                    let _ = syntax::highlight_code_to_terminal(
                                        reader,
                                        syntax_ref,
                                        &theme,
                                        Some(&file_path),
                                        Some(&scheme_name),
                                    );
                                } else {
                                    eprintln!("Failed to open file: {file_path}");
                                }
                            } else {
                                eprintln!("Unknown language: {lang}");
                            }
                        }
                    }
                }
                "image" => {
                    let output_path = output.unwrap_or_else(|| "scheme.png".to_string());
                    let labels = base16_labels(colors.len());
                    let image = palette_to_image(&colors, PaletteLabelStyle::Custom(&labels), (width, height));

                    if let Err(err) = image.save(&output_path) {
                        eprintln!("Failed to write image: {err}");
                    } else {
                        println!("Saved scheme visualization: {output_path}");
                    }
                }
                _ => {
                    eprintln!("Invalid format: {format}");
                }
            }
        }
        SchemeAction::Validate { scheme } => {
            let schemes_base16 = tinted_theming::load_base16_schemes(&scheme);
            let schemes_base24 = tinted_theming::load_base24_schemes(&scheme);

            let (colors, scheme_name, system) = if let Ok(schemes) = schemes_base16 {
                (schemes[0].colors().to_vec(), schemes[0].metadata.name.clone(), "Base16")
            } else if let Ok(schemes) = schemes_base24 {
                (schemes[0].colors().to_vec(), schemes[0].metadata.name.clone(), "Base24")
            } else {
                eprintln!("Failed to load scheme: {scheme}");
                return;
            };

            println!("Validating {system} scheme: {scheme_name}");
            println!();

            let mut issues = 0;

            let expected_count = if system == "Base16" { 16 } else { 24 };
            if colors.len() != expected_count {
                println!("  [ERROR] Expected {expected_count} colors, found {}", colors.len());
                issues += 1;
            } else {
                println!("  [OK] Color count: {}", colors.len());
            }

            let mut high_saturation_neutrals = Vec::new();
            for (i, &color) in colors.iter().take(8).enumerate() {
                let hsl: colorizer::colors::Hsl = colorizer::colors::Rgb::from(color).into();
                if hsl.s > base16_builder::NEUTRAL_MAX_SATURATION + NEUTRAL_SATURATION_TOLERANCE {
                    high_saturation_neutrals.push((i, hsl.s));
                }
            }

            if high_saturation_neutrals.is_empty() {
                println!("  [OK] Neutrals (base00-base07) have low saturation");
            } else {
                println!(
                    "  [WARN] Some neutrals have high saturation: {}",
                    high_saturation_neutrals
                        .iter()
                        .map(|(i, s)| format!("{i:02X} ({s:.2})"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            let background = colors[0];
            let mut low_contrast_accents = Vec::new();
            for (i, &color) in colors.iter().enumerate().skip(8).take(8) {
                let ratio = colorizer::wcag::contrast_ratio(background, color);
                if ratio < 4.5 {
                    low_contrast_accents.push((i, ratio));
                }
            }

            if low_contrast_accents.is_empty() {
                println!("  [OK] All accent colors meet WCAG AA contrast (4.5:1) against base00");
            } else {
                println!(
                    "  [ERROR] Low contrast accents: {}",
                    low_contrast_accents
                        .iter()
                        .map(|(i, r)| format!("{i:02X} ({r:.2}:1)"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                issues += 1;
            }

            println!();
            if issues == 0 {
                println!("Validation passed with no errors.");
            } else {
                println!("Validation found {issues} error(s).");
            }
        }
    }
}

fn handle_palette(action: PaletteAction) {
    match action {
        PaletteAction::FromBase {
            base,
            harmony,
            count,
            min_contrast,
            background,
            format,
            save_image,
            image_width,
            image_height,
            image_label,
        } => {
            let base_color = match parse_hex_color(&base) {
                Ok(color) => color,
                Err(err) => {
                    eprintln!("{err}");
                    return;
                }
            };

            let harmony_kind = match parse_harmony_kind(&harmony) {
                Some(kind) => kind,
                None => {
                    eprintln!("Unsupported harmony kind: {harmony}");
                    return;
                }
            };

            let background_color = match background.as_deref().map(parse_hex_color) {
                Some(Ok(color)) => Some(color),
                Some(Err(err)) => {
                    eprintln!("{err}");
                    return;
                }
                None => None,
            };

            let palette = palette_from_base(base_color, harmony_kind, count, None, background_color, min_contrast);
            if palette.is_empty() {
                eprintln!("No colors meet the requested constraints.");
            } else {
                output_palette(&palette, &format);

                if let Some(image_path) = save_image {
                    generate_palette_image(&palette, &image_path, image_width, image_height, &image_label);
                }
            }
        }
        PaletteAction::Random {
            count,
            method,
            min_delta_e,
            theme,
            format,
            save_image,
            image_width,
            image_height,
            image_label,
        } => {
            let palette = match method.as_str() {
                "golden" => {
                    let (s_range, l_range) = golden_theme_ranges(theme.as_deref());
                    golden_ratio_palette(count, s_range, l_range, min_delta_e)
                }
                "uniform" => {
                    let constraints = PaletteConstraints { min_delta_e, ..Default::default() };
                    random::random_palette_with_constraints(count, constraints)
                }
                "poisson" => {
                    let config = PoissonConfig { radius: min_delta_e.unwrap_or(5.0), ..Default::default() };
                    random::poisson_palette(config, count)
                        .into_iter()
                        .map(Srgb8::from)
                        .collect()
                }
                other => {
                    eprintln!("Random method '{other}' is not implemented yet.");
                    Vec::new()
                }
            };

            if palette.is_empty() {
                eprintln!("No colors generated.");
            } else {
                output_palette(&palette, &format);

                if let Some(image_path) = save_image {
                    generate_palette_image(&palette, &image_path, image_width, image_height, &image_label);
                }
            }
        }
        // TODO: add combined JSON/YAML output when directory inputs produce multiple schemes.
        PaletteAction::Base16 { scheme_yaml, format } => match tinted_theming::load_base16_schemes(&scheme_yaml) {
            Ok(schemes) => {
                for scheme in schemes {
                    print_scheme_header(&scheme.metadata);
                    output_palette(scheme.colors(), &format);
                }
            }
            Err(err) => eprintln!("Failed to load Base16 scheme: {err}"),
        },
        // TODO: add combined JSON/YAML output when directory inputs produce multiple schemes.
        PaletteAction::Base24 { scheme_yaml, format } => match tinted_theming::load_base24_schemes(&scheme_yaml) {
            Ok(schemes) => {
                for scheme in schemes {
                    print_scheme_header(&scheme.metadata);
                    output_palette(scheme.colors(), &format);
                }
            }
            Err(err) => eprintln!("Failed to load Base24 scheme: {err}"),
        },
    }
}

fn parse_hex_color(value: &str) -> Result<Srgb8, String> {
    Srgb8::from_hex(value).ok_or_else(|| format!("Invalid color value: {value}"))
}

/// TODO: allow custom angle input
fn parse_harmony_kind(value: &str) -> Option<HarmonyKind> {
    match value {
        "complementary" => Some(HarmonyKind::Complementary),
        "split-complementary" => Some(HarmonyKind::SplitComplementary),
        "analogous" => Some(HarmonyKind::Analogous(30.0)),
        "triadic" => Some(HarmonyKind::Triadic),
        "tetradic" => Some(HarmonyKind::Tetradic),
        "square" => Some(HarmonyKind::Square),
        _ => None,
    }
}

/// TODO: consider richer CLI output (labels, indexes) once UX spec is defined.
fn output_palette(colors: &[Srgb8], format: &str) {
    let hex_values: Vec<String> = colors.iter().map(|c| c.to_hex()).collect();
    match format {
        "json" => match serde_json::to_string_pretty(&hex_values) {
            Ok(serialized) => println!("{serialized}"),
            Err(err) => eprintln!("Failed to serialize palette to JSON: {err}"),
        },
        "yaml" => match serde_yml::to_string(&hex_values) {
            Ok(serialized) => print!("{serialized}"),
            Err(err) => eprintln!("Failed to serialize palette to YAML: {err}"),
        },
        _ => println!("{}", hex_values.join(", ")),
    }
}

fn golden_theme_ranges(theme: Option<&str>) -> (Range<f32>, Range<f32>) {
    match theme {
        Some("light") => (0.25..0.55, 0.6..0.9),
        Some("dark") => (0.45..0.85, 0.2..0.45),
        _ => (0.4..0.8, 0.35..0.7),
    }
}

fn parse_color_list(value: &str) -> Result<Vec<Srgb8>, String> {
    value
        .split(',')
        .map(|segment| parse_hex_color(segment.trim()))
        .collect()
}

fn base16_labels(len: usize) -> Vec<String> {
    const BASE16_KEYS: [&str; 24] = [
        "base00", "base01", "base02", "base03", "base04", "base05", "base06", "base07", "base08", "base09", "base0A",
        "base0B", "base0C", "base0D", "base0E", "base0F", "base10", "base11", "base12", "base13", "base14", "base15",
        "base16", "base17",
    ];
    BASE16_KEYS.iter().take(len).map(|label| label.to_string()).collect()
}

/// Generate and save a palette image with the specified parameters
fn generate_palette_image(palette: &[Srgb8], path: &str, width: u32, height: u32, label_style: &str) {
    let image = match label_style {
        "hex" => palette_to_image(palette, PaletteLabelStyle::Hex, (width, height)),
        "index" => palette_to_image(palette, PaletteLabelStyle::Index, (width, height)),
        "base16" => {
            let labels = base16_labels(palette.len());
            palette_to_image(palette, PaletteLabelStyle::Custom(&labels), (width, height))
        }
        _ => palette_to_image(palette, PaletteLabelStyle::None, (width, height)),
    };

    if let Err(err) = image.save(path) {
        eprintln!("Failed to save palette image to {path}: {err}");
    } else {
        println!("Saved palette image to {path}");
    }
}

fn print_scheme_header(meta: &SchemeMetadata) {
    println!("Scheme: {}", meta.name);
    if let Some(author) = &meta.author {
        println!("  Author: {author}");
    }
    if let Some(variant) = &meta.variant {
        println!("  Variant: {variant}");
    }
}

fn handle_image(
    colors: Option<String>, scheme_yaml: Option<String>, out: String, width: Option<u32>, height: Option<u32>,
    label: String, viz: bool,
) {
    let palette = if let Some(list) = colors {
        match parse_color_list(&list) {
            Ok(colors) => colors,
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        }
    } else if let Some(path) = scheme_yaml {
        eprintln!("Scheme loading from YAML is not implemented yet: {path}");
        return;
    } else {
        eprintln!("Provide either --colors or --scheme-yaml.");
        return;
    };

    if palette.is_empty() {
        eprintln!("No colors provided for image generation.");
        return;
    }

    let size = (width.unwrap_or(960), height.unwrap_or(320));
    let image = match label.as_str() {
        "hex" => palette_to_image(&palette, PaletteLabelStyle::Hex, size),
        "index" => palette_to_image(&palette, PaletteLabelStyle::Index, size),
        "base16" => {
            let labels = base16_labels(palette.len());
            palette_to_image(&palette, PaletteLabelStyle::Custom(&labels), size)
        }
        _ => palette_to_image(&palette, PaletteLabelStyle::None, size),
    };
    if let Err(err) = image.save(&out) {
        eprintln!("Failed to write {out}: {err}");
    } else {
        println!("Wrote palette image to {out}");

        if viz {
            println!();
            let labels: Vec<String> = match label.as_str() {
                "hex" => palette.iter().map(|c| c.to_hex().to_uppercase()).collect(),
                "base16" => base16_labels(palette.len()),
                "index" => (0..palette.len()).map(|i| format!("{:02}", i)).collect(),
                _ => vec![],
            };
            syntax::display_palette_in_terminal(&palette, if labels.is_empty() { None } else { Some(&labels) });
        }
    }
}

fn handle_vim_scheme(scheme_yaml: String, name: String, output_colors: String, update_vimrc: Option<String>) {
    println!("Generating Vim colorscheme '{name}'");
    println!("Scheme: {scheme_yaml}");
    println!("Output directory: {output_colors}");
    if let Some(vimrc) = update_vimrc {
        println!("Updating vimrc: {vimrc}");
    }
}

fn handle_demo(demo_type: DemoType) {
    match demo_type {
        DemoType::Palette { colors, scheme_yaml } => {
            let palette = if let Some(color_list) = colors {
                match parse_color_list(&color_list) {
                    Ok(colors) => colors,
                    Err(err) => {
                        eprintln!("{err}");
                        return;
                    }
                }
            } else if let Some(scheme_path) = scheme_yaml {
                if let Ok(schemes) = tinted_theming::load_base16_schemes(&scheme_path) {
                    schemes[0].colors().to_vec()
                } else if let Ok(schemes) = tinted_theming::load_base24_schemes(&scheme_path) {
                    schemes[0].colors().to_vec()
                } else {
                    eprintln!("Failed to load scheme from {scheme_path}");
                    return;
                }
            } else {
                eprintln!("Provide either --colors or --scheme-yaml");
                return;
            };

            let labels: Vec<String> = (0..palette.len()).map(|i| format!("{i:02X}")).collect();
            syntax::display_palette_in_terminal(&palette, Some(&labels));
        }
        DemoType::Code { language, theme_yaml, base, harmony, file } => {
            let (theme, theme_name) = if let Some(theme_path) = &theme_yaml {
                if let Ok(schemes) = tinted_theming::load_base16_schemes(theme_path) {
                    let name = schemes[0].metadata.name.clone();
                    (syntax::base16_to_theme(&schemes[0]), Some(name))
                } else if let Ok(schemes) = tinted_theming::load_base24_schemes(theme_path) {
                    let name = schemes[0].metadata.name.clone();
                    (syntax::base24_to_theme(&schemes[0]), Some(name))
                } else {
                    eprintln!("Failed to load theme from {theme_path}");
                    return;
                }
            } else if let Some(base_color) = &base {
                let base_srgb = match parse_hex_color(base_color) {
                    Ok(color) => color,
                    Err(err) => {
                        eprintln!("{err}");
                        return;
                    }
                };

                let harmony_kind = harmony
                    .as_ref()
                    .and_then(|h| parse_harmony_kind(h))
                    .unwrap_or(HarmonyKind::Complementary);

                let palette = palette_from_base(base_srgb, harmony_kind, 16, None, None, None);
                if palette.len() < 16 {
                    eprintln!("Failed to generate sufficient colors for theme");
                    return;
                }

                let mut colors = [Srgb8::new(0, 0, 0); 16];
                for (i, &color) in palette.iter().take(16).enumerate() {
                    colors[i] = color;
                }

                let metadata = SchemeMetadata {
                    system: "base16".to_string(),
                    name: "Generated".to_string(),
                    author: None,
                    variant: None,
                };

                let scheme = tinted_theming::Base16Scheme::new(metadata, colors);
                (syntax::base16_to_theme(&scheme), Some("Generated".to_string()))
            } else {
                eprintln!("Provide either --theme-yaml or --base");
                return;
            };

            let syntax_set = syntax::load_syntax_set();
            let syntax = match syntax::find_syntax_by_name(&syntax_set, &language) {
                Some(syn) => syn,
                None => {
                    eprintln!("Unknown language: {language}");
                    return;
                }
            };

            if let Some(file_path) = &file {
                match File::open(file_path) {
                    Ok(file_handle) => {
                        let reader = BufReader::new(file_handle);
                        if let Err(err) = syntax::highlight_code_to_terminal(
                            reader,
                            syntax,
                            &theme,
                            Some(file_path.as_str()),
                            theme_name.as_deref(),
                        ) {
                            eprintln!("Failed to highlight code: {err}");
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to open {file_path}: {err}");
                    }
                }
            } else {
                let stdin = io::stdin();
                let mut code = String::new();
                if let Err(err) = stdin.lock().read_to_string(&mut code) {
                    eprintln!("Failed to read from stdin: {err}");
                    return;
                }

                if let Err(err) = syntax::highlight_string_to_terminal(&code, syntax, &theme, theme_name.as_deref()) {
                    eprintln!("Failed to highlight code: {err}");
                }
            }
        }
    }
}
