// hacker theme (aligned with shalloran/rss-tui)

use ratatui::style::Color;

#[derive(Clone, Copy, Debug, Default)]
pub struct Theme;

impl Theme {
    pub fn background_color(&self) -> Color {
        Color::Black
    }

    pub fn text_color(&self) -> Color {
        Color::Rgb(0, 255, 0) // bright green
    }

    pub fn title_color(&self) -> Color {
        Color::Rgb(0, 255, 255) // bright cyan
    }

    pub fn border_color(&self) -> Color {
        Color::Rgb(0, 200, 0) // medium green
    }

    pub fn highlight_color(&self) -> Color {
        Color::Rgb(0, 255, 255) // bright cyan
    }

    pub fn error_color(&self) -> Color {
        Color::Rgb(255, 0, 0) // bright red
    }

    /// command bar: black text on green for contrast
    pub fn command_bar_text_color(&self) -> Color {
        Color::Black
    }

    pub fn flash_color(&self) -> Color {
        Color::Rgb(0, 255, 0) // bright green
    }
}
