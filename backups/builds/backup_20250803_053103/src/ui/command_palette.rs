//! Command Palette - Advanced command search and execution system for Warp Terminal
//!
//! This module provides a comprehensive command palette with fuzzy search, categories,
//! keyboard shortcuts, and rich command descriptions.

use crate::model::command_registry::{Command, CommandCategory, CommandRegistry};
use crate::model::workflow_loader::WorkflowLoader;
use crate::ui::quick_actions::{QuickActionsEngine, QuickAction};
use crate::ui::quick_actions::ActionCategory as QuickActionCategory;
use crate::Message;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, text_input, Space,
};
use iced::{
    theme, Alignment, Background, Border, Color, Element, Font, Length, Shadow, Vector,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Maximum number of search results to display
const MAX_SEARCH_RESULTS: usize = 15;

/// Maximum number of recent commands to remember
const MAX_RECENT_COMMANDS: usize = 20;

/// Command palette search and execution system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPalette {
    /// Whether the palette is currently visible
    pub is_visible: bool,
    /// Current search query
    pub query: String,
    /// Index of currently selected result
    pub selected_index: usize,
    /// Search results with match information
    pub results: Vec<SearchResult>,
    /// Recently executed commands (by ID)
    pub recent_commands: Vec<String>,
    /// Command registry containing all available commands
    pub command_registry: CommandRegistry,
    /// Active category filter (None = all categories)
    pub active_category: Option<CommandCategory>,
    /// Whether to show only favorites
    pub show_favorites_only: bool,
    /// Favorited command IDs
    pub favorites: Vec<String>,
    /// Search mode configuration
    pub search_config: SearchConfig,
    /// Workflow loader for loading YAML workflows
    pub workflow_loader: WorkflowLoader,
    /// Quick actions engine for context-aware actions
    #[serde(skip)]
    pub quick_actions_engine: QuickActionsEngine,
    /// Cached quick actions from last async refresh
    #[serde(skip)]
    pub cached_quick_actions: Vec<QuickAction>,
}

/// Search result with match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The command that matched
    pub command: Command,
    /// Match score (higher = better match)
    pub score: f64,
    /// Positions of matched characters in title
    pub title_matches: Option<Vec<usize>>,
    /// Positions of matched characters in description
    pub description_matches: Option<Vec<usize>>,
    /// Field that provided the best match
    pub best_match_field: MatchField,
}

/// Field that provided the best match for highlighting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MatchField {
    Title,
    Description,
    Keyword,
    Category,
}

/// Search configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable fuzzy matching
    pub fuzzy_enabled: bool,
    /// Minimum score threshold for results
    pub min_score: f64,
    /// Weight for title matches
    pub title_weight: f64,
    /// Weight for description matches
    pub description_weight: f64,
    /// Weight for keyword matches
    pub keyword_weight: f64,
    /// Whether to prioritize recent commands
    pub prioritize_recent: bool,
}

/// Statistics about the command palette
#[derive(Debug, Clone)]
pub struct CommandPaletteStats {
    pub total_commands: usize,
    pub recent_commands_count: usize,
    pub favorites_count: usize,
    pub current_results: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fuzzy_enabled: true,
            min_score: 0.1,
            title_weight: 3.0,
            description_weight: 1.0,
            keyword_weight: 2.0,
            prioritize_recent: true,
        }
    }
}

impl CommandPalette {
    /// Create a new command palette with default commands registered
    pub fn new() -> Self {
        let mut palette = Self {
            is_visible: false,
            query: String::new(),
            selected_index: 0,
            results: Vec::new(),
            recent_commands: Vec::new(),
            command_registry: CommandRegistry::new(),
            active_category: None,
            show_favorites_only: false,
            favorites: Vec::new(),
            search_config: SearchConfig::default(),
            workflow_loader: WorkflowLoader::default(),
            quick_actions_engine: QuickActionsEngine::new(),
            cached_quick_actions: Vec::new(),
        };

        // Register default commands
        palette.register_builtin_commands();
        
        // Load workflows from YAML files
        if let Err(e) = palette.workflow_loader.load_workflows(&mut palette.command_registry) {
            warn!("Failed to load workflows: {}", e);
        }

        // Initialize with recent commands
        palette.update_results();

        info!(
            "Command palette initialized with {} commands",
            palette.command_registry.get_all_commands().len()
        );

        palette
    }

