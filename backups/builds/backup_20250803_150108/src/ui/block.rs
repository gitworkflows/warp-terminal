use crate::Block;
use crate::Message;
use crate::ui::icons;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Font, Color};
use iced::theme;

pub fn view_block(block: &Block, font: Font, size: u16) -> Element<Message> {
    let block_content = match &block.content {
        crate::model::block::BlockContent::Command { input, output } => block.command_block_view(input, output),
        crate::model::block::BlockContent::Background { output, process_info, is_active, pid } => {
            let mut col = column![
                text(format!("Background Process: {}", process_info.as_deref().unwrap_or("N/A"))).font(font).size(size - 2),
                text(format!("PID: {}", pid.map_or("N/A".to_string(), |p| p.to_string()))).font(font).size(size - 2),
                text(format!("Active: {}", is_active)).font(font).size(size - 2),
                text(output).font(font).size(size - 2),
            ].spacing(4);
            if !output.is_empty() {
                col = col.push(text(output).font(font).size(size - 2));
            }
            col.into()
        },
        crate::model::block::BlockContent::Markdown(content) => text(content).into(),
        crate::model::block::BlockContent::FilePreview(path) => text(format!("File: {}", path.display())).into(),
        crate::model::block::BlockContent::Error(message) => text(format!("Error: {}", message)).into(),
        crate::model::block::BlockContent::Info(message) => text(format!("Info: {}", message)).into(),
        crate::model::block::BlockContent::InteractiveCommand { input, output, streaming, real_time_updates } => {
            column![
                text("Interactive Command:").font(font).size(size - 4),
                text(input).font(font).size(size - 2),
                text(output).font(font).size(size - 2),
                text(format!("Streaming: {}, RealTime Updates: {}", streaming, real_time_updates)).font(font).size(size - 2)
            ].spacing(4).into()
        },
        crate::model::block::BlockContent::AIResponse { query, response, confidence, sources: _ } => {
            column![
                text("AI Response:").font(font).size(size - 4),
                text(format!("Query: {}", query)).font(font).size(size - 2),
                text(format!("Response: {}", response)).font(font).size(size - 2),
                text(format!("Confidence: {:.2}", confidence)).font(font).size(size - 2)
            ].spacing(4).into()
        },
        crate::model::block::BlockContent::CodeSnippet { language, code, highlighted } => {
            column![
                text("Code Snippet:").font(font).size(size - 4),
                text(format!("Language: {}", language)).font(font).size(size - 2),
                text(code).font(font).size(size - 2),
                text(format!("Highlighted: {}", highlighted)).font(font).size(size - 2)
            ].spacing(4).into()
        },
        crate::model::block::BlockContent::ImagePreview { path, thumbnail: _ } => {
            text(format!("Image Preview of {}", path.display())).font(font).size(size).into()
        },
        crate::model::block::BlockContent::NetworkRequest { url, method, response_status, response_body: _ } => {
            column![
                text("Network Request:").font(font).size(size - 4),
                text(format!("URL: {}", url)).font(font).size(size - 2),
                text(format!("Method: {}", method)).font(font).size(size - 2),
                text(format!("Status: {}", response_status.map_or("N/A".to_string(), |s| s.to_string()))).font(font).size(size - 2)
            ].spacing(4).into()
        },
    };
    
    // Status indicator with icon
    let status_icon = text(icons::get_status_icon(block.metadata.exit_code))
        .size(size)
        .style(match block.metadata.exit_code {
            Some(0) => Color::from_rgb(0.2, 0.8, 0.2),
            Some(_) => Color::from_rgb(0.9, 0.3, 0.3),
            None => Color::from_rgb(0.8, 0.8, 0.2),
        });

    // Control buttons with icons
    let copy_cmd_btn = button(
        row![
            text(icons::COPY).size(size - 2),
            text(" Cmd").font(font).size(size - 2)
        ].spacing(4)
    )
    .on_press(Message::CopyCommand(block.id))
    .style(theme::Button::Secondary);

    let copy_out_btn = button(
        row![
            text(icons::COPY).size(size - 2),
            text(" Out").font(font).size(size - 2)
        ].spacing(4)
    )
    .on_press(Message::CopyOutput(block.id))
    .style(theme::Button::Secondary);

    let share_btn = button(
        row![
            text(icons::SHARE).size(size - 2),
            text(" Share").font(font).size(size - 2)
        ].spacing(4)
    )
    .on_press(Message::ShareBlock(block.id))
    .style(theme::Button::Secondary);

    let bookmark_btn = button(
        row![
            text(icons::get_bookmark_icon(block.bookmarked)).size(size - 2),
            text(if block.bookmarked { " Saved" } else { " Save" }).font(font).size(size - 2)
        ].spacing(4)
    )
    .on_press(Message::BookmarkBlock(block.id))
    .style(if block.bookmarked { theme::Button::Primary } else { theme::Button::Secondary });

    let controls = column![
        status_icon,
        row![
            copy_cmd_btn,
            copy_out_btn,
        ].spacing(4),
        row![
            share_btn,
            bookmark_btn,
        ].spacing(4),
    ]
    .spacing(6)
    .align_items(Alignment::Start)
    .width(Length::Fixed(140.0));
    
    let content_with_controls = row![
        block_content,
        Space::with_width(Length::Fill),
        controls,
    ]
    .spacing(16)
    .align_items(Alignment::Start);
    
    container(content_with_controls)
        .width(Length::Fill)
        .padding(8)
        .into()
}
