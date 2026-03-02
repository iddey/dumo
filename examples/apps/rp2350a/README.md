## Prerequisites

For running the examples, make sure you have your environment set up for `RP235x` development. The
compiler from a `rustup` installation works.

- `stable` channel Rust with `thumbv8m.main-none-eabihf` added as a target for builds.
- `flip-link` is configured as the linker in `.cargo/config.toml`.
- `picotool 2.2.0` or newer as the runner for uploading.

## Examples

Binaries are uploaded via USB in bootloader mode. Usage: `cargo run --release --bin <example>`

- `ratatui-tabs` - Demonstrates the use of the [`Tabs`] widget in an example which is an adaptation
  of the Ratatui Tabs example: <https://ratatui.rs/examples/widgets/tabs>

  Since the 0.96″ TFT-LCD display has a native resolution of 80×160 pixels, but it has 16-bit color
  support, this example uses [`FONT_6X16_4_BITS`] to get 5 rows and 26 columns of anti-aliased text
  on screen at the same time.

  *Waveshare™ RP2350-LCD-0.96*

  <img src="../assets/ratatui-tabs.gif" alt="ratatui-tabs" width="480" height="480" />

[`Tabs`]: https://docs.rs/ratatui/latest/ratatui/widgets/struct.Tabs.html
[`FONT_6X16_4_BITS`]: https://docs.rs/dumo/latest/dumo/fonts/constant.FONT_6X16_4_BITS.html

## Minimum supported Rust version

The minimum supported Rust version for `mplusfonts-examples-rp2350a` is `1.89`.

## License

The source code of `mplusfonts-examples-rp2350a` is dual-licensed under:

* Apache License, Version 2.0 ([LICENSE-APACHE] or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT License ([LICENSE-MIT] or <http://opensource.org/licenses/MIT>)

at your option.

[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
