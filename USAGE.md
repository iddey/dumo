## Step 1 - Choosing a bitmap font

References to bitmap fonts for `dumo` backends can be obtained either through the `FONT` constants
found in the `fonts` module or by invoking a `font` macro found in the `dumo` crate root — this is
also what `FONT` constant declarations do, but using predefined sets of Unicode code points, while
the arguments to a `font` macro can be more granular in terms of what characters to include in the
output of the procedural macro that generates the pixel information.

### Option A - Using crate features

The `dumo` crate has three groups of `font-*` features, where enabling at least one feature in the
first group for the font size, and one in the second for the bit depth, includes a `FONT` constant
declaration in the build process and invokes the procedural macro that defines its constant value.

Whether then, in the binary output of the application using `dumo`, the generated data structure is
included, depends on the constant being referenced or not; it is still advised to only enable those
 `BitmapFont` instances that are used in order to save on build resources as the crate is rebuilt.

|                   | `font-1-bit`            | `font-2-bits` | `font-4-bits` | `font-8-bits` |
| ----------------- | ----------------------- | ------------- | ------------- | ------------- |
| `font-6x16`       | `FONT_6X16_1_BIT`       | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-6x18`       | `FONT_6X18_1_BIT`       | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-8x20`       | `FONT_8X20_1_BIT`       | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-8x24`       | `FONT_8X24_1_BIT`       | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-8x24-bold`  | `FONT_8X24_BOLD_1_BIT`  | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-10x30`      | `FONT_10X30_1_BIT`      | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-12x30`      | `FONT_12X30_1_BIT`      | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-12x36`      | `FONT_12X36_1_BIT`      | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-12x36-bold` | `FONT_12X36_BOLD_1_BIT` | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-14x42`      | `FONT_14X42_1_BIT`      | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-16x40`      | `FONT_16X40_1_BIT`      | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |
| `font-16x48`      | `FONT_16X48_1_BIT`      | `⁓_2_BITS`    | `⁓_4_BITS`    | `⁓_8_BITS`    |

Every `FONT` constant is populated with the subsets of glyphs that are enabled in the third group.

|                                 | Third `font-*` feature, to each `FONT` constant enabled…      |
| ------------------------------- | ------------------------------------------------------------- |
| `font-latin` *(default)*        | Adds 537 Unicode code points, including ASCII characters[^1]  |
| `font-tui-block` *(default)*    | Adds 74 Unicode code points, the generic block characters[^2] |
| `font-tui-boxes` *(default)*    | Adds 130 Unicode code points, the box-drawing characters[^3]  |
| `font-tui-dots-2x4` *(default)* | Adds 257[^4] Unicode code points, the braille patterns[^5]    |
| `font-tui-rect-2x3` *(default)* | Adds 64 Unicode code points, the block sextants[^6]           |
| `font-tui-rect-2x4` *(default)* | Adds 256 Unicode code points, the block octants[^7]           |
| `font-hiragana`                 | Adds 113 Unicode code points, the hiragana letters[^8]        |
| `font-katakana`                 | Adds 120 Unicode code points, the katakana letters[^9]        |
| `font-kanji`                    | Adds 5 553 Unicode code points, the kanji characters[^10]     |

Adding grapheme clusters for which no single Unicode code points exist, such as _g̈_ and _y̆_, is not
supported using the `fonts` module.

[^1]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/latin.set>
[^2]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/tui-block.set>
[^3]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/tui-boxes.set>
[^4]: The set of braille patterns begin with a blank character that is not classified as whitespace
[^5]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/tui-dots-2x4.set>
[^6]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/tui-rect-2x3.set>
[^7]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/tui-rect-2x4.set>
[^8]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/hiragana.set>
[^9]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/katakana.set>
[^10]: <https://raw.githubusercontent.com/iddey/dumo/refs/heads/main/glyph-subsets/kanji.set>

### Option B - Invoking a macro

These are the same presets for the font size/width and weight as above, but the bit depth parameter
is an integer, it is the first argument, while the rest of the arguments determine which glyphs and
glyph clusters will be included in the `BitmapFont` instance.

|                                  | Every `font_*` macro is equivalent to the macro invocation…  |
| -------------------------------- | ------------------------------------------------------------ |
| `font_6x16!($($args:tt)*)`       | `mpluscode!(115, 482, 16.125, true, $($args)*)`              |
| `font_6x18!($($args:tt)*)`       | `mpluscode!(100, 456, 18.06, true, $($args)*)`               |
| `font_8x20!($($args:tt)*)`       | `mpluscode!(125, 444, 20.066667, true, $($args)*)`           |
| `font_8x24!($($args:tt)*)`       | `mpluscode!(100, 418, 24.08, true, $($args)*)`               |
| `font_8x24_bold!($($args:tt)*)`  | `mpluscode!(100, 482, 24.08, true, $($args)*)`               |
| `font_10x30!($($args:tt)*)`      | `mpluscode!(100, 454, 30.1, true, $($args)*)`                |
| `font_12x30!($($args:tt)*)`      | `mpluscode!(125, 466, 30.1, true, $($args)*)`                |
| `font_12x36!($($args:tt)*)`      | `mpluscode!(100, 412, 36.12, true, $($args)*)`               |
| `font_12x36_bold!($($args:tt)*)` | `mpluscode!(100, 482, 36.12, true, $($args)*)`               |
| `font_14x42!($($args:tt)*)`      | `mpluscode!(100, 490, 42.14, true, $($args)*)`               |
| `font_16x40!($($args:tt)*)`      | `mpluscode!(125, 500, 40.133333, true, $($args)*)`           |
| `font_16x48!($($args:tt)*)`      | `mpluscode!(100, 439, 48.16, true, $($args)*)`               |

