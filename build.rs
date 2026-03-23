use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").expect("expected `OUT_DIR` to be set");
    let src_path = env::current_dir()?.join("glyph-subsets");
    let dst_path = Path::new(&out_dir).join("const-items.rs");
    let dst_file = File::create(dst_path)?;
    let mut writer = BufWriter::new(dst_file);
    write_const_items(&src_path, &mut writer)?;
    writer.flush()?;

    Ok(())
}

fn write_const_items(src_path: &Path, writer: &mut BufWriter<File>) -> Result<()> {
    let font_idents = [
        cfg!(feature = "font-6x16").then_some("font_6x16"),
        cfg!(feature = "font-6x18").then_some("font_6x18"),
        cfg!(feature = "font-8x20").then_some("font_8x20"),
        cfg!(feature = "font-8x24").then_some("font_8x24"),
        cfg!(feature = "font-8x24-bold").then_some("font_8x24_bold"),
        cfg!(feature = "font-10x30").then_some("font_10x30"),
        cfg!(feature = "font-12x30").then_some("font_12x30"),
        cfg!(feature = "font-12x36").then_some("font_12x36"),
        cfg!(feature = "font-12x36-bold").then_some("font_12x36_bold"),
        cfg!(feature = "font-14x42").then_some("font_14x42"),
        cfg!(feature = "font-16x40").then_some("font_16x40"),
        cfg!(feature = "font-16x48").then_some("font_16x48"),
    ];

    let font_params = [
        cfg!(feature = "font-1-bit").then_some(("BinaryColor", 1, "bit")),
        cfg!(feature = "font-2-bits").then_some(("Gray2", 2, "bits")),
        cfg!(feature = "font-4-bits").then_some(("Gray4", 4, "bits")),
        cfg!(feature = "font-8-bits").then_some(("Gray8", 8, "bits")),
    ];

    let glyph_subsets = [
        cfg!(feature = "font-hiragana").then_some("hiragana.set"),
        cfg!(feature = "font-kanji").then_some("kanji.set"),
        cfg!(feature = "font-katakana").then_some("katakana.set"),
        cfg!(feature = "font-latin").then_some("latin.set"),
        cfg!(feature = "font-tui-block").then_some("tui-block.set"),
        cfg!(feature = "font-tui-boxes").then_some("tui-boxes.set"),
        cfg!(feature = "font-tui-dots-2x4").then_some("tui-dots-2x4.set"),
        cfg!(feature = "font-tui-rect-2x3").then_some("tui-rect-2x3.set"),
        cfg!(feature = "font-tui-rect-2x4").then_some("tui-rect-2x4.set"),
    ];

    for font_ident in font_idents.into_iter().flatten() {
        for (color_type, bit_depth, unit) in font_params.into_iter().flatten() {
            let item_ident = format!("{font_ident}_{bit_depth}_{unit}").to_uppercase();
            let item_type = format_args!(
                "::mplusfonts::BitmapFont<{color_type}, {positions}>",
                color_type = format_args!("::embedded_graphics::pixelcolor::{color_type}"),
                positions = 1,
            );

            writeln!(
                writer,
                "/// The [`{font_ident}`](crate::{font_ident}) bitmap font rendered at {bit_depth}",
            )?;

            writeln!(
                writer,
                "/// {unit} per pixel, and it was configured to include the sets of glyphs listed.",
            )?;

            if glyph_subsets.into_iter().flatten().next().is_some() {
                writeln!(writer, "///")?;
            } else {
                writeln!(writer, "/// No glyphs added; this is an empty bitmap font.")?;
            }

            for glyph_subset in glyph_subsets.into_iter().flatten() {
                writeln!(writer, "/// * `{glyph_subset}`")?;
            }

            writeln!(
                writer,
                "pub const {item_ident}: {item_type} = crate::{font_ident}!({bit_depth}, [",
            )?;

            for glyph_subset in glyph_subsets.into_iter().flatten() {
                let src_path = src_path.join(glyph_subset);
                let src_file = File::open(src_path)?;
                let reader = BufReader::new(src_file);
                for line in reader.lines() {
                    writeln!(writer, r##"r#"{line}"#,"##, line = line?)?;
                }
            }

            writeln!(writer, "]);")?;
        }
    }

    Ok(())
}
