## Prerequisites

For running the examples, make sure you have your environment set up for `ESP32-S3` development.
The compiler from an `espup` installation works.

- `esp` toolchain, which includes Rust with Xtensa support, with `esp32s3` enabled as a target.
- `espflash` tool, which is configured as the runner in `.cargo/config.toml` for uploading.

## Examples

Binaries are uploaded via USB in bootloader mode. Usage: `cargo run --release --bin <example>`

- `ratatui-logo` - Resembles the [`RatatuiLogo`] from the v0.30.0 “Bryndza” release header, leaving
  a trail of styled blocks along the entire height of the screen. It’s a basic demonstration of the
  ESP32-S3 with an ST7789 display controller, showing how to gain access to a few of their features
  using the [`esp-hal`] and [`mipidsi`] crates.

  There’s no hardware support to avoid the tearing effect with this module and the display assembly
  that it uses.

  *LILYGO® T-Display S3*

  <video src="../assets/ratatui-logo.mp4" type="video/mp4" width="480" height="480" controls />

[`RatatuiLogo`]: https://docs.rs/ratatui/latest/ratatui/widgets/struct.RatatuiLogo.html
[`esp-hal`]: https://crates.io/crates/esp-hal/1.0.0
[`mipidsi`]: https://crates.io/crates/mipidsi/0.10.0

## Minimum supported Rust version

The minimum supported Rust version for `dumo-examples-esp32s3` is `1.92`.

## License

The source code of `dumo-examples-esp32s3` is dual-licensed under:

* Apache License, Version 2.0 ([LICENSE-APACHE] or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT License ([LICENSE-MIT] or <http://opensource.org/licenses/MIT>)

at your option.

[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
