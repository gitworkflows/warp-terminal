//! Iced GUI framework integration for Warp themes

use crate::{Theme, Color};
use iced::{border, Shadow, Vector};

/// Convert Warp theme to Iced theme
impl From<&Theme> for iced::theme::Palette {
    fn from(theme: &Theme) -> Self {
        let background = warp_color_to_iced(theme.background);
        let text = warp_color_to_iced(theme.foreground);
        let primary = warp_color_to_iced(theme.accent);
        
        // Determine success/danger colors based on terminal colors
        let success = warp_color_to_iced(theme.terminal_colors.normal.green);
        let danger = warp_color_to_iced(theme.terminal_colors.normal.red);
        
        iced::theme::Palette {
            background,
            text,
            primary,
            success,
            danger,
        }
    }
}

/// Convert Warp Color to Iced Color
pub fn warp_color_to_iced(color: Color) -> iced::Color {
    iced::Color::from_rgba8(color.r, color.g, color.b, color.a as f32 / 255.0)
}

/// Convert Iced Color to Warp Color
pub fn iced_color_to_warp(color: iced::Color) -> Color {
    let rgba = color.into_linear();
    Color::new(
        (rgba[0] * 255.0) as u8,
        (rgba[1] * 255.0) as u8,
        (rgba[2] * 255.0) as u8,
        (rgba[3] * 255.0) as u8,
    )
}

/// Theme styles for Iced components
#[derive(Debug, Clone)]
pub struct WarpThemeStyle {
    pub theme: Theme,
}

impl WarpThemeStyle {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Get background color
    pub fn background(&self) -> iced::Color {
        warp_color_to_iced(self.theme.background)
    }

    /// Get foreground/text color
    pub fn foreground(&self) -> iced::Color {
        warp_color_to_iced(self.theme.foreground)
    }

    /// Get accent color
    pub fn accent(&self) -> iced::Color {
        warp_color_to_iced(self.theme.accent)
    }

    /// Get cursor color
    pub fn cursor(&self) -> iced::Color {
        warp_color_to_iced(self.theme.cursor.unwrap_or(self.theme.accent))
    }

    /// Get terminal color by index
    pub fn terminal_color(&self, index: u8, bright: bool) -> iced::Color {
        let palette = if bright {
            &self.theme.terminal_colors.bright
        } else {
            &self.theme.terminal_colors.normal
        };
        
        let color = palette.get_by_index(index).unwrap_or(self.theme.foreground);
        warp_color_to_iced(color)
    }
}

/// Custom button style using Warp theme
pub struct WarpButtonStyle {
    pub theme_style: WarpThemeStyle,
    pub variant: ButtonVariant,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
}

impl iced::widget::button::StyleSheet for WarpButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let base_color = match self.variant {
            ButtonVariant::Primary => self.theme_style.accent(),
            ButtonVariant::Secondary => self.theme_style.foreground(),
            ButtonVariant::Success => self.theme_style.terminal_color(2, false), // Green
            ButtonVariant::Danger => self.theme_style.terminal_color(1, false),  // Red
            ButtonVariant::Warning => self.theme_style.terminal_color(3, false), // Yellow
        };

        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(base_color)),
            text_color: if self.theme_style.theme.is_dark() {
                iced::Color::WHITE
            } else {
                iced::Color::BLACK
            },
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: base_color,
            },
            shadow: Shadow::default(),
            shadow_offset: Vector::new(0.0, 1.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        iced::widget::button::Appearance {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        iced::widget::button::Appearance {
            shadow_offset: Vector::new(0.0, 0.0),
            ..active
        }
    }
}

/// Custom text input style using Warp theme
pub struct WarpTextInputStyle {
    pub theme_style: WarpThemeStyle,
}

impl iced::widget::text_input::StyleSheet for WarpTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(self.theme_style.background()),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: self.theme_style.terminal_color(0, true), // Bright black for border
            },
            icon_color: self.theme_style.foreground(),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(self.theme_style.background()),
            border: border::Border {
                radius: 8.0.into(),
                width: 2.0,
                color: self.theme_style.accent(),
            },
            icon_color: self.theme_style.foreground(),
            // cursor color is not customizable in this version of iced
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.focused(style)
    }

    fn placeholder_color(&self, _style: &Self::Style) -> iced::Color {
        let mut color = self.theme_style.foreground();
        color.a *= 0.6; // Make placeholder semi-transparent
        color
    }

    fn value_color(&self, _style: &Self::Style) -> iced::Color {
        self.theme_style.foreground()
    }

    fn selection_color(&self, _style: &Self::Style) -> iced::Color {
        let mut accent = self.theme_style.accent();
        accent.a *= 0.3; // Semi-transparent selection
        accent
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        let active = self.active(style);
        iced::widget::text_input::Appearance {
            background: iced::Background::Color({
                let mut bg = self.theme_style.background();
                bg.a *= 0.6; // Make background more transparent when disabled
                bg
            }),
            ..active
        }
    }

    fn disabled_color(&self, _style: &Self::Style) -> iced::Color {
        let mut color = self.theme_style.foreground();
        color.a *= 0.4; // Make disabled text more transparent
        color
    }
}

/// Custom container style using Warp theme
pub struct WarpContainerStyle {
    pub theme_style: WarpThemeStyle,
}

impl iced::widget::container::StyleSheet for WarpContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(self.theme_style.background())),
            border: border::Border::default(),
            shadow: Shadow::default(),
            text_color: Some(self.theme_style.foreground()),
        }
    }
}

/// Custom rule style using Warp theme
pub struct WarpRuleStyle {
    pub theme_style: WarpThemeStyle,
}

impl iced::widget::rule::StyleSheet for WarpRuleStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::rule::Appearance {
        iced::widget::rule::Appearance {
            color: self.theme_style.foreground(),
            width: 1,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
        }
    }
}
