use iced::widget::{button, container, text, text_input, scrollable, row, column, Space};
use iced::{theme, Alignment, Background, Border, Color, Element, Shadow, Vector};

/// Modern glass-morphism container style
#[derive(Debug, Clone)]
pub struct GlassMorphismContainer {
    pub blur_intensity: f32,
    pub opacity: f32,
    pub border_color: Color,
    pub shadow_color: Color,
}

impl Default for GlassMorphismContainer {
    fn default() -> Self {
        Self {
            blur_intensity: 10.0,
            opacity: 0.15,
            border_color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
            shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
        }
    }
}

impl container::StyleSheet for GlassMorphismContainer {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, self.opacity))),
            border: Border {
                color: self.border_color,
                width: 1.0,
                radius: 16.0.into(),
            },
            text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
            shadow: Shadow {
                color: self.shadow_color,
                offset: Vector::new(0.0, 8.0),
                blur_radius: 32.0,
            },
        }
    }
}

/// Modern elevated card style
#[derive(Debug, Clone)]
pub struct ModernCard {
    pub elevation: f32,
    pub hover_elevation: f32,
    pub background_color: Color,
    pub is_hovered: bool,
}

impl Default for ModernCard {
    fn default() -> Self {
        Self {
            elevation: 4.0,
            hover_elevation: 8.0,
            background_color: Color::from_rgba(0.08, 0.08, 0.08, 0.95),
            is_hovered: false,
        }
    }
}

impl container::StyleSheet for ModernCard {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let elevation = if self.is_hovered {
            self.hover_elevation
        } else {
            self.elevation
        };

        container::Appearance {
            background: Some(Background::Color(self.background_color)),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.4),
                width: 1.0,
                radius: 12.0.into(),
            },
            text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, elevation / 2.0),
                blur_radius: elevation * 2.0,
            },
        }
    }
}

/// Modern gradient button style
#[derive(Debug, Clone)]
pub struct GradientButton {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub text_color: Color,
    pub is_primary: bool,
}

impl Default for GradientButton {
    fn default() -> Self {
        Self {
            primary_color: Color::from_rgb(0.0, 0.5, 1.0),
            secondary_color: Color::from_rgb(0.0, 0.7, 0.9),
            text_color: Color::WHITE,
            is_primary: true,
        }
    }
}

impl button::StyleSheet for GradientButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        if self.is_primary {
            button::Appearance {
                background: Some(Background::Color(self.primary_color)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                text_color: self.text_color,
                shadow: Shadow {
                    color: Color::from_rgba(self.primary_color.r, self.primary_color.g, self.primary_color.b, 0.4),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                shadow_offset: Vector::new(0.0, 4.0),
            }
        } else {
            button::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.6))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.8),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                text_color: Color::from_rgb(0.9, 0.9, 0.9),
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                shadow_offset: Vector::new(0.0, 2.0),
            }
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        if self.is_primary {
            button::Appearance {
                background: Some(Background::Color(Color {
                    r: self.primary_color.r + 0.1,
                    g: self.primary_color.g + 0.1,
                    b: self.primary_color.b + 0.1,
                    a: self.primary_color.a,
                })),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                text_color: self.text_color,
                shadow: Shadow {
                    color: Color::from_rgba(self.primary_color.r, self.primary_color.g, self.primary_color.b, 0.6),
                    offset: Vector::new(0.0, 6.0),
                    blur_radius: 16.0,
                },
                shadow_offset: Vector::new(0.0, 6.0),
            }
        } else {
            button::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.8))),
                border: Border {
                    color: Color::from_rgba(0.5, 0.5, 0.5, 1.0),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                text_color: Color::WHITE,
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 8.0,
                },
                shadow_offset: Vector::new(0.0, 4.0),
            }
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color {
                r: self.primary_color.r - 0.1,
                g: self.primary_color.g - 0.1,
                b: self.primary_color.b - 0.1,
                a: self.primary_color.a,
            })),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: self.text_color,
            shadow: Shadow {
                color: Color::from_rgba(self.primary_color.r, self.primary_color.g, self.primary_color.b, 0.2),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            shadow_offset: Vector::new(0.0, 2.0),
        }
    }
}