    /// Toggle palette visibility
    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            self.reset_search();
            debug!("Command palette opened");
        } else {
            debug!("Command palette closed");
        }
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.is_visible = true;
        self.reset_search();
        debug!("Command palette shown");
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.reset_search();
        debug!("Command palette hidden");
    }

    /// Reset search state
    pub fn reset_search(&mut self) {
        self.query.clear();
        self.selected_index = 0;
        self.active_category = None;
        self.show_favorites_only = false;
        self.update_results();
    }

    /// Update search query and refresh results
    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0;
        self.update_results();
        debug!("Query updated: '{}'", self.query);
    }

    /// Set category filter
    pub fn set_category_filter(&mut self, category: Option<CommandCategory>) {
        let category_clone = category.clone();
        self.active_category = category;
        self.selected_index = 0;
        self.update_results();
        debug!("Category filter set: {:?}", category_clone);
    }

    /// Toggle favorites-only mode
    pub fn toggle_favorites_only(&mut self) {
        self.show_favorites_only = !self.show_favorites_only;
        self.selected_index = 0;
        self.update_results();
        debug!("Favorites-only mode: {}", self.show_favorites_only);
    }

    /// Add/remove command from favorites
    pub fn toggle_favorite(&mut self, command_id: &str) {
        if let Some(pos) = self.favorites.iter().position(|id| id == command_id) {
            self.favorites.remove(pos);
            debug!("Removed '{}' from favorites", command_id);
        } else {
            self.favorites.push(command_id.to_string());
            debug!("Added '{}' to favorites", command_id);
        }

        // Refresh results if in favorites mode
        if self.show_favorites_only {
            self.update_results();
        }
    }

    /// Check if command is favorited
    pub fn is_favorited(&self, command_id: &str) -> bool {
        self.favorites.contains(&command_id.to_string())
    }

    /// Update search results based on current query and filters
    pub fn update_results(&mut self) {
        if self.query.trim().is_empty() {
            // Show recent commands or favorites when no query
            if self.show_favorites_only {
                self.results = self.get_favorite_results();
            } else {
                let mut results = self.get_recent_results();
                // Add quick actions when no query
                results.extend(self.get_quick_action_results(5));
                self.results = results;
            }
        } else {
            // Perform search on both commands and quick actions
            let mut results = self.perform_search(&self.query);
            // Add quick actions that match the query
            results.extend(self.search_quick_actions(&self.query));
            self.results = results;
        }

        // Apply category filter
        if let Some(ref category) = self.active_category {
            self.results
                .retain(|result| &result.command.category == category);
        }

        // Sort by score
        self.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        self.results.truncate(MAX_SEARCH_RESULTS);

        // Ensure valid selection
        if self.selected_index >= self.results.len() && !self.results.is_empty() {
            self.selected_index = 0;
        }

        debug!("Updated results: {} matches", self.results.len());
    }

    /// Navigate to previous result
    pub fn navigate_up(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.results.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Navigate to next result
    pub fn navigate_down(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.results.len();
        }
    }

    /// Execute the currently selected command
    pub fn execute_selected(&mut self) -> Option<String> {
        if let Some(result) = self.results.get(self.selected_index) {
            let command_id = result.command.id.clone();
            let command_title = result.command.title.clone();
            self.add_to_recent(&command_id);
            self.hide();
            info!(
                "Executed command: {} ({})",
                command_title, command_id
            );
            Some(command_id)
        } else {
            None
        }
    }

    /// Add command to recent history
    pub fn add_to_recent(&mut self, command_id: &str) {
        // Remove if already exists
        self.recent_commands.retain(|id| id != command_id);

        // Add to front
        self.recent_commands.insert(0, command_id.to_string());

        // Limit size
        self.recent_commands.truncate(MAX_RECENT_COMMANDS);

        debug!("Added '{}' to recent commands", command_id);
    }

    /// Get favorite commands as search results
    fn get_favorite_results(&self) -> Vec<SearchResult> {
        self.favorites
            .iter()
            .filter_map(|id| self.command_registry.get(id))
            .map(|command| SearchResult {
                command: command.clone(),
                score: 1.0,
                title_matches: None,
                description_matches: None,
                best_match_field: MatchField::Title,
            })
            .collect()
    }

    /// Get recent commands as search results
    fn get_recent_results(&self) -> Vec<SearchResult> {
        self.recent_commands
            .iter()
            .filter_map(|id| self.command_registry.get(id))
            .map(|command| SearchResult {
                command: command.clone(),
                score: 1.0,
                title_matches: None,
                description_matches: None,
                best_match_field: MatchField::Title,
            })
            .collect()
    }

    /// Get quick actions as search results (cached from previous async call)
    fn get_quick_action_results(&self, limit: usize) -> Vec<SearchResult> {
        // Return cached quick actions converted to search results
        self.cached_quick_actions
            .iter()
            .take(limit)
            .map(|action| self.quick_action_to_search_result(action.clone(), 1.0))
            .collect()
    }

    /// Search quick actions based on query (from cached actions)
    fn search_quick_actions(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        self.cached_quick_actions
            .iter()
            .filter_map(|action| {
                // Simple scoring based on title and description match
                let title_score = if action.title.to_lowercase().contains(&query_lower) {
                    0.8
                } else { 0.0 };
                
                let desc_score = if action.description.to_lowercase().contains(&query_lower) {
                    0.5
                } else { 0.0 };
                
                let final_score = (title_score + desc_score) * action.confidence;
                
                if final_score > 0.1 {
                    Some(self.quick_action_to_search_result(action.clone(), final_score as f64))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convert a QuickAction to a SearchResult
    fn quick_action_to_search_result(&self, action: QuickAction, score: f64) -> SearchResult {
        let confidence = action.confidence;
        let command = self.quick_action_to_command(action);
        SearchResult {
            command,
            score: score * confidence as f64,
            title_matches: None,
            description_matches: None,
            best_match_field: MatchField::Title,
        }
    }

    /// Convert a QuickAction to a Command for display purposes
    fn quick_action_to_command(&self, action: QuickAction) -> Command {
        let category = match action.category {
            QuickActionCategory::Git => CommandCategory::Custom,
            QuickActionCategory::FileSystem => CommandCategory::Custom,
            QuickActionCategory::Development => CommandCategory::Custom,
            QuickActionCategory::Docker => CommandCategory::Custom,
            QuickActionCategory::SSH => CommandCategory::Custom,
            QuickActionCategory::System => CommandCategory::Settings,
            QuickActionCategory::Navigation => CommandCategory::Custom,
            QuickActionCategory::Recent => CommandCategory::History,
            QuickActionCategory::Suggested => CommandCategory::Custom,
        };

        Command {
            id: format!("quick.{}", action.id),
            title: action.title,
            description: action.description,
            category,
            shortcut: action.shortcut.unwrap_or_default(),
            keywords: Vec::new(), // QuickAction doesn't have keywords field
            enabled: true,
            priority: 0, // QuickAction doesn't have priority field, using confidence instead
        }
    }

    /// Perform fuzzy search across all commands
    fn perform_search(&self, query: &str) -> Vec<SearchResult> {
        let matcher = SkimMatcherV2::default();
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for command in self.command_registry.get_all_commands() {
            if let Some(search_result) = self.score_command(&command, &query_lower, &matcher) {
                // Apply recent command boost
                let boosted_score = if self.search_config.prioritize_recent
                    && self.recent_commands.contains(&command.id)
                {
                    search_result.score * 1.5
                } else {
                    search_result.score
                };

                if boosted_score >= self.search_config.min_score {
                    results.push(SearchResult {
                        score: boosted_score,
                        ..search_result
                    });
                }
            }
        }

        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Score a single command against the query
    fn score_command(
        &self,
        command: &Command,
        query: &str,
        matcher: &SkimMatcherV2,
    ) -> Option<SearchResult> {
        let mut best_score = 0.0;
        let mut best_field = MatchField::Title;
        let mut title_matches = None;
        let mut description_matches = None;

        // Score title match
        if let Some((score, matches)) =
            matcher.fuzzy_indices(&command.title.to_lowercase(), query)
        {
            let weighted_score = score as f64 * self.search_config.title_weight;
            if weighted_score > best_score {
                best_score = weighted_score;
                best_field = MatchField::Title;
                title_matches = Some(matches);
            }
        }

        // Score description match
        if !command.description.is_empty() {
            if let Some((score, matches)) =
                matcher.fuzzy_indices(&command.description.to_lowercase(), query)
            {
                let weighted_score = score as f64 * self.search_config.description_weight;
                if weighted_score > best_score {
                    best_score = weighted_score;
                    best_field = MatchField::Description;
                    description_matches = Some(matches);
                    title_matches = None; // Reset title matches as description won
                }
            }
        }

        // Score keyword matches
        for keyword in &command.keywords {
            if let Some(score) = matcher.fuzzy_match(&keyword.to_lowercase(), query) {
                let weighted_score = score as f64 * self.search_config.keyword_weight;
                if weighted_score > best_score {
                    best_score = weighted_score;
                    best_field = MatchField::Keyword;
                    title_matches = None;
                    description_matches = None;
                }
            }
        }

        // Score category match
        let category_str = format!("{:?}", command.category).to_lowercase();
        if category_str.contains(query) {
            let score = 50.0; // Fixed score for category matches
            if score > best_score {
                best_score = score;
                best_field = MatchField::Category;
                title_matches = None;
                description_matches = None;
            }
        }

        if best_score > 0.0 {
            Some(SearchResult {
                command: command.clone(),
                score: best_score,
                title_matches,
                description_matches,
                best_match_field: best_field,
            })
        } else {
            None
        }
    }

    /// Get the currently selected command ID
    pub fn get_selected_command_id(&self) -> Option<String> {
        self.results
            .get(self.selected_index)
            .map(|r| r.command.id.clone())
    }

    /// Register a new command
    pub fn register_command(&mut self, command: Command) {
        let _ = self.command_registry.register(command);
        debug!("Registered new command");
    }

    /// Unregister a command
    pub fn unregister_command(&mut self, command_id: &str) -> bool {
        if self.command_registry.unregister(command_id).is_some() {
            // Remove from recent commands and favorites
            self.recent_commands.retain(|id| id != command_id);
            self.favorites.retain(|id| id != command_id);
            debug!("Unregistered command: {}", command_id);
            true
        } else {
            false
        }
    }

    /// Refresh cached quick actions (to be called from async context)
    pub async fn refresh_quick_actions(&mut self) {
        match self.quick_actions_engine.get_quick_actions(10).await {
            actions => {
                self.cached_quick_actions = actions;
                debug!("Refreshed {} quick actions", self.cached_quick_actions.len());
                // Update results if palette is visible and no query
                if self.is_visible && self.query.trim().is_empty() {
                    self.update_results();
                }
            }
        }
    }

    /// Set cached quick actions (called from external async context)
    pub fn set_cached_quick_actions(&mut self, actions: Vec<QuickAction>) {
        self.cached_quick_actions = actions;
        debug!("Updated cached quick actions: {} items", self.cached_quick_actions.len());
        
        // Update results if palette is visible and no query
        if self.is_visible && self.query.trim().is_empty() {
            self.update_results();
        }
    }

    /// Get statistics about the command palette
    pub fn get_stats(&self) -> CommandPaletteStats {
        CommandPaletteStats {
            total_commands: self.command_registry.get_all_commands().len(),
            recent_commands_count: self.recent_commands.len(),
            favorites_count: self.favorites.len(),
            current_results: self.results.len(),
        }
    }

    /// Create the command palette UI view
    pub fn view(&self, font: Font, font_size: u16) -> Element<Message> {
        if !self.is_visible {
            return Space::new(Length::Fill, Length::Fill).into();
        }

        let content = column![
            self.create_header(font_size),
            horizontal_rule(1),
            self.create_search_input(font_size),
            self.create_filter_bar(font_size),
            horizontal_rule(1),
            self.create_results_view(font, font_size),
            horizontal_rule(1),
            self.create_footer(font_size),
        ]
        .spacing(0);

        container(content)
            .style(theme::Container::Custom(Box::new(CommandPaletteStyle)))
            .width(Length::Fixed(700.0))
            .height(Length::Fixed(500.0))
            .center_x()
            .center_y()
            .into()
    }

    /// Create the header with title and close button
    fn create_header(&self, font_size: u16) -> Element<Message> {
        let stats = self.get_stats();
        let subtitle = if self.query.is_empty() {
            if self.show_favorites_only {
                format!("💫 {} favorite commands", stats.favorites_count)
            } else {
                format!("⚡ {} recent commands", stats.recent_commands_count)
            }
        } else {
            format!(
                "🔍 {} of {} results",
                stats.current_results, stats.total_commands
            )
        };

        row![
            column![
                text("⚡ Command Palette")
                    .size(font_size + 4)
                    .style(Color::from_rgb(0.95, 0.95, 0.95)),
                text(subtitle)
                    .size(font_size - 2)
                    .style(Color::from_rgb(0.7, 0.7, 0.7))
            ],
            Space::with_width(Length::Fill),
            button(text("✕").size(font_size))
                .on_press(Message::CommandPaletteHide)
                .style(theme::Button::Custom(Box::new(CloseButtonStyle)))
        ]
        .align_items(Alignment::Center)
        .padding([12, 20])
        .into()
    }

    /// Create the search input area
    fn create_search_input(&self, font_size: u16) -> Element<Message> {
        row![
            text("🔍")
                .size(font_size + 2)
                .style(Color::from_rgb(0.6, 0.6, 0.6)),
            Space::with_width(12),
            text_input(
                "Type a command name, description, or keyword...",
                &self.query
            )
            .on_input(Message::CommandPaletteQueryChanged)
            .padding(12)
            .size(font_size)
            .width(Length::Fill)
        ]
        .align_items(Alignment::Center)
        .padding([8, 20])
        .into()
    }

    /// Create the filter bar with category buttons
    fn create_filter_bar(&self, font_size: u16) -> Element<Message> {
        let mut filters = row![]
            .push(self.create_filter_button("All", None, font_size))
            .push(self.create_favorites_button(font_size))
            .spacing(8);

        // Add category filter buttons
        for category in [
            CommandCategory::Pane,
            CommandCategory::Settings,
            CommandCategory::Theme,
            CommandCategory::History,
            CommandCategory::Search,
            CommandCategory::Custom,
            CommandCategory::Workflow,
        ] {
            let (icon, label) = self.get_category_display(&category);
            filters = filters.push(self.create_filter_button(
                &format!("{} {}", icon, label),
                Some(category),
                font_size,
            ));
        }

        container(scrollable(filters.align_items(Alignment::Center)))
            .padding([8, 20])
            .into()
    }

    /// Create a category filter button
    fn create_filter_button(
        &self,
        label: &str,
        category: Option<CommandCategory>,
        font_size: u16,
    ) -> Element<Message> {
        let is_active = self.active_category == category;

        button(text(label).size(font_size - 2))
            .on_press(Message::CommandPaletteSetCategory(category))
            .style(if is_active {
                theme::Button::Primary
            } else {
                theme::Button::Custom(Box::new(FilterButtonStyle))
            })
            .padding([6, 12])
            .into()
    }

    /// Create the favorites toggle button
    fn create_favorites_button(&self, font_size: u16) -> Element<Message> {
        let (icon, label) = if self.show_favorites_only {
            ("⭐", "Showing Favorites")
        } else {
            ("☆", "Show Favorites")
        };

        button(text(format!("{} {}", icon, label)).size(font_size - 2))
            .on_press(Message::CommandPaletteToggleFavorites)
            .style(if self.show_favorites_only {
                theme::Button::Custom(Box::new(ActiveFavoriteButtonStyle))
            } else {
                theme::Button::Custom(Box::new(FilterButtonStyle))
            })
            .padding([6, 12])
            .into()
    }

    /// Create the results list view
    fn create_results_view(&self, font: Font, font_size: u16) -> Element<Message> {
        if self.results.is_empty() {
            return container(
                column![
                    text("🤔")
                        .size(font_size * 3)
                        .style(Color::from_rgb(0.4, 0.4, 0.4)),
                    Space::with_height(16),
                    text("No commands found")
                        .size(font_size + 2)
                        .style(Color::from_rgb(0.6, 0.6, 0.6)),
                    text(if self.query.is_empty() {
                        "Start typing to search for commands"
                    } else {
                        "Try a different search term or check the filters"
                    })
                    .size(font_size - 2)
                    .style(Color::from_rgb(0.5, 0.5, 0.5))
                ]
                .align_items(Alignment::Center)
                .spacing(8),
            )
            .center_x()
            .center_y()
            .height(Length::Fixed(250.0))
            .width(Length::Fill)
            .into();
        }

        let mut results_column = column![].spacing(2).padding([0, 12]);

        for (index, result) in self.results.iter().enumerate() {
            let is_selected = index == self.selected_index;
            let command_item =
                self.create_command_item(result, is_selected, index, font, font_size);
            results_column = results_column.push(command_item);
        }

        scrollable(results_column)
            .height(Length::Fixed(250.0))
            .into()
    }

    /// Create a single command item in the results
    fn create_command_item(
        &self,
        result: &SearchResult,
        is_selected: bool,
        index: usize,
        _font: Font,
        font_size: u16,
    ) -> Element<Message> {
        let command = &result.command;
        let (category_icon, _) = self.get_category_display(&command.category);
        let is_favorited = self.is_favorited(&command.id);

        // Create title with highlighting
        let title_element = if let Some(ref matches) = result.title_matches {
            self.create_highlighted_text(&command.title, matches, font_size, is_selected)
        } else {
            text(&command.title)
                .size(font_size)
                .style(if is_selected {
                    Color::WHITE
                } else {
                    Color::from_rgb(0.9, 0.9, 0.9)
                })
                .into()
        };

        // Create description with highlighting
        let description_element = if !command.description.is_empty() {
            if let Some(ref matches) = result.description_matches {
                self.create_highlighted_text(
                    &command.description,
                    matches,
                    font_size - 2,
                    is_selected,
                )
            } else {
                text(&command.description)
                    .size(font_size - 2)
                    .style(if is_selected {
                        Color::from_rgb(0.8, 0.8, 0.8)
                    } else {
                        Color::from_rgb(0.6, 0.6, 0.6)
                    })
                    .into()
            }
        } else {
            Space::with_height(0).into()
        };

        // Header row with title, category, and favorite
        let header_row = row![
            title_element,
            Space::with_width(Length::Fill),
            text(category_icon).size(font_size - 2),
            Space::with_width(4),
            text(if is_favorited { "⭐" } else { "☆" })
                .size(font_size - 4)
                .style(if is_favorited {
                    Color::from_rgb(1.0, 0.8, 0.0)
                } else {
                    Color::from_rgb(0.4, 0.4, 0.4)
                })
        ]
        .align_items(Alignment::Center);

        // Footer row with shortcut and score (in debug mode)
        let mut footer_row = row![];

        if !command.shortcut.is_empty() {
            footer_row = footer_row.push(
                text(&command.shortcut)
                    .size(font_size - 4)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        }

        if cfg!(debug_assertions) && result.score > 0.0 {
            footer_row = footer_row.push(Space::with_width(Length::Fill)).push(
                text(format!("Score: {:.1}", result.score))
                    .size(font_size - 6)
                    .style(Color::from_rgb(0.3, 0.3, 0.3)),
            );
        }

        let content = column![header_row, description_element, footer_row]
            .spacing(4)
            .padding(12);

        button(content)
            .on_press(Message::CommandPaletteSelectResult(index))
            .style(if is_selected {
                theme::Button::Custom(Box::new(SelectedCommandStyle))
            } else {
                theme::Button::Custom(Box::new(UnselectedCommandStyle))
            })
            .width(Length::Fill)
            .into()
    }

    /// Create highlighted text with match positions
    fn create_highlighted_text(
        &self,
        text_content: &str,
        matches: &[usize],
        size: u16,
        is_selected: bool,
    ) -> Element<Message> {
        let normal_color = if is_selected {
            Color::from_rgb(0.9, 0.9, 0.9)
        } else {
            Color::from_rgb(0.8, 0.8, 0.8)
        };

        let highlight_color = Color::from_rgb(1.0, 0.9, 0.3);

        let mut row_element = row![];
        for (i, ch) in text_content.chars().enumerate() {
            let color = if matches.contains(&i) {
                highlight_color
            } else {
                normal_color
            };

            row_element = row_element.push(text(ch.to_string()).size(size).style(color));
        }

        row_element.into()
    }

    /// Create the footer with keyboard shortcuts and info
    fn create_footer(&self, font_size: u16) -> Element<Message> {
        let shortcuts = row![
            self.create_shortcut_hint("↑↓", "Navigate", font_size),
            Space::with_width(16),
            self.create_shortcut_hint("↵", "Execute", font_size),
            Space::with_width(16),
            self.create_shortcut_hint("Esc", "Close", font_size),
            Space::with_width(16),
            self.create_shortcut_hint("Ctrl+Shift+P", "Toggle", font_size),
        ]
        .align_items(Alignment::Center);

        let info = text(format!(
            "Showing {} of {} commands",
            self.results.len(),
            self.command_registry.get_all_commands().len()
        ))
        .size(font_size - 4)
        .style(Color::from_rgb(0.5, 0.5, 0.5));

        row![shortcuts, Space::with_width(Length::Fill), info]
            .align_items(Alignment::Center)
            .padding([12, 20])
            .into()
    }

    /// Create a keyboard shortcut hint
    fn create_shortcut_hint(
        &self,
        key: &str,
        description: &str,
        font_size: u16,
    ) -> Element<Message> {
        row![
            container(text(key).size(font_size - 6))
                .style(theme::Container::Custom(Box::new(ShortcutKeyStyle)))
                .padding([2, 6]),
            Space::with_width(6),
            text(description)
                .size(font_size - 4)
                .style(Color::from_rgb(0.7, 0.7, 0.7))
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Get display info for a command category
    fn get_category_display(&self, category: &CommandCategory) -> (&'static str, &'static str) {
        match category {
            CommandCategory::Pane => ("⚡", "Pane"),
            CommandCategory::Settings => ("⚙️", "Settings"),
            CommandCategory::Theme => ("🎨", "Theme"),
            CommandCategory::History => ("📚", "History"),
            CommandCategory::Search => ("🔍", "Search"),
            CommandCategory::Custom => ("🔧", "Custom"),
            CommandCategory::Workflow => ("🚀", "Workflows"),
        }
    }

    /// Register all built-in commands
    fn register_builtin_commands(&mut self) {
        // Pane Management Commands
        self.command_registry.register(Command::builder("pane.split.horizontal", "Split Pane Horizontally")
            .description("Split the current pane into two horizontal panes")
            .category(CommandCategory::Pane)
            .shortcut("Ctrl+Shift+D")
            .keywords(vec!["split".to_string(), "horizontal".to_string(), "pane".to_string(), "divide".to_string()])
            .build().unwrap()).unwrap();

        self.command_registry.register(Command::builder("pane.split.vertical", "Split Pane Vertically")
            .description("Split the current pane into two vertical panes")
            .category(CommandCategory::Pane)
            .shortcut("Ctrl+Shift+Shift+D")
            .keywords(vec!["split".to_string(), "vertical".to_string(), "pane".to_string(), "divide".to_string()])
            .build().unwrap()).unwrap();

        self.command_registry.register(Command::builder("pane.close", "Close Pane")
            .description("Close the current pane")
            .category(CommandCategory::Pane)
            .shortcut("Ctrl+W")
            .keywords(vec!["close".to_string(), "pane".to_string(), "exit".to_string(), "quit".to_string()])
            .build().unwrap()).unwrap();

        self.command_registry.register(Command::builder("pane.focus.next", "Focus Next Pane")
            .description("Move focus to the next pane")
            .category(CommandCategory::Pane)
            .shortcut("Ctrl+Tab")
            .keywords(vec!["focus".to_string(), "next".to_string(), "pane".to_string(), "switch".to_string()])
            .build().unwrap()).unwrap();

        // Settings Commands
        self.command_registry.register(Command {
            id: "settings.open".to_string(),
            title: "Open Settings".to_string(),
            description: "Open the settings panel".to_string(),
            category: CommandCategory::Settings,
            shortcut: "Ctrl+,".to_string(),
            keywords: vec!["settings", "preferences", "config", "options"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        self.command_registry.register(Command {
            id: "settings.export".to_string(),
            title: "Export Settings".to_string(),
            description: "Export settings to a file".to_string(),
            category: CommandCategory::Settings,
            shortcut: "".to_string(),
            keywords: vec!["export", "settings", "backup", "save"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        self.command_registry.register(Command {
            id: "settings.import".to_string(),
            title: "Import Settings".to_string(),
            description: "Import settings from a file".to_string(),
            category: CommandCategory::Settings,
            shortcut: "".to_string(),
            keywords: vec!["import", "settings", "restore", "load"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        // Theme Commands
        self.command_registry.register(Command {
            id: "theme.toggle".to_string(),
            title: "Toggle Theme".to_string(),
            description: "Switch between light and dark themes".to_string(),
            category: CommandCategory::Theme,
            shortcut: "".to_string(),
            keywords: vec!["theme", "dark", "light", "toggle", "switch"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        self.command_registry.register(Command {
            id: "theme.select".to_string(),
            title: "Select Theme".to_string(),
            description: "Choose from available themes".to_string(),
            category: CommandCategory::Theme,
            shortcut: "".to_string(),
            keywords: vec!["theme", "select", "choose", "change"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        // History Commands
        self.command_registry.register(Command {
            id: "history.search".to_string(),
            title: "Search History".to_string(),
            description: "Search through command history".to_string(),
            category: CommandCategory::History,
            shortcut: "Ctrl+R".to_string(),
            keywords: vec!["history", "search", "past", "commands", "find"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        self.command_registry.register(Command {
            id: "history.clear".to_string(),
            title: "Clear History".to_string(),
            description: "Clear all command history".to_string(),
            category: CommandCategory::History,
            shortcut: "".to_string(),
            keywords: vec!["history", "clear", "delete", "remove"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        // Search Commands
        self.command_registry.register(Command {
            id: "search.toggle".to_string(),
            title: "Toggle Search".to_string(),
            description: "Open or close the search panel".to_string(),
            category: CommandCategory::Search,
            shortcut: "Ctrl+F".to_string(),
            keywords: vec!["search", "find", "toggle", "panel"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();

        // Custom Commands
        self.command_registry.register(Command {
            id: "palette.toggle".to_string(),
            title: "Toggle Command Palette".to_string(),
            description: "Show or hide the command palette".to_string(),
            category: CommandCategory::Custom,
            shortcut: "Ctrl+Shift+P".to_string(),
            keywords: vec!["palette", "command", "toggle", "show", "hide"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            enabled: true,
            priority: 0,
        }).unwrap();
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

// Custom styles for command palette
struct CommandPaletteStyle;
struct SelectedCommandStyle;
struct UnselectedCommandStyle;
struct CloseButtonStyle;
struct FilterButtonStyle;
struct ActiveFavoriteButtonStyle;
struct ShortcutKeyStyle;

impl container::StyleSheet for CommandPaletteStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.05, 0.98))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 12.0.into(),
            },
            text_color: Some(Color::WHITE),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 20.0,
            },
        }
    }
}

impl button::StyleSheet for SelectedCommandStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.5, 1.0, 0.4))),
            border: Border {
                color: Color::from_rgb(0.0, 0.5, 1.0),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            shadow_offset: Vector::new(0.0, 2.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

impl button::StyleSheet for UnselectedCommandStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.6))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }
}

impl button::StyleSheet for CloseButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.4, 0.4),
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.8, 0.4, 0.4, 0.2))),
            border: Border {
                color: Color::from_rgb(0.8, 0.4, 0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(1.0, 0.5, 0.5),
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }
}

impl button::StyleSheet for FilterButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.6))),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.8),
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.8))),
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }
}

impl button::StyleSheet for ActiveFavoriteButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 0.8, 0.0, 0.3))),
            border: Border {
                color: Color::from_rgb(1.0, 0.8, 0.0),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(1.0, 0.9, 0.0),
            shadow: Shadow {
                color: Color::from_rgba(1.0, 0.8, 0.0, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            shadow_offset: Vector::new(0.0, 1.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

impl container::StyleSheet for ShortcutKeyStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.8))),
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommandPaletteMessage {
    Show,
    Hide,
    Toggle,
    QueryChanged(String),
    NavigateUp,
    NavigateDown,
    ExecuteSelected,
    SelectResult(usize),
    SetCategory(Option<CommandCategory>),
    ToggleFavorites,
    ToggleFavorite(String),
}
