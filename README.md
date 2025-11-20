# Colorizer.rs

Colorizer.rs is a command line tool for generating & previewing color palettes and semantic color schemes (Base16/Base24) from a single accent color. It supports various strategies for palette generation like harmony-based derivation & random sampling (golden ratio, Poisson-disk, uniform).

Scheme previews can be rendered as images or terminal output, validated against WCAG contrast ratios, and/or generate Vim colorschemes.
It also includes a "demo" mode to preview syntax-highlighted code samples using the generated palettes.

## Features

- **Vim Integration**
    - Generate Vim colorschemes directly from computed or YAML-based palettes.
    - Optionally update an existing `vimrc` (injecting or switching `colorscheme`) or emit a standalone colorscheme file.
- **Terminal & Code Demos**
    - Use `syntect` for syntax-highlighted code samples driven by your generated palette.
    - Render colored output in the terminal with `owo-colors` truecolor styling.
    - "Demo" mode to quickly preview how a palette feels on real code and in terminal UI.

### Palettes

- **Generation**
    - Generate color palettes from one or more input colors.
    - Support for deterministic harmonies (e.g., complementary, triadic) and fully random palettes.
    - Load Base16 and Base24 palettes from YAML and convert them into internal palettes.
- **Visualization**
    - Render palette previews as images with vertical color bars.
    - Automatically choose white or black text on each bar for optimal readability.
    - Export high-resolution PNGs suitable for docs, theming previews, or social media.

### Schemes

- **Base16/Base24 generation**
    - Produce semantic schemes directly from a single accent color and a harmony strategy.
    - Neutral bases (`base00`-`base07`) are clamped to a maximum of 10% saturation so that backgrounds stay truly neutral.
    - Dial in the overall neutral depth (`--neutral-depth 0-1`) to match bright, moody, or in-between baseline ramps without editing YAML by hand
      (e.g., Oxocarbon Dark ≈ 1.0, Catppuccin Mocha ≈ 0.85, Frappe ≈ 0.7, Macchiato ≈ 0.6).
    - CLI validation checks imported schemes with the same ceiling, warning when backgrounds creep past the neutral limit.

## Quickstart

1. Install the CLI straight from the repo:

    ```bash
    cargo install colorizer --git https://github.com/stormlightlabs/colorizer
    ```

2. Sample candidate accent colors with the randomizer (golden for quick results, Poisson when you want stricter spacing).

   Pick your preferred hex for the next steps:

    ```bash
    colorizer palette random --method golden --count 5 --format hex
    colorizer palette random --method poisson --count 6 --min-delta-e 8 --format hex
    ```

3. Generate a dark Base16 scheme from the chosen accent (neutrals stay ≤10% saturation so backgrounds remain gray):

    ```bash
    colorizer scheme generate base16 \
      --name "Demo Dark" \
      --accent "#61afef" \
      --variant dark \
      --harmony triadic \
      --output demo-dark.yml
    ```

4. Generate the matching light scheme from the same accent:

    ```bash
    colorizer scheme generate base16 \
      --name "Demo Light" \
      --accent "#61afef" \
      --variant light \
      --harmony triadic \
      --output demo-light.yml
    ```

    > Tip: Use `--neutral-depth` to shift between classic bright backgrounds (`0.0`) and the moodier defaults (`1.0`).
    > See *[concepts](./docs/src/concepts.md)* for ready-made values (Oxocarbon, Catppuccin, etc.).

5. Validate both YAML files to confirm neutral saturation and WCAG contrast targets:

    ```bash
    colorizer scheme validate demo-dark.yml
    colorizer scheme validate demo-light.yml
    ```

6. Render each scheme as an image (swap filenames as needed), then drop screenshots where indicated:

    ```bash
    colorizer scheme show demo-dark.yml --format image --output demo-dark.png
    colorizer scheme show demo-light.yml --format image --output demo-light.png
    ```

    **add image here** (dark)

    **add image here** (light)

7. Display both schemes in your terminal to sanity-check ordering and contrast:

    ```bash
    colorizer scheme show demo-dark.yml --format terminal
    colorizer scheme show demo-light.yml --format terminal
    ```

8. Preview syntax highlighting driven by the same schemes (swap files/languages as needed):

    ```bash
    colorizer demo code --theme-yaml demo-dark.yml --file examples/languages/sample.rs
    colorizer demo code --theme-yaml demo-light.yml --file examples/languages/sample.rs
    ```

    | Rust (dark)          | Rust (light)         |
    | -------------------- | -------------------- |
    | **add image here**   | **add image here**   |

For deeper dives into harmonies, neutral depth values, and randomization strategies, see *[concepts](./docs/src/concepts.md)*.
