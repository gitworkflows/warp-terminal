use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column, Row},
    event::Status,
    Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Theme, Vector,
};

use iced_core::renderer::Style as IcedStyle;

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SubmitCommand,
    ClearOutput,
    TabSelected(TabType),
    ModelSelected(AiModel),
    WindowAction(WindowAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Terminal,
    AI,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AiModel {
    ClaudeAuto,
    ClaudeLite,
    Claude4Sonnet,
    Claude4Opus,
    Gpt4o,
    Gpt41,
    O4Mini,
    O3,
    Gemini25Pro,
}

impl std::fmt::Display for AiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AiModel::ClaudeAuto => "auto (claude 4 sonnet)",
            AiModel::ClaudeLite => "lite (basic model)",
            AiModel::Claude4Sonnet => "claude 4 sonnet",
            AiModel::Claude4Opus => "claude 4 opus",
            AiModel::Gpt4o => "gpt-4o",
            AiModel::Gpt41 => "gpt-4.1",
            AiModel::O4Mini => "o4-mini",
            AiModel::O3 => "o3",
            AiModel::Gemini25Pro => "gemini 2.5 pro",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone)]
pub enum WindowAction {
    Settings,
    Minimize,
    Maximize,
    Close,
}

#[derive(Debug, Clone)]
pub enum OutputType {
    Command,
    Output,
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub content: String,
    pub output_type: OutputType,
}

pub struct TabbedPane {
    input_value: String,
    output_lines: Vec<OutputLine>,
    active_tab: TabType,
    selected_model: AiModel,
    version: String,
}

impl Default for TabbedPane {
    fn default() -> Self {
        Self {
            input_value: String::new(),
            output_lines: Vec::new(),
            active_tab: TabType::Terminal,
            selected_model: AiModel::ClaudeAuto,
            version: "v22.15.0".to_string(),
        }
    }
}

