use iced::{Element, Length, Padding, Color};
use iced::widget::{Text, Container, Column, Row, Scrollable, Rule};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub content: BlockContent,
    pub metadata: BlockMetadata,
    pub bookmarked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub timestamp: u64,
    pub execution_time: Option<Duration>,
    pub exit_code: Option<i32>,
    pub directory: Option<PathBuf>,
    pub shell: Option<String>,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub shareable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockContent {
    Command { input: String, output: String },
    Background { 
        output: String, 
        process_info: Option<String>, // Information about the likely source process
        is_active: bool, // Whether the background process is still producing output
        pid: Option<u32>, // Process ID for background process management
    },
    Markdown(String),
    FilePreview(PathBuf),
    Error(String),
    Info(String),
    // Additional enhanced content types
    InteractiveCommand {
        input: String,
        output: String,
        streaming: bool,
        real_time_updates: bool,
    },
    AIResponse {
        query: String,
        response: String,
        confidence: f32,
        sources: Vec<String>,
    },
    CodeSnippet {
        language: String,
        code: String,
        highlighted: bool,
    },
    ImagePreview {
        path: PathBuf,
        thumbnail: Option<Vec<u8>>,
    },
    NetworkRequest {
        url: String,
        method: String,
        response_status: Option<u16>,
        response_body: String,
    },
}

