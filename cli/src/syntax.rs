//! Syntax highlighting and terminal color display utilities.
//!
//! Integrates [syntect] for code highlighting and [owo_colors] for terminal output.
//! Maps Base16/Base24 color schemes to syntax highlight themes and renders syntax-highlighted code to the terminal using truecolor ANSI escapes.

use crate::colors::Srgb8;
use crate::tinted_theming::{Base16Scheme, Base24Scheme};

use owo_colors::OwoColorize;
use std::io::{self, BufRead};
use std::str::FromStr;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, FontStyle, ScopeSelectors, Style as SyntectStyle, Theme};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

/// Displays a palette as colored terminal blocks with labels.
///
/// Each color is shown as a colored line with its hex code and optional label.
pub fn display_palette_in_terminal(colors: &[Srgb8], labels: Option<&[String]>) {
    for (idx, &color) in colors.iter().enumerate() {
        let label = labels
            .and_then(|l| l.get(idx))
            .map(|s: &String| s.as_str())
            .unwrap_or("");

        let (fg_r, fg_g, fg_b) = if is_light(color) { (0, 0, 0) } else { (255, 255, 255) };

        let block = format!("████████████  {:<10} {}", label, color.to_hex());
        println!(
            "{}",
            block
                .on_truecolor(color.r, color.g, color.b)
                .truecolor(fg_r, fg_g, fg_b)
        );
    }
}

/// Determines if a color is "light" using a simple luminance heuristic.
fn is_light(color: Srgb8) -> bool {
    let luminance = 0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32;
    luminance > 127.5
}

/// Converts a Base16 scheme to a syntect Theme.
///
/// Maps Base16 colors to syntax scopes according to tinted-theming guidelines:
/// - base00: background
/// - base05: foreground
/// - base08: variables, tags (red)
/// - base09: numbers, constants (orange)
/// - base0A: classes (yellow)
/// - base0B: strings (green)
/// - base0C: support, regex (cyan)
/// - base0D: functions (blue)
/// - base0E: keywords (magenta)
/// - base0F: deprecated (brown)
pub fn base16_to_theme(scheme: &Base16Scheme) -> Theme {
    let colors = scheme.colors();

    Theme {
        name: Some(scheme.metadata.name.clone()),
        author: scheme.metadata.author.clone(),
        settings: syntect::highlighting::ThemeSettings {
            foreground: Some(to_syntect_color(colors[5])),
            background: Some(to_syntect_color(colors[0])),
            caret: Some(to_syntect_color(colors[5])),
            line_highlight: Some(to_syntect_color(colors[1])),
            misspelling: None,
            minimap_border: None,
            accent: None,
            popup_css: None,
            phantom_css: None,
            bracket_contents_foreground: None,
            bracket_contents_options: None,
            brackets_foreground: None,
            brackets_background: None,
            brackets_options: None,
            tags_foreground: None,
            tags_options: None,
            highlight: None,
            find_highlight: None,
            find_highlight_foreground: None,
            gutter: None,
            gutter_foreground: Some(to_syntect_color(colors[3])),
            selection: Some(to_syntect_color(colors[2])),
            selection_foreground: None,
            selection_border: None,
            inactive_selection: None,
            inactive_selection_foreground: None,
            guide: None,
            active_guide: None,
            stack_guide: None,
            shadow: None,
        },
        scopes: vec![
            scope_item("", colors[5], FontStyle::empty()),
            scope_item("comment", colors[3], FontStyle::empty()),
            scope_item("keyword", colors[14], FontStyle::empty()),
            scope_item("storage", colors[14], FontStyle::empty()),
            scope_item("string", colors[11], FontStyle::empty()),
            scope_item("entity.name.function", colors[13], FontStyle::empty()),
            scope_item("support.function", colors[13], FontStyle::empty()),
            scope_item("entity.name.class", colors[10], FontStyle::empty()),
            scope_item("entity.name.type", colors[10], FontStyle::empty()),
            scope_item("support.type", colors[10], FontStyle::empty()),
            scope_item("variable", colors[8], FontStyle::empty()),
            scope_item("entity.name.tag", colors[8], FontStyle::empty()),
            scope_item("constant.numeric", colors[9], FontStyle::empty()),
            scope_item("constant.language", colors[9], FontStyle::empty()),
            scope_item("constant.character", colors[9], FontStyle::empty()),
            scope_item("support", colors[12], FontStyle::empty()),
            scope_item("string.regexp", colors[12], FontStyle::empty()),
            scope_item("keyword.operator", colors[5], FontStyle::empty()),
            scope_item("invalid.deprecated", colors[15], FontStyle::empty()),
        ],
    }
}

