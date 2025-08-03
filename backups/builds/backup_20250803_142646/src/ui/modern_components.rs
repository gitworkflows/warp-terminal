use iced::widget::{button, container, text, text_input, scrollable};
use iced::{theme, Background, Border, Color, Element, Shadow, Vector, Length};

/// Warp-style glass-morphism container (matching WarpPreview.app)
#[derive(Debug, Clone)]
pub struct WarpGlassContainer {
    pub blur_intensity: f32,
    pub opacity: f32,
    pub border_color: Color,
    pub shadow_color: Color,
    pub accent_color: Color,
}

impl Default for WarpGlassContainer {
    fn default() -> Self {
        Self {
            blur_intensity: 20.0,
            opacity: 0.12,
            border_color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
            shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            accent_color: Color::from_rgb(0.0, 0.82, 1.0), // Warp blue
        }
    }
}

impl container::StyleSheet for WarpGlassContainer {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.05, self.opacity))),
            border: Border {
                color: self.border_color,
                width: 1.0,
                radius: 16.0.into(),
            },
            text_color: Some(Color::from_rgb(0.96, 0.96, 0.96)),
            shadow: Shadow {
                color: self.shadow_color,
                offset: Vector::new(0.0, 12.0),
                blur_radius: 48.0,
            },
        }
    }
}

/// Warp-style elevated card (matching WarpPreview.app design)
#[derive(Debug, Clone)]
pub struct WarpModernCard {
    pub elevation: f32,
    pub hover_elevation: f32,
    pub background_color: Color,
    pub is_hovered: bool,
    pub is_focused: bool,
}

impl Default for WarpModernCard {
    fn default() -> Self {
        Self {
            elevation: 6.0,
            hover_elevation: 12.0,
            background_color: Color::from_rgba(0.04, 0.04, 0.04, 0.95),
            is_hovered: false,
            is_focused: false,
        }
    }
}

impl container::StyleSheet for WarpModernCard {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let elevation = if self.is_hovered {
            self.hover_elevation
        } else {
            self.elevation
        };

        let border_color = if self.is_focused {
            Color::from_rgb(0.0, 0.82, 1.0) // Warp blue
        } else {
            Color::from_rgba(0.25, 0.25, 0.25, 0.5)
        };

        container::Appearance {
            background: Some(Background::Color(self.background_color)),
            border: Border {
                color: border_color,
                width: if self.is_focused { 2.0 } else { 1.0 },
                radius: 12.0.into(),
            },
            text_color: Some(Color::from_rgb(0.96, 0.96, 0.96)),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, elevation / 2.0),
                blur_radius: elevation * 3.0,
            },
        }
    }
}

/// Warp-style gradient button (matching WarpPreview.app)
#[derive(Debug, Clone)]
pub struct WarpGradientButton {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub text_color: Color,
    pub is_primary: bool,
    pub is_disabled: bool,
}

impl Default for WarpGradientButton {
    fn default() -> Self {
        Self {
            primary_color: Color::from_rgb(0.0, 0.82, 1.0), // Warp blue
            secondary_color: Color::from_rgb(0.0, 0.6, 0.9),
            text_color: Color::WHITE,
            is_primary: true,
            is_disabled: false,
        }
    }
}

impl button::StyleSheet for WarpGradientButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        if self.is_disabled {
            return button::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.4))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                text_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
                shadow: Shadow::default(),
                shadow_offset: Vector::default(),
            };
        }

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
                    color: Color::from_rgba(0.0, 0.82, 1.0, 0.5),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
                shadow_offset: Vector::new(0.0, 4.0),
            }
        } else {
            button::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.8))),
                border: Border {
                    color: Color::from_rgba(0.35, 0.35, 0.35, 0.9),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                text_color: Color::from_rgb(0.92, 0.92, 0.92),
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                shadow_offset: Vector::new(0.0, 2.0),
            }
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        if self.is_disabled {
            return self.active(_style);
        }

        if self.is_primary {
            button::Appearance {
                background: Some(Background::Color(Color {
                    r: self.primary_color.r + 0.05,
                    g: self.primary_color.g + 0.05,
                    b: self.primary_color.b + 0.05,
                    a: self.primary_color.a,
                })),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                text_color: self.text_color,
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.82, 1.0, 0.7),
                    offset: Vector::new(0.0, 6.0),
                    blur_radius: 20.0,
                },
                shadow_offset: Vector::new(0.0, 6.0),
            }
        } else {
            button::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.25, 0.25, 0.25, 0.9))),
                border: Border {
                    color: Color::from_rgba(0.45, 0.45, 0.45, 1.0),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                text_color: Color::WHITE,
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                shadow_offset: Vector::new(0.0, 4.0),
            }
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        if self.is_disabled {
            return self.active(_style);
        }

        button::Appearance {
            background: Some(Background::Color(Color {
                r: self.primary_color.r - 0.05,
                g: self.primary_color.g - 0.05,
                b: self.primary_color.b - 0.05,
                a: self.primary_color.a,
            })),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: self.text_color,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.82, 1.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            shadow_offset: Vector::new(0.0, 2.0),
        }
    }
}

