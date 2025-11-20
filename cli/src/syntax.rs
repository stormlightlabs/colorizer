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

const PANEL_BORDER_COLOR: (u8, u8, u8) = (100, 100, 100);
const STATUS_BAR_BG: (u8, u8, u8) = (60, 60, 60);
const STATUS_BAR_FG: (u8, u8, u8) = (220, 220, 220);

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

/// Highlights source code and prints it to the terminal with colors in a bordered panel.
///
/// Reads code from the provided reader, highlights it using the theme and syntax, and outputs each line with ANSI color codes to the terminal.
/// The code is wrapped in a box with a status bar showing file and theme information.
pub fn highlight_code_to_terminal<R: BufRead>(
    reader: R, syntax: &SyntaxReference, theme: &Theme, file_path: Option<&str>, theme_name: Option<&str>,
) -> io::Result<()> {
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut highlighted_lines = Vec::new();
    let mut max_width = 0;
    let panel_bg = theme.settings.background.map(color_tuple_from_syntect);
    let status_bg = theme.settings.line_highlight.map(color_tuple_from_syntect);
    let status_fg = theme.settings.foreground.map(color_tuple_from_syntect);

    for line in reader.lines() {
        let line = line?;
        let line_with_newline = format!("{line}\n");

        let ranges = highlighter
            .highlight_line(&line_with_newline, &load_syntax_set())
            .map_err(io::Error::other)?;

        let line_str = render_highlighted_line(&ranges, panel_bg);
        let visible_width = line.chars().count();
        max_width = max_width.max(visible_width);
        highlighted_lines.push((line_str, visible_width));
    }

    draw_code_panel(
        &highlighted_lines,
        max_width,
        file_path,
        theme_name,
        syntax.name.as_str(),
        panel_bg,
        status_bg,
        status_fg,
    );

    Ok(())
}

/// Highlights source code from a string and prints to terminal in a bordered panel.
pub fn highlight_string_to_terminal(
    code: &str, syntax: &SyntaxReference, theme: &Theme, theme_name: Option<&str>,
) -> io::Result<()> {
    let syntax_set = load_syntax_set();
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut highlighted_lines = Vec::new();
    let mut max_width = 0;
    let panel_bg = theme.settings.background.map(color_tuple_from_syntect);
    let status_bg = theme.settings.line_highlight.map(color_tuple_from_syntect);
    let status_fg = theme.settings.foreground.map(color_tuple_from_syntect);

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter
            .highlight_line(line, &syntax_set)
            .map_err(io::Error::other)?;

        let line_str = render_highlighted_line(&ranges, panel_bg);
        let visible_width = line.trim_end().chars().count();
        max_width = max_width.max(visible_width);
        highlighted_lines.push((line_str, visible_width));
    }

    draw_code_panel(
        &highlighted_lines,
        max_width,
        None,
        theme_name,
        syntax.name.as_str(),
        panel_bg,
        status_bg,
        status_fg,
    );

    Ok(())
}

/// Renders a highlighted line to a String with ANSI codes.
fn render_highlighted_line(ranges: &[(SyntectStyle, &str)], panel_bg: Option<(u8, u8, u8)>) -> String {
    let mut result = String::new();

    for (style, text) in ranges {
        let text_without_newline = text.trim_end_matches('\n').trim_end_matches('\r');

        if text_without_newline.is_empty() {
            continue;
        }

        let mut segment = String::new();
        segment.push_str(&ansi_fg(style.foreground.r, style.foreground.g, style.foreground.b));

        if let Some((bg_r, bg_g, bg_b)) = style_background(style, panel_bg) {
            segment.push_str(&ansi_bg(bg_r, bg_g, bg_b));
        }

        if style.font_style.contains(FontStyle::BOLD) {
            segment.push_str("\x1b[1m");
        }
        if style.font_style.contains(FontStyle::ITALIC) {
            segment.push_str("\x1b[3m");
        }
        if style.font_style.contains(FontStyle::UNDERLINE) {
            segment.push_str("\x1b[4m");
        }

        segment.push_str(text_without_newline);
        segment.push_str("\x1b[0m");
        result.push_str(&segment);
    }

    result
}

