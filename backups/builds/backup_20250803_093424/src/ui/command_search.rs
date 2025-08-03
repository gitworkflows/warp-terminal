use iced::{Element, Length, Color, Font};
use iced::widget::{column, row, text, text_input, button, container, scrollable, Space};
use crate::Message;
use crate::utils::fuzzy_matcher::FuzzyMatcher;
use crate::model::history::HistoryManager;
use warp_workflows_types::Workflow;

#[derive(Debug, Clone)]
pub struct CommandSearchPanel {
    pub query: String,
    pub is_visible: bool,
    pub selected_index: usize,
    pub results: Vec<SearchResult>,
    pub active_filter: SearchFilter,
    pub fuzzy_matcher: FuzzyMatcher,
    pub command_history: Vec<String>,
    pub workflows: Vec<WorkflowItem>,
    pub notebooks: Vec<NotebookItem>,
    pub show_landing_page: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchFilter {
    All,
    History,
    Workflows,
    Notebooks,
    Generate,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub text: String,
    pub description: String,
    pub result_type: SearchResultType,
    pub score: f32,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub enum SearchResultType {
    CommandHistory,
    Workflow,
    Notebook,
    AiGenerate,
}

#[derive(Debug, Clone)]
pub struct WorkflowItem {
    pub name: String,
    pub command: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NotebookItem {
    pub name: String,
    pub commands: Vec<String>,
    pub description: String,
}

impl Default for CommandSearchPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandSearchPanel {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            is_visible: false,
            selected_index: 0,
            results: Vec::new(),
            active_filter: SearchFilter::All,
            fuzzy_matcher: FuzzyMatcher::new(),
            command_history: Vec::new(),
            workflows: Self::load_workflows_from_directories(),
            notebooks: Self::load_default_notebooks(),
            show_landing_page: true,
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            self.show_landing_page = self.query.is_empty();
            if !self.query.is_empty() {
                self.update_search_results();
            }
        }
    }

    fn load_workflows_from_directories() -> Vec<WorkflowItem> {
        let mut workflows = Vec::new();
        
        // Load default workflows first
        workflows.extend(Self::load_default_workflows());
        
        // Try to load from local workflow directory
        if let Some(home_dir) = dirs::home_dir() {
            let local_workflow_dir = home_dir.join(".warp").join("workflows");
            workflows.extend(Self::load_workflows_from_directory(&local_workflow_dir));
        }
        
        // Try to load from current directory's .warp/workflows (repository workflows)
        let current_dir_workflows = std::env::current_dir()
            .map(|dir| dir.join(".warp").join("workflows"))
            .ok();
        
        if let Some(repo_workflow_dir) = current_dir_workflows {
            workflows.extend(Self::load_workflows_from_directory(&repo_workflow_dir));
        }
        
        workflows
    }
    
    fn load_workflows_from_directory(directory: &std::path::Path) -> Vec<WorkflowItem> {
        let mut workflows = Vec::new();
        
        if !directory.exists() {
            return workflows;
        }
        
        if let Ok(entries) = std::fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Only process YAML files
                if path.extension().and_then(|s| s.to_str()).map_or(false, |ext| {
                    ext == "yaml" || ext == "yml"
                }) {
                    if let Some(path_str) = path.to_str() {
                        if let Ok(workflow) = Workflow::load_from_yaml(path_str) {
                            workflows.push(WorkflowItem {
                                name: workflow.name.clone(),
                                command: workflow.command.clone(),
                                description: workflow.description.clone().unwrap_or_default(),
                                tags: workflow.tags.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        workflows
    }

    pub fn update_query(&mut self, new_query: String) {
        self.query = new_query;
        self.show_landing_page = self.query.is_empty();
        self.selected_index = 0;
        
        // Check for filter prefixes
        self.active_filter = self.detect_filter_prefix(&self.query);
        
        if !self.query.is_empty() {
            self.update_search_results();
        } else {
            self.results.clear();
        }
    }

    pub fn set_filter(&mut self, filter: SearchFilter) {
        self.active_filter = filter;
        self.update_search_results();
    }

    pub fn get_selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.results.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn add_to_history(&mut self, command: String) {
        if !self.command_history.contains(&command) {
            self.command_history.insert(0, command);
            if self.command_history.len() > 1000 {
                self.command_history.truncate(1000);
            }
        }
    }

    fn detect_filter_prefix(&self, query: &str) -> SearchFilter {
        if query.starts_with("history:") || query.starts_with("h:") {
            SearchFilter::History
        } else if query.starts_with("workflows:") || query.starts_with("w:") {
            SearchFilter::Workflows
        } else if query.starts_with("notebooks:") || query.starts_with("n:") {
            SearchFilter::Notebooks
        } else if query.starts_with("#:") {
            SearchFilter::Generate
        } else {
            SearchFilter::All
        }
    }

    fn get_clean_query(&self) -> String {
        let prefixes = ["history:", "h:", "workflows:", "w:", "notebooks:", "n:", "#:"];
        let mut clean_query = self.query.clone();
        
        for prefix in &prefixes {
            if clean_query.starts_with(prefix) {
                clean_query = clean_query[prefix.len()..].trim_start().to_string();
                break;
            }
        }
        
        clean_query
    }

    fn update_search_results(&mut self) {
        let clean_query = self.get_clean_query();
        let mut results = Vec::new();

        match self.active_filter {
            SearchFilter::All => {
                results.extend(self.search_history(&clean_query));
                results.extend(self.search_workflows(&clean_query));
                results.extend(self.search_notebooks(&clean_query));
                results.extend(self.search_ai_generate(&clean_query));
            }
            SearchFilter::History => {
                results.extend(self.search_history(&clean_query));
            }
            SearchFilter::Workflows => {
                results.extend(self.search_workflows(&clean_query));
            }
            SearchFilter::Notebooks => {
                results.extend(self.search_notebooks(&clean_query));
            }
            SearchFilter::Generate => {
                results.extend(self.search_ai_generate(&clean_query));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        self.results = results;
        self.selected_index = 0;
    }

    /// Search history using the new HistoryManager
    pub fn search_history_enhanced(&self, query: &str, history_manager: &HistoryManager) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        // Use the enhanced fuzzy search from HistoryManager
        let history_entries = history_manager.fuzzy_search(query, true); // Include all sessions
        
        for entry in history_entries {
            let mut description_parts = vec![
                format!("📁 {}", entry.directory.display()),
                format!("🕐 {}", self.format_timestamp(entry.timestamp)),
            ];
            
            if let Some(exit_code) = entry.exit_code {
                let status_icon = if exit_code == 0 { "✅" } else { "❌" };
                description_parts.push(format!("{} Exit: {}", status_icon, exit_code));
            }
            
            if let Some(exec_time) = entry.execution_time {
                description_parts.push(format!("⏱️ {:.2}ms", exec_time.as_millis()));
            }
            
            if entry.run_count > 1 {
                description_parts.push(format!("🔄 {} times", entry.run_count));
            }
            
            if entry.bookmarked {
                description_parts.push("⭐ Bookmarked".to_string());
            }
            
            let icon = if entry.bookmarked {
                "⭐"
            } else if entry.is_successful() {
                "🕒"
            } else {
                "❌"
            };
            
            results.push(SearchResult {
                text: entry.command.clone(),
                description: description_parts.join(" • "),
                result_type: SearchResultType::CommandHistory,
                score: self.calculate_fuzzy_score(query, &entry.command),
                icon: icon.to_string(),
            });
        }
        
        results
    }

    fn search_history(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        for (i, command) in self.command_history.iter().enumerate() {
            let score = self.calculate_fuzzy_score(query, command);
            if score > 0.0 {
                results.push(SearchResult {
                    text: command.clone(),
                    description: format!("Command from history ({})", i + 1),
                    result_type: SearchResultType::CommandHistory,
                    score,
                    icon: "🕒".to_string(),
                });
            }
        }
        
        results
    }

    fn search_workflows(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        for workflow in &self.workflows {
            let name_score = self.calculate_fuzzy_score(query, &workflow.name);
            let desc_score = self.calculate_fuzzy_score(query, &workflow.description) * 0.7;
            let tag_score = workflow.tags.iter()
                .map(|tag| self.calculate_fuzzy_score(query, tag) * 0.5)
                .fold(0.0, f32::max);
            
            let score = name_score.max(desc_score).max(tag_score);
            
            if score > 0.0 {
                results.push(SearchResult {
                    text: workflow.command.clone(),
                    description: format!("{} - {}", workflow.name, workflow.description),
                    result_type: SearchResultType::Workflow,
                    score,
                    icon: "$_".to_string(),
                });
            }
        }
        
        results
    }

    fn search_notebooks(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        for notebook in &self.notebooks {
            let name_score = self.calculate_fuzzy_score(query, &notebook.name);
            let desc_score = self.calculate_fuzzy_score(query, &notebook.description) * 0.7;
            let commands_score = notebook.commands.iter()
                .map(|cmd| self.calculate_fuzzy_score(query, cmd) * 0.8)
                .fold(0.0, f32::max);
            
            let score = name_score.max(desc_score).max(commands_score);
            
            if score > 0.0 {
                results.push(SearchResult {
                    text: notebook.commands.join(" && "),
                    description: format!("{} - {}", notebook.name, notebook.description),
                    result_type: SearchResultType::Notebook,
                    score,
                    icon: "📄".to_string(),
                });
            }
        }
        
        results
    }

    fn search_ai_generate(&self, query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        // Simple AI generate suggestions - in real implementation this would call an AI service
        vec![SearchResult {
            text: format!("Generate command for: {}", query),
            description: "AI-generated command suggestion".to_string(),
            result_type: SearchResultType::AiGenerate,
            score: 0.8,
            icon: "✨".to_string(),
        }]
    }

    fn format_timestamp(&self, timestamp: u64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let age = now.saturating_sub(timestamp);
        
        if age < 60 {
            "just now".to_string()
        } else if age < 3600 {
            format!("{}m ago", age / 60)
        } else if age < 86400 {
            format!("{}h ago", age / 3600)
        } else if age < 604800 {
            format!("{}d ago", age / 86400)
        } else {
            format!("{}w ago", age / 604800)
        }
    }

    fn calculate_fuzzy_score(&self, query: &str, target: &str) -> f32 {
        if query.is_empty() {
            return 0.0;
        }
        
        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();
        
        if target_lower.contains(&query_lower) {
            let exact_match_bonus = if target_lower.starts_with(&query_lower) { 0.2 } else { 0.0 };
            let length_ratio = query.len() as f32 / target.len() as f32;
            return 0.5 + length_ratio * 0.3 + exact_match_bonus;
        }
        
        // Simple character matching for fuzzy search
        let mut score = 0.0;
        let mut query_chars = query_lower.chars().peekable();
        
        for target_char in target_lower.chars() {
            if let Some(&query_char) = query_chars.peek() {
                if target_char == query_char {
                    score += 1.0;
                    query_chars.next();
                }
            }
        }
        
        if query_chars.peek().is_none() {
            score / target.len() as f32
        } else {
            0.0
        }
    }

    fn load_default_workflows() -> Vec<WorkflowItem> {
        vec![
            WorkflowItem {
                name: "Git Status".to_string(),
                command: "git status --porcelain".to_string(),
                description: "Show git status in short format".to_string(),
                tags: vec!["git".to_string(), "status".to_string()],
            },
            WorkflowItem {
                name: "Build Rust Project".to_string(),
                command: "cargo build --release".to_string(),
                description: "Build Rust project in release mode".to_string(),
                tags: vec!["rust".to_string(), "cargo".to_string(), "build".to_string()],
            },
            WorkflowItem {
                name: "List Docker Containers".to_string(),
                command: "docker ps -a".to_string(),
                description: "List all Docker containers".to_string(),
                tags: vec!["docker".to_string(), "containers".to_string()],
            },
        ]
    }

    fn load_default_notebooks() -> Vec<NotebookItem> {
        vec![
            NotebookItem {
                name: "Git Workflow".to_string(),
                commands: vec![
                    "git status".to_string(),
                    "git add .".to_string(),
                    "git commit -m \"Update\"".to_string(),
                    "git push origin main".to_string(),
                ],
                description: "Standard git workflow".to_string(),
            },
            NotebookItem {
                name: "System Info".to_string(),
                commands: vec![
                    "uname -a".to_string(),
                    "lscpu".to_string(),
                    "free -h".to_string(),
                    "df -h".to_string(),
                ],
                description: "Get system information".to_string(),
            },
        ]
    }

    pub fn view(&self, font: Font, font_size: u16) -> Element<Message> {
        if !self.is_visible {
            return Space::new(Length::Shrink, Length::Shrink).into();
        }

        let mut content = column![]
            .spacing(8)
            .padding(20)
            .width(Length::Fill);

        // Header with title and close button
        let header = row![
            text("Command Search")
                .size(font_size + 4)
                .font(font),
            Space::with_width(Length::Fill),
            button(text("✕").font(font).size(font_size))
                .on_press(Message::ToggleCommandSearch)
        ]
        .align_items(iced::Alignment::Center);

        content = content.push(header);

        // Search input
        let search_input = text_input("Search commands, workflows, notebooks...", &self.query)
            .on_input(Message::CommandSearchQueryChanged)
            .on_submit(Message::CommandSearchExecuteSelected)
            .size(font_size)
            .padding(10)
            .width(Length::Fill);

        content = content.push(search_input);

        // Filter buttons
        let filter_row = row![
            self.filter_button("All", SearchFilter::All, font, font_size),
            self.filter_button("History", SearchFilter::History, font, font_size),
            self.filter_button("Workflows", SearchFilter::Workflows, font, font_size),
            self.filter_button("Notebooks", SearchFilter::Notebooks, font, font_size),
            self.filter_button("Generate", SearchFilter::Generate, font, font_size),
        ]
        .spacing(8);

        content = content.push(filter_row);

        if self.show_landing_page {
            content = content.push(self.landing_page_view(font, font_size));
        } else if !self.results.is_empty() {
            content = content.push(self.results_view(font, font_size));
        } else if !self.query.is_empty() {
            let no_results = text("No results found")
                .size(font_size)
                .font(font)
                .style(Color::from_rgb(0.6, 0.6, 0.6));
            content = content.push(no_results);
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.95))),
                border: iced::Border::with_radius(8.0),
                ..Default::default()
            })
            .into()
    }

    fn filter_button(&self, label: &str, filter: SearchFilter, font: Font, font_size: u16) -> Element<Message> {
        let is_active = self.active_filter == filter;
        
        button(text(label).font(font).size(font_size))
            .on_press(Message::CommandSearchSetFilter(filter))
            .style(if is_active {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            })
            .into()
    }

    fn landing_page_view(&self, font: Font, font_size: u16) -> Element<Message> {
        let mut content = column![]
            .spacing(16)
            .padding(20);

        let welcome_text = text("Welcome to Command Search!")
            .size(font_size + 2)
            .font(font);
        content = content.push(welcome_text);

        let description = text("Search across command history, workflows, notebooks, and get AI-powered suggestions.")
            .size(font_size - 2)
            .font(font)
            .style(Color::from_rgb(0.7, 0.7, 0.7));
        content = content.push(description);

        // Quick filter buttons
        let quick_filters = column![
            text("Quick filters:").size(font_size - 1).font(font),
            row![
                button(text("🕒 Command History").font(font).size(font_size - 2))
                    .on_press(Message::CommandSearchSetFilter(SearchFilter::History)),
                button(text("$_ Workflows").font(font).size(font_size - 2))
                    .on_press(Message::CommandSearchSetFilter(SearchFilter::Workflows)),
                button(text("📄 Notebooks").font(font).size(font_size - 2))
                    .on_press(Message::CommandSearchSetFilter(SearchFilter::Notebooks)),
                button(text("✨ AI Generate").font(font).size(font_size - 2))
                    .on_press(Message::CommandSearchSetFilter(SearchFilter::Generate)),
            ]
            .spacing(8)
        ]
        .spacing(8);

        content = content.push(quick_filters);

        // Usage tips
        let tips = column![
            text("Tips:").size(font_size - 1).font(font),
            text("• Use 'h:' or 'history:' to filter command history").size(font_size - 2).font(font),
            text("• Use 'w:' or 'workflows:' to filter workflows").size(font_size - 2).font(font),
            text("• Use 'n:' or 'notebooks:' to filter notebooks").size(font_size - 2).font(font),
            text("• Use '#:' to activate AI Generate").size(font_size - 2).font(font),
        ]
        .spacing(4);

        content = content.push(tips);

        container(content)
            .width(Length::Fill)
            .into()
    }

    fn results_view(&self, font: Font, font_size: u16) -> Element<Message> {
        let mut results_column = column![]
            .spacing(4)
            .width(Length::Fill);

        for (index, result) in self.results.iter().take(10).enumerate() {
            let is_selected = index == self.selected_index;
            
            let result_row = row![
                text(&result.icon).size(font_size).font(font),
                column![
                    text(&result.text).size(font_size).font(font),
                    text(&result.description)
                        .size(font_size - 2)
                        .font(font)
                        .style(Color::from_rgb(0.7, 0.7, 0.7))
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_items(iced::Alignment::Center);

            let result_container = container(result_row)
                .padding(12)
                .width(Length::Fill)
                .style(container::Appearance {
                    background: Some(iced::Background::Color(
                        if is_selected {
                            Color::from_rgba(0.3, 0.3, 0.8, 0.3)
                        } else {
                            Color::from_rgba(0.2, 0.2, 0.2, 0.5)
                        }
                    )),
                    border: iced::Border::with_radius(4.0),
                    ..Default::default()
                });

            results_column = results_column.push(
                button(result_container)
                    .on_press(Message::CommandSearchSelectResult(index))
                    .style(iced::theme::Button::Text)
            );
        }

        scrollable(results_column)
            .height(Length::Fixed(400.0))
            .into()
    }
}