#### Call chain of a macro

Below is an explanation of the parameters found in the definition of the `mplusfonts!` macro, which
is the one that every `font` macro calls.

```rust
macro_rules! mpluscode {
    (
        $width:tt,   // Font width. Ranges from `100` to `125`. `100` is `NORMAL`.
        $weight:tt,  // Font weight. Ranges from `100` to `700`. `400` is `NORMAL`.
        $height:tt,  // Line height in pixels. Rounded to an integer by the text shaper.
        $hint:tt,    // Set to `true` to enable font hinting and adjust text shapes to a pixel grid.
        $($rest:tt)* // Bit depth of glyph images, followed by zero or more sources of textual data.
    ) => {
        ::mplusfonts::mplus!(code($width), $weight, code_line_height($height), $hint, 1, $($rest)*)
    }
}
```

In the end, it’s an [`mplus!`](https://docs.rs/mplusfonts/latest/mplusfonts/macro.mplus.html) macro
invocation that’s setting up and executing font rasterization.

#### Example of a bitmap font

The bitmap font definition from the `cursor-1` example is shown here.

```rust
pub fn draw() {
    // Can render text with numbers, letters, '#', '"', '(', ')', and box-drawing characters.
    let bitmap_font = dumo::font_8x20!(4, '0'..='9', 'A'..='Z', 'a'..='z', [r#"#"()"#], '─'..='╋');

    // ...
}
```

The ranges `'0'..='9'`, `'A'..='Z'`, … , are to be all of the characters which are expected during
text rendering, while arrays are for specifying individual characters as well as grapheme clusters
in string literals. The distribution of string literals between arrays and the number of character
occurrences make no difference, the resulting `BitmapFont` instance will be identical in any case.

There is a special case, when text to be shown on the display is known in advance at compile time,
and the block of code that contains the bitmap font definition also contains string literals to be
used by the application and would need to be added to the bitmap font, then it is possible to take
advantage of the [`strings`](https://docs.rs/mplusfonts/latest/mplusfonts/attr.strings.html) macro
and its `strings::skip` and `strings::emit` helper attributes.

```toml
[dependencies]
mplusfonts = "0.3.4"
```

```rust
#[mplusfonts::strings]
pub fn run() {
    // Can render text with numbers, letters, '#', '"', '(', ')', box-drawing characters, and "福岡市"
    // because the marked macro invocation is rewritten with literals found in the marked item body
    // added as an extra argument, which is what expansion of the `#[strings]` macro does.
    #[strings::emit]
    let bitmap_font = dumo::font_8x20!(4, '0'..='9', 'A'..='Z', 'a'..='z', [r#"#"()"#], '─'..='╋');

    // ... scanning for literals, such the argument to this function call, adding '福', '岡', and '市'.
    foo("福岡市");

    // Drawing this literal using `bitmap_font` would result in the characters "��市" being rendered
    // since the marked token tree was skipped.
    #[strings::skip]
    let bar = "札幌市";

    // ...
}

fn foo(text: &str) {
    // ...
}
```

#### All-inclusive bitmap font

The expression `dumo::font_8x20!(4, ..)` is valid, and although the resulting `BitmapFont` instance
will have a full set of glyph images for all Unicode code points available, grapheme clusters would
only have combining characters if they were added in string literals the same way they would appear.

For projects that do define a bitmap font with an unbounded range of characters, it is recommended
to move it to a library crate that doesn’t need rebuilding as often as the application referencing
the struct expression.

## Step 2 - Setting up the backend

This code example demonstrates [Option A](#option-a---using-crate-features).

```rust
use dumo::DumoBackend;
use dumo::fonts::*;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;

#[cfg(all(feature = "font-8x24", feature = "font-8x24-bold", feature = "font-4-bits"))]
pub fn run(target: &mut impl DrawTarget<Color = Rgb565, Error: core::fmt::Debug>) {
    // Create and configure a backend with regular and bold fonts
    let mut backend = DumoBackend::new(target, &FONT_8X24_4_BITS);
    backend.font_bold = Some(&FONT_8X24_BOLD_4_BITS);

    // Set up a callback for `.flush()` as needed
    let backend = backend.with_flush(|target| {
        // ...

        Ok(())
    });

    // ...
}
```

## Step 3 - Passing it all to the terminal

This code example demonstrates [Option B](#option-b---invoking-a-macro) and the `backend` in action.

```rust
use dumo::blink::Blink;
use dumo::cursor::Cursor;
use dumo::error::Error;
use dumo::{ConfigureBackend, DumoBackend};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use ratatui::{Frame, Terminal};

#[cfg(feature = "alloc")]
pub fn run<D>(target: &mut D) -> Result<(), Error<D::Error>>
where
    D: DrawTarget<Color = Rgb565, Error: core::fmt::Debug>
{
    let bitmap_font = dumo::font_8x24!(4, /* ... */);
    let bitmap_font_bold = dumo::font_8x24_bold!(4, /* ... */);

    // Create a backend with a regular font and also add three wrappers,
    // which can be done in any order, their effect will be identical
    let mut backend = DumoBackend::new(target, &bitmap_font)
        // Set up `.slow_blink()` and `.rapid_blink()` style modifiers
        .with_blink(Blink::with_period(16), Blink::with_period(8))
        // Set up `.show_cursor()` or `.set_cursor_position()`
        .with_cursor(Cursor::default())
        // Set up a callback for `.flush()` as needed
        .with_flush(|target| {
            // ...

            Ok(())
        });

    // Continue configuring backend with wrappers already added
    backend.set_font_bold(Some(&bitmap_font_bold));

    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame: &mut Frame| {
            // ...
        })?;
    }

    // ...

    Ok(())
}
```