/// Modern text input with enhanced styling
#[derive(Debug, Clone)]
pub struct ModernTextInput {
    pub focused: bool,
    pub accent_color: Color,
}

impl Default for ModernTextInput {
    fn default() -> Self {
        Self {
            focused: false,
            accent_color: Color::from_rgb(0.0, 0.5, 1.0),
        }
    }
}

impl text_input::StyleSheet for ModernTextInput {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8)),
            border: Border {
                color: if self.focused {
                    self.accent_color
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 0.6)
                },
                width: if self.focused { 2.0 } else { 1.0 },
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.12, 0.12, 0.12, 0.9)),
            border: Border {
                color: self.accent_color,
                width: 2.0,
                radius: 8.0.into(),
            },
            icon_color: self.accent_color,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.6, 0.6, 0.6, 0.8)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.95, 0.95, 0.95)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.4, 0.4, 0.4, 0.6)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(self.accent_color.r, self.accent_color.g, self.accent_color.b, 0.4)
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.05, 0.05, 0.05, 0.5)),
            border: Border {
                color: Color::from_rgba(0.2, 0.2, 0.2, 0.3),
                width: 1.0,
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgba(0.4, 0.4, 0.4, 0.6),
        }
    }
}

/// Enhanced scrollable with modern scrollbar
#[derive(Debug, Clone)]
pub struct ModernScrollable;

impl scrollable::StyleSheet for ModernScrollable {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: container::Appearance::default(),
            scrollbar: scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.3))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                scroller: scrollable::Scroller {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.8),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, _style: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Appearance {
        scrollable::Appearance {
            container: container::Appearance::default(),
            scrollbar: if is_mouse_over_scrollbar {
                scrollable::Scrollbar {
                    background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.5))),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    scroller: scrollable::Scroller {
                        color: Color::from_rgba(0.6, 0.6, 0.6, 1.0),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                    },
                }
            } else {
                self.active(_style).scrollbar
            },
            gap: None,
        }
    }

    fn dragging(&self, _style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: container::Appearance::default(),
            scrollbar: scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.7))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                scroller: scrollable::Scroller {
                    color: Color::from_rgba(0.8, 0.8, 0.8, 1.0),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                },
            },
            gap: None,
        }
    }
}

/// Utility functions for creating modern UI components
pub struct ModernUI;