impl Block {
    pub fn new_command(input: String, output: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content: BlockContent::Command { input, output },
            metadata: BlockMetadata {
                timestamp: Self::current_timestamp(),
                execution_time: None,
                exit_code: None,
                directory: std::env::current_dir().ok(),
                shell: std::env::var("SHELL").ok(),
                tags: Vec::new(),
                is_pinned: false,
                shareable: true,
            },
            bookmarked: false,
        }
    }
    
    pub fn new_background(process_info: Option<String>, pid: Option<u32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            content: BlockContent::Background {
                output: String::new(),
                process_info,
                is_active: true,
                pid,
            },
            metadata: BlockMetadata {
                timestamp: Self::current_timestamp(),
                execution_time: None,
                exit_code: None,
                directory: std::env::current_dir().ok(),
                shell: std::env::var("SHELL").ok(),
                tags: Vec::new(),
                is_pinned: false,
                shareable: true,
            },
            bookmarked: false,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content = match &self.content {
            BlockContent::Command { input, output } => self.command_block_view(input, output),
            BlockContent::Background { output, process_info, is_active, pid } => {
                let mut col = Column::new();
                if let Some(info) = process_info {
                    col = col.push(Text::new(format!("Background Process: {}", info)));
                }
                if let Some(p) = pid {
                    col = col.push(Text::new(format!("PID: {}", p)));
                }
                col = col.push(Text::new(format!("Active: {}", is_active)));
                col = col.push(Text::new(output.clone()));
                col.into()
            },
            BlockContent::Markdown(content) => Text::new(content).into(),
            BlockContent::FilePreview(path) => Text::new(format!("File: {}", path.display())).into(),
            BlockContent::Error(message) => Text::new(format!("Error: {}", message)).into(),
            BlockContent::Info(message) => Text::new(format!("Info: {}", message)).into(),
            BlockContent::InteractiveCommand { input, output, streaming, real_time_updates } => {
                let mut col = Column::new();
                col = col.push(Text::new("Interactive Command:"));
                col = col.push(Text::new(input));
                col = col.push(Text::new(output));
                col = col.push(Text::new(format!("Streaming: {}, RealTime Updates: {}", streaming, real_time_updates)));
                col.into()
            },
            BlockContent::AIResponse { query, response, confidence, sources: _ } => {
                let mut col = Column::new();
                col = col.push(Text::new("AI Response:"));
                col = col.push(Text::new(format!("Query: {}", query)));
                col = col.push(Text::new(format!("Response: {}", response)));
                col = col.push(Text::new(format!("Confidence: {:.2}", confidence)));
                col.into()  
            },
            BlockContent::CodeSnippet { language, code, highlighted } => {
                let mut col = Column::new();
                col = col.push(Text::new("Code Snippet:"));
                col = col.push(Text::new(format!("Language: {}", language)));
                col = col.push(Text::new(code));
                col = col.push(Text::new(format!("Highlighted: {}", highlighted)));
                col.into()
            },
            BlockContent::ImagePreview { path, thumbnail: _ } => {
                Text::new(format!("Image Preview of {}", path.display())).into()
            },
            BlockContent::NetworkRequest { url, method, response_status, response_body: _ } => {
                let mut col = Column::new();
                col = col.push(Text::new("Network Request:"));
                col = col.push(Text::new(format!("URL: {}", url)));
                col = col.push(Text::new(format!("Method: {}", method)));
                col = col.push(Text::new(format!("Status: {}", response_status.map_or("N/A".to_string(), |s| s.to_string()))));
                col.into()
            },
        };

        Container::new(content)
            .padding(Padding::from([10, 15]))
            .width(Length::Fill)
            .into()
    }

    pub fn command_block_view<'a>(&self, input: &'a str, output: &'a str) -> Element<'a, Message> {
        let mut content = Column::new();

        // Command input with prompt
        let prompt_text = Text::new("❯").style(Color::from_rgb(0.4, 0.8, 0.4));
        let input_text = Text::new(input);
        let input_row = Row::new()
            .push(prompt_text)
            .push(input_text)
            .spacing(8)
            .padding(Padding::from([5, 10]));

        content = content.push(input_row);

        // Separator line
        content = content.push(Rule::horizontal(1));

        // Command output
        if !output.is_empty() {
            let output_text = Text::new(output)
                .size(13)
                .style(Color::from_rgb(0.8, 0.8, 0.8));
            
            let output_container = Container::new(
                Scrollable::new(output_text)
                    .height(Length::Shrink)
            )
            .padding(Padding::from([10, 15]));

            content = content.push(output_container);
        }

        // Block container
        Container::new(content)
            .padding(Padding::from([12, 16]))
            .width(Length::Fill)
            .into()
    }

    #[allow(dead_code)]
    fn generate_id() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn set_exit_code(&mut self, code: i32) {
        self.metadata.exit_code = Some(code);
    }
    
    pub fn set_execution_time(&mut self, duration: Duration) {
        self.metadata.execution_time = Some(duration);
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
        }
    }
    
    pub fn remove_tag(&mut self, tag: &str) {
        self.metadata.tags.retain(|t| t != tag);
    }
    
    pub fn toggle_pin(&mut self) {
        self.metadata.is_pinned = !self.metadata.is_pinned;
    }

    pub fn update_output(&mut self, new_output: String) {
        if let BlockContent::Command { ref mut output, .. } = self.content {
            *output = new_output;
        }
    }

    pub fn toggle_bookmark(&mut self) {
        self.bookmarked = !self.bookmarked;
    }

    pub fn get_command_text(&self) -> String {
        match &self.content {
            BlockContent::Command { input, .. } => input.clone(),
            _ => String::new(),
        }
    }

    pub fn get_output_text(&self) -> String {
        match &self.content {
            BlockContent::Command { output, .. } => output.clone(),
            _ => String::new(),
        }
    }

    pub fn get_both_text(&self) -> String {
        match &self.content {
            BlockContent::Command { input, output } => {
                format!("$ {}\n{}", input, output)
            }
            _ => String::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct BlockManager {
    blocks: Vec<Block>,
}

impl BlockManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_command(&mut self, input: String) -> &Block {
        let block = Block::new_command(input, String::new());
        self.blocks.push(block);
        self.blocks.last().unwrap()
    }

    pub fn add_background_block(&mut self, process_info: Option<String>, pid: Option<u32>) -> &Block {
        let block = Block::new_background(process_info, pid);
        self.blocks.push(block);
        self.blocks.last().unwrap()
    }

    pub fn update_block_output(&mut self, block_id: Uuid, output: String) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            match &mut block.content {
                BlockContent::Command { output: block_output, .. } => {
                    *block_output = output;
                },
                BlockContent::Background { output: block_output, .. } => {
                    *block_output = output;
                },
                _ => (),
            }
        }
    }

    pub fn set_block_exit_code(&mut self, block_id: Uuid, code: i32) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.set_exit_code(code);
            if let BlockContent::Background { is_active, .. } = &mut block.content {
                *is_active = false;
            }
        }
    }

    pub fn toggle_bookmark(&mut self, block_id: Uuid) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.toggle_bookmark();
        }
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }
}