/// Draws a bordered panel around code with a status bar at the bottom.
fn draw_code_panel(
    lines: &[(String, usize)], max_width: usize, file_path: Option<&str>, theme_name: Option<&str>, language: &str,
    panel_bg: Option<(u8, u8, u8)>, status_bg: Option<(u8, u8, u8)>, status_fg: Option<(u8, u8, u8)>,
) {
    let panel_width = max_width.max(50).min(120);
    let (border_r, border_g, border_b) = PANEL_BORDER_COLOR;
    let top_border = format!("┌{}┐", "─".repeat(panel_width + 2));
    println!("{}", top_border.truecolor(border_r, border_g, border_b));

    for (line, visible_width) in lines {
        let padding =
            if *visible_width < panel_width { " ".repeat(panel_width - visible_width) } else { String::new() };

        print!("{}", "│ ".truecolor(border_r, border_g, border_b));
        print!("{}", line);

        if let Some((bg_r, bg_g, bg_b)) = panel_bg {
            let padded = format!("{}", padding.on_truecolor(bg_r, bg_g, bg_b));
            println!("{}{}", padded, " │".truecolor(border_r, border_g, border_b));
        } else {
            println!("{}{}", padding, " │".truecolor(border_r, border_g, border_b));
        }
    }

    let bottom_border = format!("└{}┘", "─".repeat(panel_width + 2));
    println!("{}", bottom_border.truecolor(border_r, border_g, border_b));

    let (status_bg_r, status_bg_g, status_bg_b) = status_bg.unwrap_or(STATUS_BAR_BG);
    let status_fg_from_theme = status_fg.unwrap_or(STATUS_BAR_FG);
    let (status_fg_r, status_fg_g, status_fg_b) =
        pick_contrasting_text(status_bg.unwrap_or(STATUS_BAR_BG), status_fg_from_theme);

    let file_info = file_path.unwrap_or("stdin");
    let theme_info = theme_name.unwrap_or("custom");
    let status_text = format!(" {} | {} | {} ", file_info, language, theme_info);

    let total_width = panel_width + 4;
    let status_text_len = status_text.chars().count();
    let status_padding = if status_text_len < total_width {
        " ".repeat(total_width - status_text_len)
    } else {
        String::new()
    };

    let full_status = format!("{}{}", status_text, status_padding);
    println!(
        "{}",
        full_status
            .on_truecolor(status_bg_r, status_bg_g, status_bg_b)
            .truecolor(status_fg_r, status_fg_g, status_fg_b)
    );
}

fn color_tuple_from_syntect(color: Color) -> (u8, u8, u8) {
    (color.r, color.g, color.b)
}

fn pick_contrasting_text(bg: (u8, u8, u8), preferred: (u8, u8, u8)) -> (u8, u8, u8) {
    if is_light(Srgb8::new(bg.0, bg.1, bg.2)) {
        if is_light(Srgb8::new(preferred.0, preferred.1, preferred.2)) { (0, 0, 0) } else { preferred }
    } else if !is_light(Srgb8::new(preferred.0, preferred.1, preferred.2)) {
        (255, 255, 255)
    } else {
        preferred
    }
}

fn ansi_fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

fn ansi_bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{r};{g};{b}m")
}
fn style_background(style: &SyntectStyle, panel_bg: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
    if style.background.a > 0 && (style.background.r != 0 || style.background.g != 0 || style.background.b != 0) {
        Some((style.background.r, style.background.g, style.background.b))
    } else {
        panel_bg
    }
}

/// Loads the extended syntax set with support for 100+ languages including TypeScript and Elm.
pub fn load_syntax_set() -> SyntaxSet {
    two_face::syntax::extra_newlines()
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
