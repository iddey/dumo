use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use dumo::DumoBackend;
use dumo::error::Error;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::SimulatorEvent::Quit;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Position, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::StatefulWidget;
use ratatui::{Frame, Terminal};

struct DumoBanner;
struct DumoBannerState {
    alt: bool,
}

impl DumoBanner {
    const BG_COLOR: Color = Color::Rgb(0, 0, 0);
    const CRUST: Color = Color::Rgb(176, 128, 96);
    const CRUMB: Color = Color::Rgb(255, 255, 224);
    const BLANKET: Color = Color::Rgb(255, 255, 255);
    const CHECKER: Color = Color::Rgb(255, 0, 0);
    const DIFFUSE: Color = Color::Rgb(216, 216, 216);
    const AMBIENT: Color = Color::Rgb(164, 222, 255);
    const CONTENT: Color = Color::Rgb(185, 229, 255);
    const BASKET: Color = Color::Rgb(176, 160, 80);
    const STRIPE: Color = Color::Rgb(0, 155, 213);

    const TEXT_ROWS: [&str; 10] = [
        "                                                            ",
        "                                           ▂🮅🮅▂🮅🮅▂🮅🮅▂▂      ",
        "  𜵡𜴧𜴧𜴧𜴧𜴧𜶀𜺣                             𜷡█▘𜶖██𜷞𜴿█𜷞𜴿█𜷞𜴿███𜷞   ",
        "  𜶚 𜶖𜴆𜴆𜴆𜴧𜴷𜶄                            ██ ▐██████████████   ",
        "  ▐ ▐    ▌𜺫𜵈𜶖𜴆𜵈 𜶖𜴆𜵈𜶖𜴆𜶄𜴧𜴆𜶀𜴧𜴆𜶀 𜺠𜴐𜴆𜴆𜴜𜺣   𜵷𜴉𜺣▆▄▀🮅🮅🮅🮅🮅🮅🮅🮅▀▀🮂𜺣𜶻𜶠  ",
        "  ▐ ▐   ▂𜴍𜺠𜴍▐ ▌𜺠𜵛 ▌𜺫▌𜶖🮂𜵈𜶖🮂𜵈▐ ▌𜶖🮂🮂𜵈▐   𜵷𜴖𜵩𜷌𜵷𜴐𜷒𜴐𜴂 𜴅𜴀𜶋𜴜𜷆𜶠𜴤𜵀𜶽𜶠  ",
        "  𜵊 𜺫🮂🮂🮂𜺠𜵑𜴁 𜴡𜺣𜺫𜺨𜵑𜺣𜶘 ▌▐ ▌▐ ▌▐ 𜴝𜺣🮂🮂𜺠𜴒   𜵷𜴖𜵩𜵒𜴂𜴁 ▂𜶀𜵩𜷏𜶀𜺣 𜺫𜴜𜴤𜵀𜶽𜶠  ",
        "  🮂🮂🮂🮂🮂🮂𜺨    𜺫🮂🮂 🮂🮂 🮂🮂 🮂🮂 🮂🮂  𜺫🮂🮂𜺨    𜴂𜴁 𜺠𜵡𜷏𜶎𜵪𜷒𜶎𜵪𜷒𜶎𜵪𜷏𜶀𜺣𜺫𜺫𜶠  ",
        "██████████████████████████████████████𜷋𜷡𜷥𜷞▄▄▄▄▄▄▄▄▄▄▄▄𜷡𜷤𜷞𜶻██",
        "                                                            ",
    ];

    const STYLE_ROWS: [&str; 10] = [
        "000000000000000000000000000000000000000000000000000000000000",
        "000000000000000000000000000000000000000000012212212211000000",
        "006666666600000000000000000000000000000113111331331331111000",
        "006077777670000000000000000000000000000113111111111111111000",
        "006070000677667066766766766706666670008884599999999999988800",
        "006070006677607660766776776706777670008888888888888888888800",
        "00607666777067667770670670670676677000888888AAAAAAAA88888800",
        "00777777700007770770770770770077770000888AAAAAAAAAAAAAA88800",
        "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBCCCCCCCCCCCCCCBBBBB",
        "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
    ];