/// Converts a Base24 scheme to a syntect [Theme].
pub fn base24_to_theme(scheme: &Base24Scheme) -> Theme {
    let colors = scheme.colors();

    Theme {
        name: Some(scheme.metadata.name.clone()),
        author: scheme.metadata.author.clone(),
        settings: syntect::highlighting::ThemeSettings {
            foreground: Some(to_syntect_color(colors[5])),
            background: Some(to_syntect_color(colors[0])),
            caret: Some(to_syntect_color(colors[5])),
            line_highlight: Some(to_syntect_color(colors[1])),
            misspelling: None,
            minimap_border: None,
            accent: None,
            popup_css: None,
            phantom_css: None,
            bracket_contents_foreground: None,
            bracket_contents_options: None,
            brackets_foreground: None,
            brackets_background: None,
            brackets_options: None,
            tags_foreground: None,
            tags_options: None,
            highlight: None,
            find_highlight: None,
            find_highlight_foreground: None,
            gutter: None,
            gutter_foreground: Some(to_syntect_color(colors[3])),
            selection: Some(to_syntect_color(colors[2])),
            selection_foreground: None,
            selection_border: None,
            inactive_selection: None,
            inactive_selection_foreground: None,
            guide: None,
            active_guide: None,
            stack_guide: None,
            shadow: None,
        },
        scopes: vec![
            scope_item("", colors[5], FontStyle::empty()),
            scope_item("comment", colors[3], FontStyle::empty()),
            scope_item("keyword", colors[14], FontStyle::empty()),
            scope_item("storage", colors[14], FontStyle::empty()),
            scope_item("string", colors[11], FontStyle::empty()),
            scope_item("entity.name.function", colors[13], FontStyle::empty()),
            scope_item("support.function", colors[13], FontStyle::empty()),
            scope_item("entity.name.class", colors[10], FontStyle::empty()),
            scope_item("entity.name.type", colors[10], FontStyle::empty()),
            scope_item("support.type", colors[10], FontStyle::empty()),
            scope_item("variable", colors[8], FontStyle::empty()),
            scope_item("entity.name.tag", colors[8], FontStyle::empty()),
            scope_item("constant.numeric", colors[9], FontStyle::empty()),
            scope_item("constant.language", colors[9], FontStyle::empty()),
            scope_item("constant.character", colors[9], FontStyle::empty()),
            scope_item("support", colors[12], FontStyle::empty()),
            scope_item("string.regexp", colors[12], FontStyle::empty()),
            scope_item("keyword.operator", colors[5], FontStyle::empty()),
            scope_item("invalid.deprecated", colors[15], FontStyle::empty()),
        ],
    }
}

/// Helper to create a scope item for theme definition.
fn scope_item(scope: &str, color: Srgb8, font_style: FontStyle) -> syntect::highlighting::ThemeItem {
    syntect::highlighting::ThemeItem {
        scope: ScopeSelectors::from_str(scope).unwrap(),
        style: syntect::highlighting::StyleModifier {
            foreground: Some(to_syntect_color(color)),
            background: None,
            font_style: Some(font_style),
        },
    }
}

