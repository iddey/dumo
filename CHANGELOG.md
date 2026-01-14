# Changelog

## [Unreleased]

### Added

- The built-in `WIN_16` and `WEB_256` palettes, which offer more familiar, basic color definitions.
- Trait for configuring the backend, even one that has wrappers, borrowing it back from a terminal.
- First two examples that use `embedded-graphics-simulator`: clock animation and built-in palettes.

### Fixed

- Font graphics being drawn to a different number of cells than expected by Ratatui. The graphics
  data remain unchanged and are managed through cropping and padding as needed.

### Changed

- The `Error` associated with the draw target for the backend no longer has to implement `Display`.

## [0.1.0-beta.0] - 2025-12-20

### Added

- Initial implementation of the backend interface from `ratatui-core 0.1.0-beta.0`.
- Support for draw targets from `embedded-graphics 0.8.1` with an optional flush callback.
- Support for text renderers from `mplusfonts 0.3.2` with its parametrized bitmap font.
- Configurable 16 and 256-color lookup tables, with `xterm`’s color palette used by default.

[0.1.0-beta.0]: https://github.com/iddey/dumo/releases/tag/v0.1.0-beta.0
