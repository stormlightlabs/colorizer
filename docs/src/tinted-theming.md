# Tinted Theming

## Base16

Source: <https://github.com/chriskempson/base16>

| base0X   | Terminal                   | Text Editor                                                         |
| -------- | -------------------------- | ------------------------------------------------------------------- |
| `base00` | Black (Background)         | Default Background                                                  |
| `base01` | (Darkest Gray)             | Lighter Background (Used for status bars)                           |
| `base02` | (Dark Gray)                | Selection Background                                                |
| `base03` | Bright Black (Gray)        | Comments, Invisibles, Line Highlighting                             |
| `base04` | (Light Gray)               | Dark Foreground (Used for status bars)                              |
| `base05` | White                      | Default Foreground, Caret, Delimiters, Operators                    |
| `base06` | (Lighter White)            | Light Foreground                                                    |
| `base07` | Bright White               | The Lightest Foreground                                             |
| `base08` | Red and Bright Red         | Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted   |
| `base09` | (Orange)                   | Integers, Boolean, Constants, XML Attributes, Markup Link Url       |
| `base0A` | Yellow and Bright Yellow   | Classes, Markup Bold, Search Text Background                        |
| `base0B` | Green and Bright Green     | Strings, Inherited Class, Markup Code, Diff Inserted                |
| `base0C` | Cyan and Bright Cyan       | Support, Regular Expressions, Escape Characters, Markup Quotes      |
| `base0D` | Blue and Bright Blue       | Functions, Methods, Attribute IDs, Headings                         |
| `base0E` | Magenta and Bright Magenta | Keywords, Storage, Selector, Markup Italic, Diff Changed            |
| `base0F` | (Dark Red or Brown)        | Deprecated, Opening/Closing Embedded Language Tags, e.g. `<?php ?>` |

### Dark Theme

- Colors from base00 to base07 should range from dark to light.
- Colors base10 to base11 should span from light to dark, but still darker than base00.

### Light Theme

- Colors from base00 to base07 should range from light to dark.
- Colors base10 to base11 should span from dark to light, but lighter than base00.

## Base24

Source: <https://github.com/tinted-theming/base24/>

### Mapping

| Base24   | Base16   |
| -------- | -------- |
| `base10` | `base00` |
| `base11` | `base00` |
| `base12` | `base08` |
| `base13` | `base0A` |
| `base14` | `base0B` |
| `base15` | `base0C` |
| `base16` | `base0D` |
| `base17` | `base0E` |

### Guidelines

| Base0X   | Terminal/Color Use  | Text Editor                                                                                                 |
| -------- | ------------------- | ----------------------------------------------------------------------------------------------------------- |
| `base00` | Background          | Default Background                                                                                          |
| `base01` | (Darkest Gray)      | Lighter Background (Used for status bars)                                                                   |
| `base02` | Bright Black        | Selection Background                                                                                        |
| `base03` | (Gray)              | Comments, Invisibles, Line Highlighting                                                                     |
| `base04` | (Light Gray)        | Dark Foreground (Used for status bars)                                                                      |
| `base05` | Foreground          | Default Foreground, Caret, Delimiters, Operators                                                            |
| `base06` | White               | Light Foreground (Not often used)                                                                           |
| `base07` | Bright White        | The Lightest Foreground (Not often used)                                                                    |
| `base08` | Red                 | Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted                                           |
| `base09` | (Orange)            | Integers, Boolean, Constants, XML Attributes, Markup Link Url                                               |
| `base0A` | Yellow              | Classes, Markup Bold, Search Text Background                                                                |
| `base0B` | Green               | Strings, Inherited Class, Markup Code, Diff Inserted                                                        |
| `base0C` | Cyan                | Support, Regular Expressions, Escape Characters, Markup Quotes                                              |
| `base0D` | Blue                | Functions, Methods, Attribute IDs, Headings                                                                 |
| `base0E` | Magenta             | Keywords, Storage, Selector, Markup Italic, Diff Changed                                                    |
| `base0F` | (Dark Red or Brown) | Deprecated Highlighting for Methods and Functions, Opening/Closing Embedded Language Tags, e.g., `<?php ?>` |
| `base10` | (Darker Black)      | Darker Background                                                                                           |
| `base11` | (Darkest Black)     | The Darkest Background                                                                                      |
| `base12` | Bright Red          | NA                                                                                                          |
| `base13` | Bright Yellow       | NA                                                                                                          |
| `base14` | Bright Green        | NA                                                                                                          |
| `base15` | Bright Cyan         | NA                                                                                                          |
| `base16` | Bright Blue         | NA                                                                                                          |
| `base17` | Bright Magenta      | NA                                                                                                          |