impl ModernUI {
    /// Create a modern glass-morphism container
    pub fn glass_container<'a, Message>(
        content: Element<'a, Message>,
    ) -> container::Container<'a, Message> {
        container(content)
            .style(theme::Container::Custom(Box::new(GlassMorphismContainer::default())))
            .padding(20)
    }

    /// Create a modern elevated card
    pub fn card<'a, Message>(content: Element<'a, Message>) -> container::Container<'a, Message> {
        container(content)
            .style(theme::Container::Custom(Box::new(ModernCard::default())))
            .padding(16)
    }

    /// Create a modern primary button
    pub fn primary_button<'a, Message: Clone>(
        label: &str,
        on_press: Option<Message>,
    ) -> button::Button<'a, Message> {
        let mut btn = button(text(label).size(14))
            .style(theme::Button::Custom(Box::new(GradientButton {
                is_primary: true,
                ..GradientButton::default()
            })))
            .padding([12, 20]);

        if let Some(msg) = on_press {
            btn = btn.on_press(msg);
        }

        btn
    }

    /// Create a modern secondary button
    pub fn secondary_button<'a, Message: Clone>(
        label: &str,
        on_press: Option<Message>,
    ) -> button::Button<'a, Message> {
        let mut btn = button(text(label).size(14))
            .style(theme::Button::Custom(Box::new(GradientButton {
                is_primary: false,
                ..GradientButton::default()
            })))
            .padding([10, 16]);

        if let Some(msg) = on_press {
            btn = btn.on_press(msg);
        }

        btn
    }

    /// Create a modern text input
    pub fn text_input<'a, Message: Clone>(
        placeholder: &str,
        value: &str,
        on_change: impl Fn(String) -> Message + 'a,
    ) -> text_input::TextInput<'a, Message> {
        text_input(placeholder, value)
            .on_input(on_change)
            .style(theme::TextInput::Custom(Box::new(ModernTextInput::default())))
            .padding(12)
            .size(14)
    }

    /// Create a modern scrollable container
    pub fn scrollable<'a, Message>(
        content: Element<'a, Message>,
    ) -> scrollable::Scrollable<'a, Message> {
        scrollable(content)
            .style(theme::Scrollable::Custom(Box::new(ModernScrollable)))
    }

    /// Create a section header with modern styling
    pub fn section_header<'a, Message: 'a>(title: &str, subtitle: Option<&str>) -> Element<'a, Message> {
        let mut header = column![text(title)
            .size(20)
            .style(Color::from_rgb(0.95, 0.95, 0.95))];

        if let Some(sub) = subtitle {
            header = header.push(
                text(sub)
                    .size(14)
                    .style(Color::from_rgba(0.7, 0.7, 0.7, 0.9)),
            );
        }

        header.spacing(4).into()
    }

    /// Create a modern status indicator
    pub fn status_indicator<'a, Message: 'a>(
        status: &str,
        color: Color,
        is_active: bool,
    ) -> Element<'a, Message> {
        let indicator_color = if is_active { color } else { Color::from_rgba(0.4, 0.4, 0.4, 0.6) };

        row![
            container(Space::new(8, 8))
                .style(theme::Container::Custom(Box::new(StatusIndicatorStyle {
                    color: indicator_color
                })))
                .padding(0),
            Space::with_width(8),
            text(status)
                .size(12)
                .style(if is_active {
                    Color::from_rgb(0.9, 0.9, 0.9)
                } else {
                    Color::from_rgba(0.6, 0.6, 0.6, 0.8)
                })
        ]
        .align_items(Alignment::Center)
        .into()
    }
}

/// Status indicator style
#[derive(Debug, Clone)]
pub struct StatusIndicatorStyle {
    pub color: Color,
}

impl container::StyleSheet for StatusIndicatorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: None,
            shadow: Shadow {
                color: Color::from_rgba(self.color.r, self.color.g, self.color.b, 0.4),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
        }
    }
}

/// Animation states for UI components
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Hover,
    Active,
    Loading,
}

/// Modern theme colors palette
#[derive(Debug, Clone)]
pub struct ModernPalette {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub surface: Color,
    pub background: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub border: Color,
}

impl Default for ModernPalette {
    fn default() -> Self {
        Self {
            primary: Color::from_rgb(0.0, 0.5, 1.0),
            secondary: Color::from_rgb(0.4, 0.2, 0.8),
            success: Color::from_rgb(0.0, 0.8, 0.4),
            warning: Color::from_rgb(1.0, 0.7, 0.0),
            error: Color::from_rgb(1.0, 0.3, 0.3),
            info: Color::from_rgb(0.0, 0.7, 0.9),
            surface: Color::from_rgba(0.1, 0.1, 0.1, 0.9),
            background: Color::from_rgb(0.05, 0.05, 0.05),
            text_primary: Color::from_rgb(0.95, 0.95, 0.95),
            text_secondary: Color::from_rgba(0.7, 0.7, 0.7, 0.9),
            border: Color::from_rgba(0.3, 0.3, 0.3, 0.6),
        }
    }
}

// Additional component styles and types that are used by other modules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Danger,
    Ghost,
}

/// Creates a glass-morphism container (function wrapper for ModernUI::glass_container)
pub fn glass_container<'a, Message>(
    content: Element<'a, Message>
) -> container::Container<'a, Message> {
    ModernUI::glass_container(content)
}

/// Creates a modern button (function wrapper for ModernUI::primary_button/secondary_button)
pub fn modern_button<'a, Message: Clone>(
    label: &str,
    style: ButtonStyle,
    on_press: Option<Message>
) -> button::Button<'a, Message> {
    match style {
        ButtonStyle::Primary => ModernUI::primary_button(label, on_press),
        _ => ModernUI::secondary_button(label, on_press),
    }
}
