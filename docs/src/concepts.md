# Concepts

This page surveys the core ideas behind Colorizer: the color theory primitives we expose through the CLI and the algorithms that make "random" palettes feel intentional.

## Color theory cheatsheet

- **Hue** - angle on the color wheel (0-360°). We derive accents by rotating the hue of your seed color.
- **Saturation** - intensity of a color. Neutral backgrounds hold saturation ≤10% so they read as gray.
- **Lightness** - perceived brightness. The `--neutral-depth` slider blends between classic Base16 lightness values and deeper ones modeled after Oxocarbon/Catppuccin.
- **Complementary** - hues opposite each other (~180° apart). Great for high contrast.
- **Analogous** - neighboring hues (±30°). Cohesive, low-contrast palettes.
- **Triadic / Tetradic / Square** - evenly spaced points around the wheel. Balance variety with harmony.

### Base16 basics

- `base00`-`base07` are neutral backgrounds/foregrounds. We auto-clamp saturation and let you interpolate lightness.
- `base08`-`base0F` are semantic accents (variables, strings, keywords, deprecated).
- Light vs dark variants swap the neutral ramp direction.

## Neutral depth

`--neutral-depth` applies to scheme generation:

- `0.0` - classic Base16 values (e.g., `base00 ≈ #4d4f53` for dark themes).
- `~0.6` - Catppuccin Macchiato / Frappe territory.
- `0.85` - Catppuccin Mocha.
- `1.0` - Oxocarbon Dark (`base00 = #161616`, `base01 = #262626`).

Light variants mirror the same scale so you can make cohesive pairs by reusing depth across both variants.

## Harmony choices

When generating accents (palette `from-base` or `scheme generate`), pick a harmony that matches the mood:

- `analogous` - subtle variations near the seed hue.
- `complementary` - stark warm/cool contrast.
- `split-complementary` - complement ±30° for a softer take.
- `triadic` - evenly spaced 120° rotations (balanced variety).
- `tetradic`/`square` - four-hue meshes, good for UI with many semantic roles.

## Randomization algorithms

### Golden ratio sampling

- Uses the golden ratio conjugate (≈0.618) to step through hue space, creating evenly spaced colors without clustering.
- Saturation/lightness ranges can be fixed or sampled per theme (`--theme dark|light` adjusts ranges).
- Great for "balanced" palettes with minimal configuration.

### Poisson-disk sampling

- Works in Lab/Lch space to enforce a minimum ΔE distance between colors.
- `--min-delta-e` controls how distinct colors must be; higher values → fewer, more separated hues.
- Optionally biases toward light/dark themes.

### Uniform sampling

- Pure random draws (optionally filtered by min ΔE / contrast).
- Use when you want complete control or to feed custom constraints.

## Contrast and WCAG

- We compute contrast ratios in linear RGB (WCAG 2.1) for palette filtering and scheme validation.
- `--min-contrast` (palette generation) ensures colors stay legible against a background.
- `colorizer scheme validate` warns when neutrals exceed saturation or accents drop below 4.5:1 against `base00`.

## Terminal demo anatomy

`colorizer demo code` renders:

- Code panel background = `base00`.
- Status bar background = `base01`.
- `syntect` mapping uses `base05` for foreground, `base03` for comments, `base08`-`base0F` for semantic scopes.

## Putting it together

1. Use `palette random --method golden|poisson` to audition accent seeds.
2. Pick a harmony that fits the mood (analogous vs complementary).
3. Generate dark/light schemes with the same accent and match `--neutral-depth`.
4. Validate + demo to ensure WCAG compliance.
5. Export to YAML/PNG or feed into your editor via `colorizer vim-scheme`.

This toolkit keeps the math predictable while giving you wiggle room to express your style.
