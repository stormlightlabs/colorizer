# Workflows

This guide demonstrates end-to-end workflows combining palette generation, visualization, and terminal demos.

## Triadic Harmony Pipeline

Generate a triadic harmony palette from a base color, filter by contrast, and render as an image.

### Generate palette and image

Start with a brand color and derive triadic harmonies with WCAG AA contrast against a dark background:

```bash
colorizer palette from-base \
  --base "#ff6600" \
  --harmony triadic \
  --count 9 \
  --min-contrast 4.5 \
  --background "#0f0f0f" \
  --save-image triadic-palette.png \
  --image-width 1200 \
  --image-height 400 \
  --image-label hex
```

This produces 9 colors spaced around the color wheel at 120° intervals, expanded with tints and shades, filtered to ensure readable text on dark backgrounds. The palette is output to stdout and saved as an image with hex labels.

## Base16 Scheme Demo

Load a tinted-theming scheme, export the palette, generate an image, and syntax-highlight code samples.

### Export palette from YAML

```bash
colorizer palette base16 \
  --scheme-yaml examples/base16/oxocarbon-dark.yml \
  --format hex
```

Outputs the 16 base colors as comma-separated hex codes.

### Generate scheme image

Render the scheme with Base16 labels:

```bash
colorizer image \
  --scheme-yaml examples/base16/oxocarbon-dark.yml \
  --out oxocarbon-dark.png \
  --width 960 \
  --height 320 \
  --label base16
```

Labels each bar with `base00` through `base0F` for easy reference against tinted-theming docs.

### Syntax highlight code samples

Apply the scheme to real source code and render in the terminal with full truecolor support:

```bash
colorizer demo code \
  --theme-yaml examples/base16/oxocarbon-dark.yml \
  --language rust \
  --file examples/languages/sample.rs
```

Supports multiple languages (rust, python, javascript, typescript, go, elm, and more). Reads from stdin if `--file` is omitted.

### Try different languages

```bash
# Python
colorizer demo code \
  --theme-yaml examples/base24/catppuccin-mocha.yml \
  --language python \
  --file examples/languages/sample.py

# TypeScript
colorizer demo code \
  --theme-yaml examples/base24/catppuccin-frappe.yml \
  --language typescript \
  --file examples/languages/sample.ts
```

## Poisson-Disk Palette Showcase

Generate perceptually uniform palettes using Poisson-disk sampling in Lab/Lch space.

### Basic Poisson sampling

```bash
colorizer palette random \
  --method poisson \
  --count 8 \
  --min-delta-e 12 \
  --format hex
```

The `min-delta-e` parameter controls the minimum perceptual distance between colors. Higher values produce more distinct colors.

### Light theme variant

```bash
colorizer palette random \
  --method poisson \
  --count 6 \
  --min-delta-e 10 \
  --theme light \
  --format hex
```

The `--theme light` flag biases sampling toward lighter, more saturated colors suitable for light backgrounds.

### Dark theme variant

```bash
colorizer palette random \
  --method poisson \
  --count 6 \
  --min-delta-e 10 \
  --theme dark \
  --format hex
```

### Visualize and compare

Generate multiple Poisson palettes and compare them side by side:

```bash
# Generate first palette with image
colorizer palette random \
  --method poisson \
  --count 6 \
  --min-delta-e 15 \
  --save-image poisson-sparse.png

# Generate second with tighter spacing
colorizer palette random \
  --method poisson \
  --count 12 \
  --min-delta-e 8 \
  --save-image poisson-dense.png
```

## Golden Ratio Palette Generation

Generate aesthetically balanced palettes using golden ratio hue stepping.

### Default (balanced)

```bash
colorizer palette random \
  --method golden \
  --count 8 \
  --format hex
```

Uses moderate saturation (0.4-0.8) and lightness (0.35-0.7) ranges for versatile colors.

### Light theme

```bash
colorizer palette random \
  --method golden \
  --count 6 \
  --theme light \
  --format hex
```

Reduces saturation (0.25-0.55) and increases lightness (0.6-0.9) for pastel tones on light backgrounds.

### Dark theme

```bash
colorizer palette random \
  --method golden \
  --count 6 \
  --theme dark \
  --format hex
```

Increases saturation (0.45-0.85) and reduces lightness (0.2-0.45) for vibrant colors on dark backgrounds.

## Ad-Hoc Theme Generation

Quickly generate a syntax theme from any base color without saving a scheme file.

### From base color with harmony

```bash
colorizer demo code \
  --base "#e06c75" \
  --harmony complementary \
  --language rust \
  --file examples/languages/sample.rs
```

Generates a 16-color Base16-style theme on the fly using the specified harmony and applies it to syntax highlighting.

### Available harmonies

- `complementary`: opposite on the color wheel (180°)
- `split-complementary`: base + two neighbors of complement
- `analogous`: adjacent hues (±30°)
- `triadic`: evenly spaced at 120° intervals
- `tetradic`: two complementary pairs (90° spacing)
- `square`: four evenly spaced hues (90° spacing)

### Example workflow

```bash
# Try different harmonies on the same code
for harmony in complementary triadic tetradic square; do
  echo "=== $harmony ==="
  colorizer demo code \
    --base "#61afef" \
    --harmony "$harmony" \
    --language python \
    --file examples/languages/sample.py
  sleep 2
done
```

## Export and Integration

### JSON output for tooling

```bash
colorizer palette random --method golden --count 16 --format json > palette.json
```

Produces a JSON array of hex strings suitable for scripts and build pipelines.

### YAML output for configuration

```bash
colorizer palette from-base \
  --base "#98c379" \
  --harmony analogous \
  --count 8 \
  --format yaml > palette.yml
```

### Generate palette with image output

```bash
colorizer palette random \
  --method poisson \
  --count 5 \
  --min-delta-e 12 \
  --save-image random-palette.png \
  --image-width 1200 \
  --image-height 400 \
  --image-label hex
```

Generates both palette output and a visualization image in a single command.

## Advanced Constraint-Based Generation

Use the uniform method with perceptual spacing constraints.

### Minimum spacing

```bash
colorizer palette random \
  --method uniform \
  --count 6 \
  --min-delta-e 15 \
  --format hex
```

Enforces a minimum ΔE2000 distance of 15 between all color pairs, ensuring highly distinguishable colors.

### Increase color count

```bash
colorizer palette random \
  --method uniform \
  --count 12 \
  --min-delta-e 8 \
  --format hex
```

Lower spacing allows more colors while maintaining perceptual separation.

## Terminal Palette Visualization

Preview any palette without creating image files.

### From hex list

```bash
colorizer demo palette \
  --colors "#ff6188,#fc9867,#ffd866,#a9dc76,#78dce8,#ab9df2,#ff6188"
```

### From scheme file

```bash
colorizer demo palette \
  --scheme-yaml examples/base24/catppuccin-latte.yml
```

Loads Base16 or Base24 schemes automatically and displays all colors with labels.
