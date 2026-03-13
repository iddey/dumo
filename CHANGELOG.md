# Changelog

## [Unreleased]

### Added

- Two examples that run on an `rp2350a`: `ratatui` tabs and `ratatui` calendar with `tui-big-text`.
- An introduction to the `dumo` crate and its font features.

### Fixed

- Enabling `alloc` no longer enables the `defmt` feature; however, if `defmt` is enabled, then it
  does enable the `defmt/alloc` feature.

## [0.1.0-beta.4] - 2026-02-21

### Added

- Fifth example that uses `embedded-graphics-simulator`: cursor example #2, demonstrating glyphs.
- First example that runs on a development board and uses `esp-hal`: The `ratatui 0.30.0` banner.

### Fixed

- The area of the cell to the left of the glyph image, when the glyph for a double-width character
  is missing, and the _x_-axis anchor point is not set to `Left`, not being cleared (as it used to
  be prior to `0.1.0-beta.3`).

### Changed

- Calling `advance_cursor_blink_to` when the cursor is solid on. Since this means that the cursor
  has _blinked_ indefinitely, advancing the cursor blink to the _blinked_ state now returns `Ok`.
- Upgrade dependencies: `embedded-graphics 0.8.2` and `mplusfonts 0.3.3`.
- Upgrade dev-dependencies: `tui-big-text 0.8.2`.

## [0.1.0-beta.3] - 2026-02-15

### Added

- Trait for configuring the slow and rapid blinking animation cycles.
- Support for a terminal-style cursor position indicator with a configurable appearance. Requires
  the `alloc` feature to remain enabled. The cursor wrapper is added by calling the `with_cursor`
  method, which can be done before or after any other wrappers are added.
- Fourth example that uses `embedded-graphics-simulator`: cursor settings, rendering dynamically.
- Bitmap font configuration through features; this is for convenience, not having to parametrize
  macro invocations with character ranges and strings to be added, however, it results in images
  of glyphs being generated and included, even though these may never be used in any text that’s
  rendered during runtime.

### Fixed

- Underline and strikethrough decorations, when the glyph for a double-width character is missing
  in the bitmap font used, not being extended to the entire width of the cells.

### Changed

- Missing implementations of the `clear` and `clear_region` methods; these are now functioning in
  accordance with the semantics described in the clearing API contract.

## [0.1.0-beta.2] - 2026-01-31

### Added

- Support for terminal-style blinking animation using modifiers. Requires the new `alloc` feature
  to remain enabled; otherwise, the wrapper that does the animation is not available. The wrapper
  is configured using the `with_blink` method, and it can be called before or after `with_flush`.
- Third example that uses `embedded-graphics-simulator`: style modifiers, demonstrating blinking.

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
[0.1.0-beta.2]: https://github.com/iddey/dumo/releases/tag/v0.1.0-beta.2
[0.1.0-beta.3]: https://github.com/iddey/dumo/releases/tag/v0.1.0-beta.3
[0.1.0-beta.4]: https://github.com/iddey/dumo/releases/tag/v0.1.0-beta.4
