//! Welcome screen UI for when the terminal is empty.

use crate::Message;
use iced::widget::{button, column, container, row, text};
use iced::{theme, Alignment, Background, Color, Element, Length, border, Font};

pub fn welcome_screen(font: Font, size: u16) -> Element<'static, Message> {
    // Main heading with gradient-like styling and emoji
    let heading = text("⚡ Hey there!")
        .font(font)
        .size(size + 24)
        .style(Color::from_rgb(0.95, 0.95, 1.0))
        .width(Length::Fill)
        .horizontal_alignment(iced::alignment::Horizontal::Center);
    
    let subtitle = text("Get started with one of these suggestions")
        .font(font)
        .size(size + 1)
        .style(Color::from_rgb(0.75, 0.8, 0.85))
        .width(Length::Fill)
        .horizontal_alignment(iced::alignment::Horizontal::Center);

    let suggestion_cards = column![
        row![
            suggestion_card("⬇️", "Install", "Install a binary/dependency", "npm install react", font, size, "npm install "),
            suggestion_card("</>", "Code", "Start a new project/feature or fix a bug", "git status", font, size, "git "),
        ].spacing(20).align_items(Alignment::Center),
        row![
            suggestion_card("🚀", "Deploy", "Deploy your project", "docker build -t myapp .", font, size, "docker "),
            suggestion_card("🤖", "Something else?", "Pair with an Agent to accomplish another task", "Ask me anything!", font, size, ""),
        ].spacing(20).align_items(Alignment::Center),
    ]
    .spacing(20)
    .align_items(Alignment::Center);

    // Quick tips section
    let tips = column![
        text("💡 Quick Tips:")
            .font(font)
            .size(size + 2)
            .style(Color::from_rgb(0.8, 0.8, 0.5)),
        text("• Type any command and press Enter to execute")
            .font(font)
            .size(size)
            .style(Color::from_rgb(0.6, 0.6, 0.6)),
        text("• Use Ctrl+C to copy command/output from blocks")
            .font(font)
            .size(size)
            .style(Color::from_rgb(0.6, 0.6, 0.6)),
        text("• Click the bookmark icon to save important commands")
            .font(font)
            .size(size)
            .style(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(8)
    .align_items(Alignment::Start)
    .width(Length::Fill);
    
    let content = column![
        heading,
        subtitle,
        suggestion_cards,
        container(tips)
            .padding(16)
            .style(theme::Container::Custom(Box::new(TipsContainerStyle)))
            .width(Length::Fixed(400.0))
    ]
    .spacing(32)
    .align_items(Alignment::Center)
    .width(Length::Fill);
    
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(32)
        .into()
}

fn suggestion_card(icon: &str, title: &str, description: &str, example: &str, font: Font, size: u16, command_prefix: &str) -> Element<'static, Message> {
    let icon_text = text(icon).size(size + 12).style(Color::from_rgb(0.9, 0.7, 0.2));
    let title_text = text(title)
        .font(font)
        .size(size + 3)
        .style(Color::from_rgb(0.95, 0.95, 1.0));
    let description_text = text(description)
        .font(font)
        .size(size - 1)
        .style(Color::from_rgb(0.7, 0.7, 0.8));
    let example_text = text(format!("e.g. {}", example))
        .font(font)
        .size(size - 2)
        .style(Color::from_rgb(0.5, 0.6, 0.7));
    
    let card_content = column![
        icon_text,
        title_text,
        description_text,
        example_text,
    ]
    .spacing(6)
    .align_items(Alignment::Start)
    .width(Length::Fill);
    
    let action = if !command_prefix.is_empty() {
        Message::InputChanged(command_prefix.to_string())
    } else {
        Message::CommandPaletteShow
    };
    
    button(card_content)
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(140.0))
        .padding(18)
        .on_press(action)
        .style(theme::Button::Custom(Box::new(SuggestionCardStyle)))
        .into()
}

struct SuggestionCardStyle;

impl button::StyleSheet for SuggestionCardStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            text_color: Color::WHITE,
            shadow_offset: iced::Vector::new(0.0, 2.0),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.3))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.4, 0.4, 0.6),
            },
            text_color: Color::WHITE,
            shadow_offset: iced::Vector::new(0.0, 4.0),
            ..Default::default()
        }
    }
}

struct TipsContainerStyle;

impl container::StyleSheet for TipsContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.1))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.2, 0.2, 0.3),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}