    const fn decode_style(code: char, alt: bool) -> Option<Style> {
        match code {
            '0' => Some(Style::new().bg(Self::BG_COLOR)),
            '1' => Some(Style::new().bg(Self::BG_COLOR).fg(Self::CRUST)),
            '2' => Some(Style::new().bg(Self::CRUMB).fg(Self::BG_COLOR)),
            '3' => Some(Style::new().bg(Self::CRUMB).fg(Self::CRUST)),
            '4' => Some(Style::new().bg(Self::CRUMB).fg(Self::BLANKET)),
            '5' => Some(Style::new().bg(Self::BG_COLOR).fg(Self::BLANKET)),
            '6' => Some(Style::new().bg(Self::BG_COLOR).fg(Self::DIFFUSE)),
            '7' if alt => Some(Style::new().bg(Self::BG_COLOR).fg(Self::AMBIENT)),
            '7' => Some(Style::new().bg(Self::BG_COLOR).fg(Self::CONTENT)),
            '8' => Some(Style::new().bg(Self::BLANKET).fg(Self::CHECKER)),
            '9' => Some(Style::new().bg(Self::BLANKET).fg(Self::CRUST)),
            'A' => Some(Style::new().bg(Self::BLANKET).fg(Self::BASKET)),
            'B' => Some(Style::new().bg(Self::BLANKET).fg(Self::STRIPE)),
            'C' => Some(Style::new().bg(Self::BASKET).fg(Self::STRIPE)),
            'D' => Some(Style::new().bg(Self::STRIPE)),
            _ => None,
        }
    }
}

impl StatefulWidget for DumoBanner {
    type State = DumoBannerState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let offset = Offset::new(area.x.into(), area.y.into());

        for ((text, y), styles) in Self::TEXT_ROWS.into_iter().zip(0..).zip(Self::STYLE_ROWS) {
            for ((char, x), code) in text.chars().zip(0..).zip(styles.chars()) {
                let position = Position::new(x, y).offset(offset);

                if let Some(cell) = buffer.cell_mut(position)
                    && let Some(style) = Self::decode_style(code, state.alt)
                {
                    cell.set_char(char).set_style(style);
                }
            }
        }
    }
}

/// Displays "Dumo" and its bread logo for the header of the title page in the first release.
#[mplusfonts::strings]
pub fn main() -> Result<(), Error<Infallible>> {
    const VERSION: &str = "0.1";
    const RELEASE: &str = "Liptauer";

    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(480, 222));

    #[strings::emit]
    let bitmap_font = dumo::font_8x24_bold!(4, '▀'..='▟', '𜴀'..='𜷥', [" 𜺨🮂𜺫🯦🯧𜺣𜺠🮅"]);

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    #[strings::skip]
    let mut window = Window::new("Simulator", &output_settings);

    let is_running = AtomicBool::new(true);

    let mut state = DumoBannerState { alt: false };

    let backend = DumoBackend::new(&mut display, &bitmap_font).with_flush(|display| {
        window.update(display);

        for event in window.events() {
            match event {
                Quit => {
                    is_running.store(false, Ordering::Relaxed);
                }
                _ => continue,
            }
        }

        Ok(())
    });

    let mut terminal = Terminal::new(backend)?;

    while is_running.load(Ordering::Relaxed) {
        let render = |frame: &mut Frame| {
            let area = frame.area();
            let version_area = Rect::new(area.x + 2, area.y + 8, area.width - 26, 1);

            frame.render_stateful_widget(DumoBanner, area, &mut state);
            frame.render_widget(format!("v{VERSION} “{RELEASE}”").reversed(), version_area);
        };

        terminal.draw(render)?;

        thread::sleep(Duration::from_millis(50));

        state.alt = !state.alt;
    }

    Ok(())
}
