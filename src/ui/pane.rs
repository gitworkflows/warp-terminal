use iced::{
    widget::{column, container, row, text, text_input, scrollable, Space},
    Alignment, Background, Border, Color, Element, Length, Padding, Theme,
};
use crate::ui::modern_components::{glass_container, modern_button, ButtonStyle};


#[derive(Debug, Clone)]
pub enum PaneMessage {
    InputChanged(String),
    CommandSubmitted,
    SettingsPressed,
    MinimizePressed,
    MaximizePressed,
    ClosePressed,
    ClearOutput,
}

#[derive(Debug, Clone)]
pub struct PaneState {
    pub input_value: String,
    pub output_content: Vec<OutputLine>,
    pub is_maximized: bool,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub content: String,
    pub line_type: OutputType,
}

#[derive(Debug, Clone)]
pub enum OutputType {
    Command,
    Output,
    Error,
    Info,
}

impl Default for PaneState {
    fn default() -> Self {
        Self {
            input_value: String::new(),
            output_content: vec![
                OutputLine {
                    content: "Welcome to Warp Terminal".to_string(),
                    line_type: OutputType::Info,
                },
                OutputLine {
                    content: "Type commands below or ask AI for help".to_string(),
                    line_type: OutputType::Info,
                },
            ],
            is_maximized: false,
            version: "v0.2024.01.09.08.02.stable_01".to_string(),
        }
    }
}

impl PaneState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_output_line(&mut self, content: String, line_type: OutputType) {
        self.output_content.push(OutputLine { content, line_type });
    }

    pub fn clear_output(&mut self) {
        self.output_content.clear();
    }

    pub fn submit_command(&mut self) {
        if !self.input_value.trim().is_empty() {
            // Add command to output
            self.add_output_line(
                format!("$ {}", self.input_value),
                OutputType::Command,
            );
            
            // Simulate command processing (in real implementation, this would execute the command)
            self.add_output_line(
                format!("Executing: {}", self.input_value),
                OutputType::Output,
            );
            
            self.input_value.clear();
        }
    }
}

pub fn pane<'a>(state: &PaneState) -> Element<'a, PaneMessage> {
    let header = create_header(&state.version, state.is_maximized);
    let input_area = create_input_area(&state.input_value);
    let content_area = create_content_area(&state.output_content);

    let main_content = column![
        header,
        input_area,
        content_area,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill);

    glass_container(main_content.into())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn create_header<'a>(version: &str, is_maximized: bool) -> Element<'a, PaneMessage> {
    let version_text = text(version)
        .size(12)
        .style(Color::from_rgba(1.0, 1.0, 1.0, 0.7));

    let settings_btn = modern_button("⚙", ButtonStyle::Secondary, Some(PaneMessage::SettingsPressed))
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(24.0));

    let minimize_btn = modern_button("−", ButtonStyle::Secondary, Some(PaneMessage::MinimizePressed))
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(24.0));

    let maximize_btn = modern_button(
        if is_maximized { "◱" } else { "□" },
        ButtonStyle::Secondary,
        Some(PaneMessage::MaximizePressed)
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(24.0));

    let close_btn = modern_button("×", ButtonStyle::Danger, Some(PaneMessage::ClosePressed))
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(24.0));

    let controls = row![
        settings_btn,
        minimize_btn,
        maximize_btn,
        close_btn,
    ]
    .spacing(4)
    .align_items(Alignment::Center);

    let header_content = row![
        version_text,
        horizontal_space(),
        controls,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);

    container(header_content)
        .padding(Padding::from([12, 16]))
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.15, 0.9))),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.4, 0.3),
                width: 0.0,
                radius: [8.0, 8.0, 0.0, 0.0].into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}

fn create_input_area<'a>(input_value: &str) -> Element<'a, PaneMessage> {
    let prompt_text = text("$")
        .size(16)
        .style(Color::from_rgba(0.4, 0.8, 0.4, 1.0)); // Green prompt

    let input_field = text_input("Code, ask, build, or run commands...", input_value)
        .on_input(PaneMessage::InputChanged)
        .on_submit(PaneMessage::CommandSubmitted)
        .size(16)
        .padding(Padding::from([8, 12]));

    let clear_btn = modern_button("Clear", ButtonStyle::Secondary, Some(PaneMessage::ClearOutput))
        .width(Length::Fixed(60.0))
        .height(Length::Fixed(32.0));

    let input_row = row![
        prompt_text,
        input_field,
        clear_btn,
    ]
    .spacing(12)
    .align_items(Alignment::Center)
    .width(Length::Fill);

    container(input_row)
        .padding(Padding::from([16, 16]))
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 0.8))),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.4, 0.2),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}

fn create_content_area<'a>(output_lines: &[OutputLine]) -> Element<'a, PaneMessage> {
    let mut content = column![]
        .spacing(4)
        .width(Length::Fill);

    for line in output_lines {
        let line_color = match line.line_type {
            OutputType::Command => Color::from_rgba(0.4, 0.8, 0.4, 1.0), // Green
            OutputType::Output => Color::from_rgba(0.9, 0.9, 0.95, 1.0), // Light gray
            OutputType::Error => Color::from_rgba(0.9, 0.4, 0.4, 1.0),   // Red
            OutputType::Info => Color::from_rgba(0.4, 0.7, 0.9, 1.0),    // Blue
        };

        let line_text = text(&line.content)
            .size(14)
            .style(line_color);

        content = content.push(line_text);
    }

    let scrollable_content = scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill);

    container(scrollable_content)
        .padding(Padding::from([16, 16]))
        .style(|_theme: &Theme| container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.05, 0.9))),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.4, 0.2),
                width: 0.0,
                radius: [0.0, 0.0, 8.0, 8.0].into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// Helper function to create horizontal space
fn horizontal_space<'a>() -> Element<'a, PaneMessage> {
    container(text(""))
        .width(Length::Fill)
        .into()
}

pub fn update(state: &mut PaneState, message: PaneMessage) {
    match message {
        PaneMessage::InputChanged(value) => {
            state.input_value = value;
        }
        PaneMessage::CommandSubmitted => {
            state.submit_command();
        }
        PaneMessage::SettingsPressed => {
            state.add_output_line("Settings opened".to_string(), OutputType::Info);
        }
        PaneMessage::MinimizePressed => {
            state.add_output_line("Window minimized".to_string(), OutputType::Info);
        }
        PaneMessage::MaximizePressed => {
            state.is_maximized = !state.is_maximized;
            let status = if state.is_maximized { "maximized" } else { "restored" };
            state.add_output_line(format!("Window {}", status), OutputType::Info);
        }
        PaneMessage::ClosePressed => {
            state.add_output_line("Close requested".to_string(), OutputType::Info);
        }
        PaneMessage::ClearOutput => {
            state.clear_output();
        }
    }
}