impl TabbedPane {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::SubmitCommand => {
                if !self.input_value.trim().is_empty() {
                    self.output_lines.push(OutputLine {
                        content: format!("$ {}", self.input_value),
                        output_type: OutputType::Command,
                    });
                    
                    // Simulate command processing based on active tab
                    match self.active_tab {
                        TabType::Terminal => {
                            self.output_lines.push(OutputLine {
                                content: "Command executed successfully".to_string(),
                                output_type: OutputType::Output,
                            });
                        }
                        TabType::AI => {
                            self.output_lines.push(OutputLine {
                                content: format!("Processing with {} model...", self.selected_model),
                                output_type: OutputType::Info,
                            });
                        }
                        TabType::Settings => {
                            self.output_lines.push(OutputLine {
                                content: "Settings command processed".to_string(),
                                output_type: OutputType::Info,
                            });
                        }
                    }
                    
                    self.input_value.clear();
                }
            }
            Message::ClearOutput => {
                self.output_lines.clear();
            }
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Message::ModelSelected(model) => {
                self.selected_model = model;
            }
            Message::WindowAction(action) => {
                // Handle window actions (implement as needed)
                match action {
                    WindowAction::Settings => {
                        self.active_tab = TabType::Settings;
                    }
                    WindowAction::Minimize => {
                        // Implement minimize logic
                    }
                    WindowAction::Maximize => {
                        // Implement maximize logic
                    }
                    WindowAction::Close => {
                        // Implement close logic
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = self.create_header();
        let tab_bar = self.create_tab_bar();
        let content = self.create_content();
        let input_area = self.create_input_area();

        column![header, tab_bar, content, input_area]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_header(&self) -> Element<Message> {
        let version_text = text(&self.version)
            .size(14)
            .style(Color::from_rgb(0.7, 0.7, 0.7));

        let settings_btn = button("⚙")
            .on_press(Message::WindowAction(WindowAction::Settings))
            .style(|theme: &Theme, status| button_style(theme, status, false));

        let minimize_btn = button("−")
            .on_press(Message::WindowAction(WindowAction::Minimize))
            .style(|theme: &Theme, status| button_style(theme, status, false));

        let maximize_btn = button("□")
            .on_press(Message::WindowAction(WindowAction::Maximize))
            .style(|theme: &Theme, status| button_style(theme, status, false));

        let close_btn = button("×")
            .on_press(Message::WindowAction(WindowAction::Close))
            .style(|theme: &Theme, status| button_style(theme, status, true));

        let header_content = row![
            version_text,
            row![settings_btn, minimize_btn, maximize_btn, close_btn].spacing(8)
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .spacing(10);

        container(header_content)
            .style(|_theme: &Theme| IcedStyle {
                background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.9))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: [8.0, 8.0, 0.0, 0.0].into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .padding(Padding::from([12, 16]))
            .width(Length::Fill)
            .into()
    }

    fn create_tab_bar(&self) -> Element<Message> {
        let terminal_tab = button("Terminal")
            .on_press(Message::TabSelected(TabType::Terminal))
            .style(move |theme: &Theme, status| {
                tab_button_style(theme, status, self.active_tab == TabType::Terminal)
            });

        let ai_tab = button("AI")
            .on_press(Message::TabSelected(TabType::AI))
            .style(move |theme: &Theme, status| {
                tab_button_style(theme, status, self.active_tab == TabType::AI)
            });

        let settings_tab = button("Settings")
            .on_press(Message::TabSelected(TabType::Settings))
            .style(move |theme: &Theme, status| {
                tab_button_style(theme, status, self.active_tab == TabType::Settings)
            });

        let tabs = row![terminal_tab, ai_tab, settings_tab]
            .spacing(2)
            .align_items(Alignment::Center);

        container(tabs)
            .style(|_theme: &Theme| IcedStyle {
                background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.05, 0.9))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .padding(Padding::from([8, 16]))
            .width(Length::Fill)
            .into()
    }

    fn create_content(&self) -> Element<Message> {
        match self.active_tab {
            TabType::Terminal => self.create_terminal_content(),
            TabType::AI => self.create_ai_content(),
            TabType::Settings => self.create_settings_content(),
        }
    }

    fn create_terminal_content(&self) -> Element<Message> {
        let output_content = if self.output_lines.is_empty() {
            column![text("Terminal ready. Type commands below.")
                .size(14)
                .color(Color::from_rgb(0.6, 0.6, 0.6))]
        } else {
            let lines: Vec<Element<Message>> = self
                .output_lines
                .iter()
                .map(|line| {
                    let color = match line.output_type {
                        OutputType::Command => Color::from_rgb(0.4, 0.8, 0.4),
                        OutputType::Output => Color::from_rgb(0.9, 0.9, 0.9),
                        OutputType::Error => Color::from_rgb(0.9, 0.4, 0.4),
                        OutputType::Info => Color::from_rgb(0.4, 0.7, 0.9),
                    };
                    text(&line.content).size(13).color(color).into()
                })
                .collect();
            Column::with_children(lines).spacing(2)
        };

        let scrollable_content = scrollable(output_content).height(Length::Fill);

        container(scrollable_content)
            .style(|_theme: &Theme| IcedStyle {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.02, 0.95))),
                border: Border {
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_ai_content(&self) -> Element<Message> {
        let model_selector = pick_list(
            vec![
                AiModel::ClaudeAuto,
                AiModel::ClaudeLite,
                AiModel::Claude4Sonnet,
                AiModel::Claude4Opus,
                AiModel::Gpt4o,
                AiModel::Gpt41,
                AiModel::O4Mini,
                AiModel::O3,
                AiModel::Gemini25Pro,
            ],
            Some(self.selected_model.clone()),
            Message::ModelSelected,
        )
        .placeholder("Select AI Model")
        .style(|theme: &Theme, status| pick_list_style(theme, status));

        let model_info = text(format!("Current model: {}", self.selected_model))
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        let ai_content = column![
            text("AI Assistant")
                .size(18)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
            model_selector,
            model_info,
            text("Ask questions, request code assistance, or get help with your projects.")
                .size(14)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
        .spacing(16)
        .align_items(Alignment::Start);

        let output_content = if self.output_lines.is_empty() {
            ai_content
        } else {
            let lines: Vec<Element<Message>> = self
                .output_lines
                .iter()
                .map(|line| {
                    let color = match line.output_type {
                        OutputType::Command => Color::from_rgb(0.4, 0.8, 0.4),
                        OutputType::Output => Color::from_rgb(0.9, 0.9, 0.9),
                        OutputType::Error => Color::from_rgb(0.9, 0.4, 0.4),
                        OutputType::Info => Color::from_rgb(0.4, 0.7, 0.9),
                    };
                    text(&line.content).size(13).color(color).into()
                })
                .collect();
            column![ai_content, Column::with_children(lines).spacing(2)].spacing(16)
        };

        let scrollable_content = scrollable(output_content).height(Length::Fill);

        container(scrollable_content)
            .style(|_theme: &Theme| IcedStyle {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.02, 0.95))),
                border: Border {
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_settings_content(&self) -> Element<Message> {
        let settings_content = column![
            text("Settings")
                .size(18)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
            text("Configure your terminal and AI preferences here.")
                .size(14)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            text("• Theme settings")
                .size(13)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            text("• Keyboard shortcuts")
                .size(13)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            text("• AI model preferences")
                .size(13)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            text("• Terminal behavior")
                .size(13)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
        ]
        .spacing(12)
        .align_items(Alignment::Start);

        container(settings_content)
            .style(|_theme: &Theme| IcedStyle {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.02, 0.95))),
                border: Border {
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_input_area(&self) -> Element<Message> {
        let prompt = text("❯")
            .size(16)
            .color(Color::from_rgb(0.4, 0.8, 0.4));

        let placeholder = match self.active_tab {
            TabType::Terminal => "Type terminal commands here...",
            TabType::AI => "Ask AI, request code help, or describe what you want to build...",
            TabType::Settings => "Search settings or enter configuration commands...",
        };

        let input = text_input(placeholder, &self.input_value)
            .on_input(Message::InputChanged)
            .on_submit(Message::SubmitCommand)
            .size(14)
            .style(|theme: &Theme, status| text_input_style(theme, status));

        let clear_btn = button("Clear")
            .on_press(Message::ClearOutput)
            .style(|theme: &Theme, status| button_style(theme, status, false));

        let input_row = row![prompt, input, clear_btn]
            .spacing(12)
            .align_items(Alignment::Center)
            .width(Length::Fill);

        container(input_row)
            .style(|_theme: &Theme| IcedStyle {
                background: Some(Background::Color(Color::from_rgba(0.08, 0.08, 0.08, 0.9))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: [0.0, 0.0, 8.0, 8.0].into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, -2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .padding(Padding::from([16, 20]))
            .width(Length::Fill)
            .into()
    }
}

// Style functions
fn button_style(
    _theme: &Theme,
    status: Status,
    is_danger: bool,
) -> iced::widget::button::Appearance {
    let base_color = if is_danger {
        Color::from_rgba(0.8, 0.3, 0.3, 0.8)
    } else {
        Color::from_rgba(0.3, 0.3, 0.3, 0.8)
    };

    let hover_color = if is_danger {
        Color::from_rgba(0.9, 0.4, 0.4, 0.9)
    } else {
        Color::from_rgba(0.4, 0.4, 0.4, 0.9)
    };

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(base_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(hover_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgba(0.6, 0.6, 0.6, 0.7),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.9))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.7),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
            text_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
        },
    }
}

fn tab_button_style(
    _theme: &Theme,
    status: Status,
    is_active: bool,
) -> iced::widget::button::Appearance {
    let base_color = if is_active {
        Color::from_rgba(0.2, 0.4, 0.7, 0.9)
    } else {
        Color::from_rgba(0.15, 0.15, 0.15, 0.8)
    };

    let hover_color = if is_active {
        Color::from_rgba(0.3, 0.5, 0.8, 0.9)
    } else {
        Color::from_rgba(0.25, 0.25, 0.25, 0.9)
    };

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(base_color)),
            text_color: if is_active { Color::WHITE } else { Color::from_rgb(0.7, 0.7, 0.7) },
            border: Border {
                color: if is_active {
                    Color::from_rgba(0.4, 0.6, 0.9, 0.8)
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 0.5)
                },
                width: 1.0,
                radius: [4.0, 4.0, 0.0, 0.0].into(),
            },
            shadow: Shadow::default(),
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(hover_color)),
            text_color: Color::WHITE,
            border: Border {
                color: if is_active {
                    Color::from_rgba(0.5, 0.7, 1.0, 0.9)
                } else {
                    Color::from_rgba(0.4, 0.4, 0.4, 0.7)
                },
                width: 1.0,
                radius: [4.0, 4.0, 0.0, 0.0].into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.9))),
            text_color: Color::from_rgb(0.8, 0.8, 0.8),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.7),
                width: 1.0,
                radius: [4.0, 4.0, 0.0, 0.0].into(),
            },
            shadow: Shadow::default(),
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
            text_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgba(0.2, 0.2, 0.2, 0.3),
                width: 1.0,
                radius: [4.0, 4.0, 0.0, 0.0].into(),
            },
            shadow: Shadow::default(),
        },
    }
}