/// Converts Srgb8 to syntect Color.
fn to_syntect_color(color: Srgb8) -> Color {
    Color { r: color.r, g: color.g, b: color.b, a: 255 }
}

/// Highlights source code and prints it to the terminal with colors.
///
/// Reads code from the provided reader, highlights it using the theme and syntax, and outputs each line with ANSI color codes to the terminal.
pub fn highlight_code_to_terminal<R: BufRead>(reader: R, syntax: &SyntaxReference, theme: &Theme) -> io::Result<()> {
    let mut highlighter = HighlightLines::new(syntax, theme);

    for line in reader.lines() {
        let line = line?;
        let line_with_newline = format!("{}\n", line);

        let ranges = highlighter
            .highlight_line(&line_with_newline, &SyntaxSet::load_defaults_newlines())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        print_highlighted_line(&ranges);
    }

    Ok(())
}

/// Highlights source code from a string and prints to terminal.
pub fn highlight_string_to_terminal(code: &str, syntax: &SyntaxReference, theme: &Theme) -> io::Result<()> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let mut highlighter = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter
            .highlight_line(line, &syntax_set)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        print_highlighted_line(&ranges);
    }

    Ok(())
}

/// Prints a highlighted line to the terminal using owo-colors.
fn print_highlighted_line(ranges: &[(SyntectStyle, &str)]) {
    for (style, text) in ranges {
        let output = text.truecolor(style.foreground.r, style.foreground.g, style.foreground.b);

        if style.font_style.contains(FontStyle::BOLD) {
            if style.font_style.contains(FontStyle::ITALIC) {
                if style.font_style.contains(FontStyle::UNDERLINE) {
                    print!("{}", output.bold().italic().underline());
                } else {
                    print!("{}", output.bold().italic());
                }
            } else if style.font_style.contains(FontStyle::UNDERLINE) {
                print!("{}", output.bold().underline());
            } else {
                print!("{}", output.bold());
            }
        } else if style.font_style.contains(FontStyle::ITALIC) {
            if style.font_style.contains(FontStyle::UNDERLINE) {
                print!("{}", output.italic().underline());
            } else {
                print!("{}", output.italic());
            }
        } else if style.font_style.contains(FontStyle::UNDERLINE) {
            print!("{}", output.underline());
        } else {
            print!("{}", output);
        }
    }
}

/// Loads the default syntax set from syntect.
pub fn load_syntax_set() -> SyntaxSet {
    SyntaxSet::load_defaults_newlines()
}

/// Finds a syntax by language name (e.g., "rust", "python").
///
/// Case-insensitive search that tries both the name and extension.
pub fn find_syntax_by_name<'a>(syntax_set: &'a SyntaxSet, name: &str) -> Option<&'a SyntaxReference> {
    syntax_set
        .find_syntax_by_name(name)
        .or_else(|| syntax_set.syntaxes().iter().find(|s| s.name.eq_ignore_ascii_case(name)))
        .or_else(|| syntax_set.find_syntax_by_extension(name))
        .or_else(|| {
            syntax_set
                .syntaxes()
                .iter()
                .find(|s| s.file_extensions.iter().any(|ext| ext.eq_ignore_ascii_case(name)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tinted_theming;

    #[test]
    fn base16_theme_has_correct_colors() {
        let schemes = tinted_theming::load_base16_schemes("../examples/base16/oxocarbon-dark.yml").unwrap();
        let theme = base16_to_theme(&schemes[0]);

        assert!(theme.name.is_some());
        assert!(theme.settings.foreground.is_some());
        assert!(theme.settings.background.is_some());
        assert!(!theme.scopes.is_empty());
    }

    #[test]
    fn display_palette_does_not_panic() {
        let colors = vec![Srgb8::new(255, 0, 0), Srgb8::new(0, 255, 0), Srgb8::new(0, 0, 255)];
        let labels = vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()];
        display_palette_in_terminal(&colors, Some(&labels));
    }
}
