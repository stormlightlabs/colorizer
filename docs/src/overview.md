# Overview

The `colorizer` CLI bundles together color generation, palette filtering, and visualization tools built on the perceptual color engine in this repository. This guide demonstrates the workflow end-to-end.

## Install

Install the CLI directly from the Git repository (no crates.io release yet):

```bash
cargo install colorizer --git https://github.com/stormlightlabs/colorizer --branch main
```

This compiles the `colorizer` binary and places it in your Cargo bin directory (`~/.cargo/bin` by default).

## Quickstart

1. Sample candidate accent colors with the randomizer (golden is ergonomic, Poisson gives stricter spacing). Choose any of the resulting hex codes:

    ```bash
    colorizer palette random --method golden --count 5 --format hex
    colorizer palette random --method poisson --count 6 --min-delta-e 8 --format hex
    ```

2. Generate a dark Base16 scheme from the chosen accent (neutrals remain ≤10% saturation):

    ```bash
    colorizer scheme generate base16 \
      --name "Demo Dark" \
      --accent "#61afef" \
      --variant dark \
      --harmony triadic \
      --output demo-dark.yml
    ```

    Tip: pass `--neutral-depth 0.0` for classic bright neutrals, `1.0` for the moodier defaults, or anything in between (Oxocarbon Dark ≈ 1.0, Catppuccin Mocha ≈ 0.85, Frappe ≈ 0.7, Macchiato ≈ 0.6).

3. Generate the matching light scheme:

    ```bash
    colorizer scheme generate base16 \
      --name "Demo Light" \
      --accent "#61afef" \
      --variant light \
      --harmony triadic \
      --output demo-light.yml
    ```

4. Validate both YAML files:

    ```bash
    colorizer scheme validate demo-dark.yml
    colorizer scheme validate demo-light.yml
    ```

5. Render and preview each variant (image + terminal):

    ```bash
    colorizer scheme show demo-dark.yml --format image --output demo-dark.png
    colorizer scheme show demo-light.yml --format image --output demo-light.png
    colorizer scheme show demo-dark.yml --format terminal
    colorizer scheme show demo-light.yml --format terminal
    ```

6. Preview syntax highlighting driven by either file:

    ```bash
    colorizer demo code --theme-yaml demo-dark.yml --language rust --file examples/languages/sample.rs
    colorizer demo code --theme-yaml demo-light.yml --language rust --file examples/languages/sample.rs
    ```

## Generate Palettes

### Random palettes

Golden-ratio sampling gives evenly spaced hues with nice defaults for saturation/lightness:

```bash
colorizer palette random --method golden --count 5 --format hex
```

For uniform sampling with perceptual spacing, leverage the constraint solver:

```bash
colorizer palette random --method uniform --count 6 --min-delta-e 8 --format hex
```

### Harmony palettes

Start from a brand/base color and derive complements or triads:

```bash
colorizer palette from-base \
  --base "#ff6600" \
  --harmony complementary \
  --count 6 \
  --min-contrast 4.5 \
  --background "#101010" \
  --format hex
```

The harmony generator expands each hue by lightening/darkening passes and enforces optional WCAG contrast checks.

## Visualize as Images

Take any palette (from the commands above, a YAML scheme, or a comma-separated list) and produce a PNG preview:

```bash
colorizer image \
  --colors "#ff6600,#ffd166,#06d6a0,#118ab2,#073b4c" \
  --out palette.png \
  --width 960 \
  --height 320 \
  --label hex
```

The renderer draws vertical bars, picks white/black text automatically for each hex label, and saves PNG/JPEG based on the file extension.

## Preview in Terminal

Visualize palettes and syntax-highlighted code directly in your terminal without creating files.

### Display palette with colored blocks

```bash
colorizer demo palette \
  --colors "#ff6600,#ffd166,#06d6a0,#118ab2,#073b4c"
```

Shows each color as a filled terminal block with automatic contrast-aware text labels.

### Syntax-highlight code samples

Apply a theme to real code and render with truecolor ANSI escapes:

```bash
colorizer demo code \
  --theme-yaml examples/base16/oxocarbon-dark.yml \
  --language rust \
  --file examples/languages/sample.rs
```

Or generate a theme on-the-fly from a base color:

```bash
colorizer demo code \
  --base "#61afef" \
  --harmony triadic \
  --language python \
  --file examples/languages/sample.py
```

Supports rust, python, javascript, typescript, go, elm, and many more languages.
