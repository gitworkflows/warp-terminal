use iced::Theme as IcedTheme;
use iced::Color;

#[derive(Debug, Clone)]
pub struct AppTheme {
    #[allow(dead_code)]
    name: String,
    background_color: Color,
    text_color: Color,
    accent_color: Color,
    cursor_color: Color,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            name: "Dark".to_string(),
            background_color: Color::from_rgb(0.07, 0.07, 0.07),
            text_color: Color::WHITE,
            accent_color: Color::from_rgb(0.2, 0.6, 1.0),
            cursor_color: Color::from_rgb(0.3, 0.7, 1.0),
        }
    }
}

impl From<AppTheme> for IcedTheme {
    fn from(_theme: AppTheme) -> Self {
        IcedTheme::Dark
    }
}

impl AppTheme {
    pub fn new_dark() -> Self {
        Self::default()
    }

    pub fn background_color(&self) -> iced::Color {
        self.background_color
    }

    pub fn text_color(&self) -> iced::Color {
        self.text_color
    }

    pub fn accent_color(&self) -> iced::Color {
        self.accent_color
    }

    pub fn cursor_color(&self) -> iced::Color {
        self.cursor_color
    }

    pub fn terminal_color(&self, index: u8, bright: bool) -> iced::Color {
        // Simple terminal color mapping
        match (index, bright) {
            (0, false) => Color::from_rgb(0.2, 0.2, 0.2), // Black
            (0, true) => Color::from_rgb(0.6, 0.6, 0.6),  // Bright Black
            (1, false) => Color::from_rgb(0.8, 0.3, 0.3), // Red
            (1, true) => Color::from_rgb(1.0, 0.4, 0.4),  // Bright Red
            (2, false) => Color::from_rgb(0.3, 0.8, 0.3), // Green
            (2, true) => Color::from_rgb(0.4, 1.0, 0.4),  // Bright Green
            (3, false) => Color::from_rgb(0.8, 0.8, 0.3), // Yellow
            (3, true) => Color::from_rgb(1.0, 1.0, 0.4),  // Bright Yellow
            (4, false) => Color::from_rgb(0.3, 0.3, 0.8), // Blue
            (4, true) => Color::from_rgb(0.4, 0.4, 1.0),  // Bright Blue
            (5, false) => Color::from_rgb(0.8, 0.3, 0.8), // Magenta
            (5, true) => Color::from_rgb(1.0, 0.4, 1.0),  // Bright Magenta
            (6, false) => Color::from_rgb(0.3, 0.8, 0.8), // Cyan
            (6, true) => Color::from_rgb(0.4, 1.0, 1.0),  // Bright Cyan
            (7, false) => Color::from_rgb(0.8, 0.8, 0.8), // White
            (7, true) => Color::WHITE,                     // Bright White
            _ => self.text_color,
        }
    }
}
