# Changelog

## [Unreleased]

### Fixed

- Underline and strikethrough decorations being cut off before the edge of the cell when the style
  of the text is set to bold and the set of glyph images don’t close the gap to the next cell.

### Changed

- Dim and hidden modifiers now have the effect of rendering pixels at 50% and 0%, respectively, of
  their values. Extended the drawing capabilities of backends to this effect.

## [0.1.0-beta.1] - 2026-01-15

### Added

- The built-in `WIN_16` and `WEB_256` palettes, which offer more familiar, basic color definitions.
- Trait for configuring the backend, even one that has wrappers, borrowing it back from a terminal.
- First two examples that use `embedded-graphics-simulator`: clock animation and built-in palettes.

### Fixed

- Font graphics being drawn to a different number of cells than expected by Ratatui. The graphics
  data remain unchanged and are managed through cropping and padding as needed.

### Changed

- The `Error` associated with the draw target for the backend no longer has to implement `Display`.
- Upgrade dependencies: `ratatui-core 0.1.0`.
- Upgrade dev-dependencies: `ratatui 0.30.0`.

## [0.1.0-beta.0] - 2025-12-20

### Added

- Initial implementation of the backend interface from `ratatui-core 0.1.0-beta.0`.
- Support for draw targets from `embedded-graphics 0.8.1` with an optional flush callback.
- Support for text renderers from `mplusfonts 0.3.2` with its parametrized bitmap font.
- Configurable 16 and 256-color lookup tables, with `xterm`’s color palette used by default.

[0.1.0-beta.0]: https://github.com/iddey/dumo/releases/tag/v0.1.0-beta.0
[0.1.0-beta.1]: https://github.com/iddey/dumo/releases/tag/v0.1.0-beta.1
