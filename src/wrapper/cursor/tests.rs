use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::pixelcolor::{Gray8, Rgb888};
use mplusfonts::BitmapFont;

use crate::backend::DumoBackend;
use crate::blink::{Blink, Blinked, ControlCursorBlinking};
use crate::cursor::Cursor;
use crate::error::AdvanceCursorBlinkingError::InvalidBlinked;
use crate::error::Error;

macro_rules! test_advance_cursor_blink_to_blinked {
    (
        $(
            $fn_ident:ident, $cursor:expr, $blinked:expr, $expected:pat,
        )*
    ) => {
        $(
            #[test]
            fn $fn_ident() {
                let mut target: MockDisplay<Rgb888> = MockDisplay::new();
                let bitmap_font: BitmapFont<Gray8, 1> = BitmapFont::NULL;
                let backend = DumoBackend::new(&mut target, &bitmap_font);
                let mut backend = backend.with_cursor($cursor);
                let result = backend.advance_cursor_blink_to($blinked);
                assert!(matches!(result, $expected));
            }
        )*
    }
}

test_advance_cursor_blink_to_blinked! {
    advance_cursor_blink_with_delay_0_blink_0_to_blinked,
        Cursor::default().blink(Blink::Repeat(0, 0)),
        Blinked(true),
        Ok(()),

    advance_cursor_blink_with_delay_0_blink_0_to_not_blinked,
        Cursor::default().blink(Blink::Repeat(0, 0)),
        Blinked(false),
        Err(Error::AdvanceCursorBlinking(InvalidBlinked)),

    advance_cursor_blink_with_delay_0_blink_1_to_blinked,
        Cursor::default().blink(Blink::Repeat(0, 1)),
        Blinked(true),
        Ok(()),

    advance_cursor_blink_with_delay_0_blink_1_to_not_blinked,
        Cursor::default().blink(Blink::Repeat(0, 1)),
        Blinked(false),
        Err(Error::AdvanceCursorBlinking(InvalidBlinked)),

    advance_cursor_blink_with_delay_1_blink_0_to_blinked,
        Cursor::default().blink(Blink::Repeat(1, 0)),
        Blinked(true),
        Err(Error::AdvanceCursorBlinking(InvalidBlinked)),

    advance_cursor_blink_with_delay_1_blink_0_to_not_blinked,
        Cursor::default().blink(Blink::Repeat(1, 0)),
        Blinked(false),
        Ok(()),

    advance_cursor_blink_with_delay_1_blink_1_to_blinked,
        Cursor::default().blink(Blink::Repeat(1, 1)),
        Blinked(true),
        Ok(()),

    advance_cursor_blink_with_delay_1_blink_1_to_not_blinked,
        Cursor::default().blink(Blink::Repeat(1, 1)),
        Blinked(false),
        Ok(()),
}