/// Warp-style text input (matching WarpPreview.app)
#[derive(Debug, Clone)]
pub struct WarpTextInput {
    pub focused: bool,
    pub accent_color: Color,
    pub error: bool,
}

impl Default for WarpTextInput {
    fn default() -> Self {
        Self {
            focused: false,
            accent_color: Color::from_rgb(0.0, 0.82, 1.0),
            error: false,
        }
    }
}

impl text_input::StyleSheet for WarpTextInput {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.08, 0.08, 0.08, 0.9)),
            border: Border {
                color: if self.error {
                    Color::from_rgb(1.0, 0.3, 0.3)
                } else if self.focused {
                    self.accent_color
                } else {
                    Color::from_rgba(0.25, 0.25, 0.25, 0.7)
                },
                width: if self.focused || self.error { 2.0 } else { 1.0 },
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.95)),
            border: Border {
                color: self.accent_color,
                width: 2.0,
                radius: 8.0.into(),
            },
            icon_color: self.accent_color,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.5, 0.5, 0.5, 0.9)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.96, 0.96, 0.96)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.4, 0.4, 0.4, 0.7)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(self.accent_color.r, self.accent_color.g, self.accent_color.b, 0.4)
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.03, 0.03, 0.03, 0.6)),
            border: Border {
                color: Color::from_rgba(0.15, 0.15, 0.15, 0.4),
                width: 1.0,
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgba(0.3, 0.3, 0.3, 0.7),
        }
    }
}

/// Warp-style scrollable (matching WarpPreview.app)
#[derive(Debug, Clone)]
pub struct WarpScrollable;

impl scrollable::StyleSheet for WarpScrollable {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: container::Appearance::default(),
            scrollbar: scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.05, 0.4))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                scroller: scrollable::Scroller {
                    color: Color::from_rgba(0.3, 0.3, 0.3, 0.9),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 6.0.into(),
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
                    background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.6))),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 6.0.into(),
                    },
                    scroller: scrollable::Scroller {
                        color: Color::from_rgba(0.5, 0.5, 0.5, 1.0),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 6.0.into(),
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
                background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.8))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                scroller: scrollable::Scroller {
                    color: Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 6.0.into(),
                    },
                },
            },
            gap: None,
        }
    }
}

/// Warp UI utility functions
pub struct WarpUI;

impl WarpUI {
    /// Create a Warp-style glass container
    pub fn glass_container<'a, Message>(
        content: Element<'a, Message>,
    ) -> container::Container<'a, Message> {
        container(content)
            .style(theme::Container::Custom(Box::new(WarpGlassContainer::default())))
            .padding(24)
    }

    /// Create a Warp-style card
    pub fn card<'a, Message>(content: Element<'a, Message>) -> container::Container<'a, Message> {
        container(content)
            .style(theme::Container::Custom(Box::new(WarpModernCard::default())))
            .padding(20)
    }

    /// Create a Warp-style primary button
    pub fn primary_button<'a, Message: Clone>(
        label: &str,
        on_press: Option<Message>,
    ) -> button::Button<'a, Message> {
        let mut btn = button(text(label).size(14))
            .style(theme::Button::Custom(Box::new(WarpGradientButton {
                is_primary: true,
                ..Default::default()
            })))
            .padding([12, 24]);

        if let Some(msg) = on_press {
            btn = btn.on_press(msg);
        }

        btn
    }

    /// Create a Warp-style secondary button
    pub fn secondary_button<'a, Message: Clone>(
        label: &str,
        on_press: Option<Message>,
    ) -> button::Button<'a, Message> {
        let mut btn = button(text(label).size(14))
            .style(theme::Button::Custom(Box::new(WarpGradientButton {
                is_primary: false,
                ..Default::default()
            })))
            .padding([12, 24]);

        if let Some(msg) = on_press {
            btn = btn.on_press(msg);
        }

        btn
    }

    /// Create a Warp-style text input
    pub fn text_input<'a, Message: Clone>(
        placeholder: &str,
        value: &str,
        on_change: impl Fn(String) -> Message + 'a,
    ) -> text_input::TextInput<'a, Message> {
        text_input(placeholder, value)
            .on_input(on_change)
            .style(theme::TextInput::Custom(Box::new(WarpTextInput::default())))
            .padding([12, 16])
            .size(14)
    }

    /// Create a Warp-style scrollable
    pub fn scrollable<'a, Message>(
        content: Element<'a, Message>,
    ) -> scrollable::Scrollable<'a, Message> {
        scrollable(content)
            .style(theme::Scrollable::Custom(Box::new(WarpScrollable)))
    }

    /// Create a Warp-style section header
    pub fn section_header<'a, Message: 'a>(title: &str) -> Element<'a, Message> {
        container(text(title).size(20).horizontal_alignment(iced::alignment::Horizontal::Center))
            .width(Length::Fill)
            .padding(10)
            .into()
    }
}
