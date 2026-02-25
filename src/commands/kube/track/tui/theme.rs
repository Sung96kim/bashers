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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_has_colors() {
        let theme = Theme::default();
        assert!(!theme.pane_colors.is_empty());
        assert!(!theme.title_colors.is_empty());
    }

    #[test]
    fn test_pane_color_wraps_around() {
        let theme = Theme::default();
        let len = theme.pane_colors.len();
        assert_eq!(theme.pane_color(0), theme.pane_color(len));
        assert_eq!(theme.pane_color(1), theme.pane_color(len + 1));
    }

    #[test]
    fn test_title_color_wraps_around() {
        let theme = Theme::default();
        let len = theme.title_colors.len();
        assert_eq!(theme.title_color(0), theme.title_color(len));
        assert_eq!(theme.title_color(1), theme.title_color(len + 1));
    }

    #[test]
    fn test_pane_color_sequential_are_distinct() {
        let theme = Theme::default();
        for i in 0..theme.pane_colors.len() - 1 {
            assert_ne!(theme.pane_color(i), theme.pane_color(i + 1));
        }
    }

    #[test]
    fn test_title_color_sequential_are_distinct() {
        let theme = Theme::default();
        for i in 0..theme.title_colors.len() - 1 {
            assert_ne!(theme.title_color(i), theme.title_color(i + 1));
        }
    }

    #[test]
    fn test_theme_clone() {
        let theme = Theme::default();
        let cloned = theme.clone();
        assert_eq!(cloned.pane_colors.len(), theme.pane_colors.len());
        assert_eq!(cloned.title_colors.len(), theme.title_colors.len());
    }
}
