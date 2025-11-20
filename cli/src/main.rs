use clap::{Parser, Subcommand};
use colorizer::{
    HarmonyKind,
    colors::Srgb8,
    palette::{PaletteLabelStyle, golden_ratio_palette, palette_from_base, palette_to_image},
    random::{self, PaletteConstraints, PoissonConfig},
    syntax,
    tinted_theming::{self, SchemeMetadata},
};
use std::fs::File;
use std::io::{self, BufReader, Read};
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
                "uniform" => {
                    let mut constraints = PaletteConstraints::default();
                    constraints.min_delta_e = min_delta_e;
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
    label: String,
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
                // Try Base16 first, then Base24
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

            let labels: Vec<String> = (0..palette.len()).map(|i| format!("base{:02X}", i)).collect();

            syntax::display_palette_in_terminal(&palette, Some(&labels));
        }
        DemoType::Code { language, theme_yaml, base, harmony, file } => {
            let theme = if let Some(theme_path) = theme_yaml {
                if let Ok(schemes) = tinted_theming::load_base16_schemes(&theme_path) {
                    syntax::base16_to_theme(&schemes[0])
                } else if let Ok(schemes) = tinted_theming::load_base24_schemes(&theme_path) {
                    syntax::base24_to_theme(&schemes[0])
                } else {
                    eprintln!("Failed to load theme from {theme_path}");
                    return;
                }
            } else if let Some(base_color) = base {
                let base_srgb = match parse_hex_color(&base_color) {
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
                syntax::base16_to_theme(&scheme)
            } else {
                eprintln!("Provide either --theme-yaml or --base");
                return;
            };

            // Load syntax
            let syntax_set = syntax::load_syntax_set();
            let syntax = match syntax::find_syntax_by_name(&syntax_set, &language) {
                Some(syn) => syn,
                None => {
                    eprintln!("Unknown language: {language}");
                    return;
                }
            };

            if let Some(file_path) = file {
                match File::open(&file_path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(err) = syntax::highlight_code_to_terminal(reader, syntax, &theme) {
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

                if let Err(err) = syntax::highlight_string_to_terminal(&code, syntax, &theme) {
                    eprintln!("Failed to highlight code: {err}");
                }
            }
        }
    }
}
