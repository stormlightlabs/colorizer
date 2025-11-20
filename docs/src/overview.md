# Overview

The `colorizer` CLI bundles together color generation, palette filtering, and visualization tools built on the perceptual color engine in this repository. This guide demonstrates the workflow end-to-end.

## Install

Install the CLI directly from the Git repository (no crates.io release yet):

```bash
cargo install colorizer --git https://github.com/stormlightlabs/colorizer --branch main
```

This compiles the `colorizer` binary and places it in your Cargo bin directory (`~/.cargo/bin` by default).

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

## Next Steps

- Use `--method poisson` on the random command to explore Poisson-disk sampling in Lab/Lch space.
- Experiment with `--label base16` when rendering images to quickly inspect Base16 style slot ordering.
- Wire palette output into downstream tooling (e.g., Vim scheme generation) as the remaining TODO items land.

Refer back to the repository README for feature highlights and keep an eye on the TODO list for upcoming capabilities.