Note: The colors `base00` through `base05` are typically neutral. The colors from `base08` and up are typically more colorful, and give the color scheme a distinctive "look".

Note: **Bright** colors can have a higher luminosity relative to its non-bright counterpart. Conventionally, the luminosity can be determined by looking at the `L` value in the `HSL` color space (for the best accuracy, [`OKHSL`/`OKHSV`](https://bottosson.github.io/misc/colorpicker) is recommended).
Bright colors can also have increased saturation for stronger emphasis, but this is not a hard requirement.

![Red and Bright Red Example](https://github.com/tinted-theming/base24/blob/main/assets/red-and-bright-red-example.png?raw=true)
![Red and Bright Red Grayscale Example](https://github.com/tinted-theming/base24/blob/main/assets/red-and-bright-red-grayscale-example.png?raw=true)

### Normal elements

Ordinary text uses foreground `base05` and background `base00`.
Choose these colors for _high_ legibility, as the user does most of the reading and writing with these colors.

Compositors and display managers:

- Use foreground `base00` and background `base01` or `base05` to label normal unfocused workspaces, clients, and tabs.
- Use `base01` or `base05` for the borders of those elements.

### Focus elements

These colors indicate where the user is currently interacting.

Text editors use foreground `base05` and background `base01` to indicate the current line.

Compositors and display managers:

- Use foreground `base00` and background `base0D` to label focused workspaces, clients, and tabs.
- Use `base05` for the borders of those elements.

### Inactive elements

`base03` is used to indicate that something is not active.

Text editors use this as text foreground for comments.
Ensure that it is legible against the normal background (`base00`).

Compositors and display managers:

- Use foreground `base05` and background `base01` to label inactive workspaces, clients, and tabs
- Use `base01` or `base03` for the borders of those elements.

### Category elements

`base08`, `base09`, `base0A`, `base0B`, `base0C`, `base0D` and `base0E`
are used to distinguish between different kinds of elements.

Text editors use these colors as text foregrounds for syntax highlighting.
Ensure that they are legible against the ordinary background (`base01`).

Compositors use these colors for borders and tabs on windows in a tabbed arrangement.

Analysis apps (e.g., system monitors) use these colors in plots and charts to represent different variables.

### Warning elements

`base0F` is used for elements that provide a warning.

Text editors use this as text foreground for warnings.
Ensure that it is legible against the normal background (`base00`).

### Alert elements

These colors indicate errors, alerts, and anything urgent.

Text editors use `base08` as text foreground for errors.
Ensure that it is legible against the normal background (`base00`).

Compositors and display managers:

- Use foreground `base00` and background `base08` to label urgent workspaces, clients, and tabs.
- Use `base08` for the borders of those elements.

### Menu elements

Normal menu options use `base04` as text foreground and `base00` for the background.

### Selected elements

Currently selected menu options use `base05` or `base06` as text foreground and `base02` for the background.

### Matching elements

In applications where the user can search for text, `base06` is used as foreground for the matching strings. Some menus will filter the selections as the user begins typing.

- Use `base06`, `base0D` or `base0E` as foreground for the matching characters in unselected options.
- Use `base0D` as foreground for the matching characters in selected options.
