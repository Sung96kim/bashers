use ratatui::style::Color;

const TUI_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Green,
    Color::Magenta,
    Color::Yellow,
    Color::Blue,
    Color::LightCyan,
    Color::LightGreen,
    Color::LightMagenta,
];

const TITLE_COLORS: &[Color] = &[
    Color::Rgb(0x00, 0xee, 0xff),
    Color::Rgb(0x00, 0xff, 0x88),
    Color::Rgb(0xff, 0x66, 0xff),
    Color::Rgb(0xff, 0xee, 0x00),
    Color::Rgb(0x44, 0xaa, 0xff),
    Color::Rgb(0x66, 0xff, 0xff),
    Color::Rgb(0x66, 0xff, 0x99),
    Color::Rgb(0xff, 0x99, 0xff),
];

#[derive(Clone)]
pub struct Theme {
    pub pane_colors: &'static [Color],
    pub title_colors: &'static [Color],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            pane_colors: TUI_COLORS,
            title_colors: TITLE_COLORS,
        }
    }
}

impl Theme {
    pub fn pane_color(&self, index: usize) -> Color {
        self.pane_colors[index % self.pane_colors.len()]
    }

    pub fn title_color(&self, index: usize) -> Color {
        self.title_colors[index % self.title_colors.len()]
    }
}
