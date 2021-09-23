use clutter::Color;

pub struct Theme;

impl Theme {
    pub fn font_name() -> &'static str {
        "Fira Sans Semi-Light 10"
    }

    pub fn small_font_name() -> &'static str {
        "Fira Sans Semi-Light 9"
    }

    pub fn color_background() -> Color {
        Color::new(0x33, 0x30, 0x2F, 0xFF)
    }

    pub fn color_border() -> Color {
        Color::new(0xFB, 0xB8, 0x6C, 0xFF)
    }

    pub fn color_highlight() -> Color {
        Color::new(0x5A, 0x57, 0x57, 0xFF)
    }

    pub fn color_input() -> Color {
        Color::new(0x2B, 0x29, 0x28, 0xFF)
    }

    pub fn color_text() -> Color {
        Color::new(0xE6, 0xE6, 0xE6, 0xFF)
    }
}