fn text_input_style(_theme: &Theme, status: Status) -> iced::widget::text_input::Appearance {
    match status {
        text_input::Status::Active => text_input::Style {
            background: Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.9)),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.8),
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: Color::from_rgb(0.7, 0.7, 0.7),
            placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.7),
            value: Color::from_rgb(0.9, 0.9, 0.9),
            selection: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
        },
        text_input::Status::Focused => text_input::Style {
            background: Background::Color(Color::from_rgba(0.12, 0.12, 0.12, 0.95)),
            border: Border {
                color: Color::from_rgba(0.4, 0.6, 0.9, 0.9),
                width: 2.0,
                radius: 6.0.into(),
            },
            icon: Color::from_rgb(0.8, 0.8, 0.8),
            placeholder: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            value: Color::WHITE,
            selection: Color::from_rgba(0.3, 0.5, 0.8, 0.6),
        },
        text_input::Status::Hovered => text_input::Style {
            background: Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.9)),
            border: Border {
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.9),
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: Color::from_rgb(0.8, 0.8, 0.8),
            placeholder: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            value: Color::from_rgb(0.95, 0.95, 0.95),
            selection: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(Color::from_rgba(0.05, 0.05, 0.05, 0.5)),
            border: Border {
                color: Color::from_rgba(0.2, 0.2, 0.2, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
            placeholder: Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            value: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            selection: Color::TRANSPARENT,
        },
    }
}

fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    match status {
        pick_list::Status::Active => pick_list::Style {
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            placeholder_color: Color::from_rgba(0.5, 0.5, 0.5, 0.7),
            handle_color: Color::from_rgb(0.7, 0.7, 0.7),
            background: Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.9)),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.8),
                width: 1.0,
                radius: 6.0.into(),
            },
        },
        pick_list::Status::Hovered => pick_list::Style {
            text_color: Color::WHITE,
            placeholder_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            handle_color: Color::from_rgb(0.8, 0.8, 0.8),
            background: Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.9)),
            border: Border {
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.9),
                width: 1.0,
                radius: 6.0.into(),
            },
        },
        pick_list::Status::Opened => pick_list::Style {
            text_color: Color::WHITE,
            placeholder_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            handle_color: Color::from_rgb(0.4, 0.6, 0.9),
            background: Background::Color(Color::from_rgba(0.12, 0.12, 0.12, 0.95)),
            border: Border {
                color: Color::from_rgba(0.4, 0.6, 0.9, 0.9),
                width: 2.0,
                radius: 6.0.into(),
            },
        },
    }
}
