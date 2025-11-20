use clap::{Parser, Subcommand};
use colorizer::{
    HarmonyKind,
    colors::Srgb8,
    palette::{golden_ratio_palette, palette_from_base},
};
use std::ops::Range;

#[derive(Parser)]
#[command(name = "colorizer")]
#[command(about = "Color palette generation and manipulation tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate and manipulate color palettes
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
        #[arg(long, value_parser = ["base16", "index", "none"], default_value = "index")]
        label: String,
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
        Commands::Palette { action } => handle_palette(action),
        Commands::Image { colors, scheme_yaml, out, width, height, label } => {
            handle_image(colors, scheme_yaml, out, width, height, label)
        }
        Commands::VimScheme { scheme_yaml, name, output_colors, update_vimrc } => {
            handle_vim_scheme(scheme_yaml, name, output_colors, update_vimrc)
        }
        Commands::Demo { demo_type } => handle_demo(demo_type),
    }
}

fn handle_palette(action: PaletteAction) {
    match action {
        PaletteAction::FromBase { base, harmony, count, min_contrast, background, format } => {
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
            }
        }
        PaletteAction::Random { count, method, min_delta_e, theme, format } => {
            let palette = match method.as_str() {
                "golden" => {
                    let (s_range, l_range) = golden_theme_ranges(theme.as_deref());
                    golden_ratio_palette(count, s_range, l_range, min_delta_e)
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
            }
        }
        PaletteAction::Base16 { scheme_yaml, format } => {
            println!("Exporting Base16 palette from {scheme_yaml}");
            println!("Format: {format}");
        }
        PaletteAction::Base24 { scheme_yaml, format } => {
            println!("Exporting Base24 palette from {scheme_yaml}");
            println!("Format: {format}");
        }
    }
}

fn parse_hex_color(value: &str) -> Result<Srgb8, String> {
    Srgb8::from_hex(value).ok_or_else(|| format!("Invalid color value: {value}"))
}

fn parse_harmony_kind(value: &str) -> Option<HarmonyKind> {
    match value {
        "complementary" => Some(HarmonyKind::Complementary),
        "split-complementary" => Some(HarmonyKind::SplitComplementary),
        "analogous" => Some(HarmonyKind::Analogous(30.0)), // TODO: allow custom angle input
        "triadic" => Some(HarmonyKind::Triadic),
        "tetradic" => Some(HarmonyKind::Tetradic),
        "square" => Some(HarmonyKind::Square),
        _ => None,
    }
}

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
        // TODO: consider richer CLI output (labels, indexes) once UX spec is defined.
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

fn handle_image(
    colors: Option<String>, scheme_yaml: Option<String>, out: String, width: Option<u32>, height: Option<u32>,
    label: String,
) {
    println!("Generating palette image: {out}");
    if let Some(c) = colors {
        println!("Colors: {c}");
    }
    if let Some(s) = scheme_yaml {
        println!("Scheme: {s}");
    }
    if let Some(w) = width {
        println!("Width: {w}");
    }
    if let Some(h) = height {
        println!("Height: {h}");
    }
    println!("Label style: {label}");
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
            println!("Displaying palette in terminal");
            if let Some(c) = colors {
                println!("Colors: {c}");
            }
            if let Some(s) = scheme_yaml {
                println!("Scheme: {s}");
            }
        }
        DemoType::Code { language, theme_yaml, base, harmony, file } => {
            println!("Highlighting code in {language}");
            if let Some(t) = theme_yaml {
                println!("Theme: {t}");
            }
            if let Some(b) = base {
                println!("Base color: {b}");
                if let Some(h) = harmony {
                    println!("Harmony: {h}");
                }
            }
            if let Some(f) = file {
                println!("File: {f}");
            } else {
                println!("Reading from stdin");
            }
        }
    }
}
