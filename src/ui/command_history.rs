//! Enhanced Command History & Search - Advanced command history management with intelligent search and analytics
//!
//! This module provides a comprehensive command history interface with:
//! - Advanced search with multiple filters
//! - Command analytics and statistics
//! - Smart suggestions based on usage patterns
//! - Visual timeline and usage insights

use crate::model::history::{HistoryEntry, HistoryManager};
use crate::Message;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, text_input, Space,
    progress_bar,
};
use iced::{
    theme, Alignment, Background, Border, Color, Element, Font, Length, Vector, Shadow,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info};
use uuid::Uuid;

/// Maximum number of search results to display
const MAX_SEARCH_RESULTS: usize = 50;

/// Maximum number of suggestions to show
#[allow(dead_code)]
const MAX_SUGGESTIONS: usize = 10;

/// Command History & Search interface
pub struct CommandHistoryUI {
    /// Whether the history panel is visible
    pub is_visible: bool,
    /// Current search query
    pub search_query: String,
    /// Active search filters
    pub active_filters: SearchFilters,
    /// Search results with match information
    pub search_results: Vec<HistorySearchResult>,
    /// Selected result index
    pub selected_index: usize,
    /// Current view mode
    pub view_mode: HistoryViewMode,
    /// Analytics data
    pub analytics: HistoryAnalytics,
    /// Smart suggestions
    pub suggestions: Vec<CommandSuggestion>,
    /// Fuzzy matcher for search (not Debug/Clone so we exclude from derives)
    fuzzy_matcher: SkimMatcherV2,
    /// Last search time for debouncing
    last_search_time: std::time::Instant,
}

// Manual Debug implementation to skip the fuzzy_matcher field
impl std::fmt::Debug for CommandHistoryUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandHistoryUI")
            .field("is_visible", &self.is_visible)
            .field("search_query", &self.search_query)
            .field("active_filters", &self.active_filters)
            .field("search_results", &self.search_results)
            .field("selected_index", &self.selected_index)
            .field("view_mode", &self.view_mode)
            .field("analytics", &self.analytics)
            .field("suggestions", &self.suggestions)
            .field("fuzzy_matcher", &"<SkimMatcherV2>")
            .field("last_search_time", &self.last_search_time)
            .finish()
    }
}

/// Search filters for command history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by command success/failure
    pub status_filter: Option<StatusFilter>,
    /// Filter by time range
    pub time_filter: TimeFilter,
    /// Filter by directory
    pub directory_filter: Option<String>,
    /// Filter by session
    pub session_filter: SessionFilter,
    /// Minimum execution time filter
    pub min_execution_time: Option<Duration>,
    /// Show only bookmarked commands
    pub bookmarked_only: bool,
    /// Show only frequent commands (run more than N times)
    pub frequent_only: Option<u32>,
}

/// Status filter options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StatusFilter {
    Success,
    Failed,
    Running,
}

/// Time filter options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimeFilter {
    All,
    Today,
    ThisWeek,
    ThisMonth,
    Custom { start: u64, end: u64 },
}

/// Session filter options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionFilter {
    CurrentSession,
    AllSessions,
    SpecificSession(Uuid),
}

/// View mode for history display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HistoryViewMode {
    List,
    Timeline,
    Analytics,
    Suggestions,
}

/// Search result with enhanced match information
#[derive(Debug, Clone)]
pub struct HistorySearchResult {
    pub entry: HistoryEntry,
    pub match_score: f64,
    pub match_positions: Vec<usize>,
    pub context_info: ContextInfo,
}

/// Context information for search results
#[derive(Debug, Clone)]
pub struct ContextInfo {
    pub similar_commands_count: usize,
    pub success_rate: f32,
    pub avg_execution_time: Option<Duration>,
    pub last_used_relative: String,
    pub usage_trend: UsageTrend,
}

/// Usage trend for commands
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsageTrend {
    Increasing,
    Stable,
    Decreasing,
    New,
}

/// Command suggestion with reasoning
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command: String,
    pub confidence: f32,
    pub reason: SuggestionReason,
    pub estimated_usage: u32,
    pub related_commands: Vec<String>,
}

/// Reason for command suggestion
#[derive(Debug, Clone)]
pub enum SuggestionReason {
    FrequentlyUsed,
    RecentlyUsed,
    PatternBased { pattern: String },
    ContextBased { context: String },
    TimeBasedHabit { time_pattern: String },
    DirectoryBased { directory: String },
}

/// Analytics data for command history
#[derive(Debug, Clone)]
pub struct HistoryAnalytics {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub success_rate: f32,
    pub avg_execution_time: Duration,
    pub most_used_commands: Vec<(String, u32)>,
    pub command_trends: HashMap<String, UsageTrend>,
    pub daily_activity: Vec<DailyActivity>,
    pub directory_usage: HashMap<String, u32>,
    pub error_patterns: Vec<ErrorPattern>,
}

/// Daily activity statistics
#[derive(Debug, Clone)]
pub struct DailyActivity {
    pub date: String,
    pub command_count: u32,
    pub success_rate: f32,
    pub avg_execution_time: Duration,
}

/// Common error pattern
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub command_pattern: String,
    pub error_count: u32,
    pub common_errors: Vec<String>,
    pub suggested_fixes: Vec<String>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            status_filter: None,
            time_filter: TimeFilter::All,
            directory_filter: None,
            session_filter: SessionFilter::CurrentSession,
            min_execution_time: None,
            bookmarked_only: false,
            frequent_only: None,
        }
    }
}

impl CommandHistoryUI {
    /// Create a new command history UI
    pub fn new() -> Self {
        Self {
            is_visible: false,
            search_query: String::new(),
            active_filters: SearchFilters::default(),
            search_results: Vec::new(),
            selected_index: 0,
            view_mode: HistoryViewMode::List,
            analytics: HistoryAnalytics::default(),
            suggestions: Vec::new(),
            fuzzy_matcher: SkimMatcherV2::default(),
            last_search_time: std::time::Instant::now(),
        }
    }

    /// Toggle history panel visibility
    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            self.refresh_suggestions();
        }
        debug!("Command history panel visibility: {}", self.is_visible);
    }

    /// Show the history panel
    pub fn show(&mut self) {
        self.is_visible = true;
        self.refresh_suggestions();
        debug!("Command history panel shown");
    }

    /// Hide the history panel
    pub fn hide(&mut self) {
        self.is_visible = false;
        debug!("Command history panel hidden");
    }

    /// Update search query and perform search
    pub fn update_search_query(&mut self, query: String, history_manager: &HistoryManager) {
        self.search_query = query;
        self.selected_index = 0;
        self.last_search_time = std::time::Instant::now();
        self.perform_search(history_manager);
        debug!("Search query updated: '{}'", self.search_query);
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: HistoryViewMode, history_manager: &HistoryManager) {
        self.view_mode = mode;
        match mode {
            HistoryViewMode::Analytics => self.refresh_analytics(history_manager),
            HistoryViewMode::Suggestions => self.refresh_suggestions(),
            _ => {}
        }
        debug!("View mode changed to: {:?}", mode);
    }

    /// Update search filters
    pub fn update_filters(&mut self, filters: SearchFilters, history_manager: &HistoryManager) {
        self.active_filters = filters;
        self.perform_search(history_manager);
        debug!("Search filters updated");
    }

    /// Navigate to previous result
    pub fn navigate_up(&mut self) {
        if !self.search_results.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.search_results.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Navigate to next result
    pub fn navigate_down(&mut self) {
        if !self.search_results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.search_results.len();
        }
    }

    /// Get selected command
    pub fn get_selected_command(&self) -> Option<String> {
        self.search_results
            .get(self.selected_index)
            .map(|result| result.entry.command.clone())
    }

    /// Perform search with current query and filters
    pub fn perform_search(&mut self, history_manager: &HistoryManager) {
        let query = self.search_query.trim();
        
        if query.is_empty() {
            // Show recent commands when no query
            self.search_results = self.get_recent_results(history_manager);
        } else {
            self.search_results = self.execute_advanced_search(query, history_manager);
        }

        // Ensure valid selection
        if self.selected_index >= self.search_results.len() && !self.search_results.is_empty() {
            self.selected_index = 0;
        }

        debug!("Search performed: {} results found", self.search_results.len());
    }

    /// Execute advanced search with fuzzy matching and scoring
    fn execute_advanced_search(&self, query: &str, history_manager: &HistoryManager) -> Vec<HistorySearchResult> {
        let history_entries = match self.active_filters.session_filter {
            SessionFilter::CurrentSession => history_manager.get_session_history(),
            SessionFilter::AllSessions => history_manager.get_combined_history(),
            SessionFilter::SpecificSession(_) => {
                // TODO: Implement specific session filtering
                history_manager.get_session_history()
            }
        };

        let mut scored_results = Vec::new();

        for entry in history_entries {
            // Apply filters first
            if !self.passes_filters(entry) {
                continue;
            }

            // Perform fuzzy matching
            if let Some((score, positions)) = self.fuzzy_matcher.fuzzy_indices(&entry.command, query) {
                let context_info = self.generate_context_info(entry, history_manager);
                
                // Calculate enhanced score
                let enhanced_score = self.calculate_enhanced_score(score as f64, entry, &context_info, query);
                
                scored_results.push(HistorySearchResult {
                    entry: entry.clone(),
                    match_score: enhanced_score,
                    match_positions: positions,
                    context_info,
                });
            }
        }

        // Sort by score (highest first)
        scored_results.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());
        scored_results.truncate(MAX_SEARCH_RESULTS);

        scored_results
    }

    /// Check if entry passes current filters
    fn passes_filters(&self, entry: &HistoryEntry) -> bool {
        // Status filter
        if let Some(status_filter) = self.active_filters.status_filter {
            match status_filter {
                StatusFilter::Success => {
                    if !entry.is_successful() {
                        return false;
                    }
                }
                StatusFilter::Failed => {
                    if entry.is_successful() {
                        return false;
                    }
                }
                StatusFilter::Running => {
                    if entry.exit_code.is_some() {
                        return false;
                    }
                }
            }
        }

        // Time filter
        let entry_time = entry.timestamp;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        match self.active_filters.time_filter {
            TimeFilter::All => {},
            TimeFilter::Today => {
                let today_start = now - (now % 86400);
                if entry_time < today_start {
                    return false;
                }
            }
            TimeFilter::ThisWeek => {
                let week_start = now - (7 * 86400);
                if entry_time < week_start {
                    return false;
                }
            }
            TimeFilter::ThisMonth => {
                let month_start = now - (30 * 86400);
                if entry_time < month_start {
                    return false;
                }
            }
            TimeFilter::Custom { start, end } => {
                if entry_time < start || entry_time > end {
                    return false;
                }
            }
        }

        // Directory filter
        if let Some(ref dir_filter) = self.active_filters.directory_filter {
            if !entry.directory.to_string_lossy().contains(dir_filter) {
                return false;
            }
        }

        // Execution time filter
        if let Some(min_time) = self.active_filters.min_execution_time {
            if let Some(exec_time) = entry.execution_time {
                if exec_time < min_time {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Bookmarked filter
        if self.active_filters.bookmarked_only && !entry.bookmarked {
            return false;
        }

        // Frequent commands filter
        if let Some(min_count) = self.active_filters.frequent_only {
            if entry.run_count < min_count {
                return false;
            }
        }

        true
    }

    /// Generate context information for a history entry
    fn generate_context_info(&self, entry: &HistoryEntry, history_manager: &HistoryManager) -> ContextInfo {
        let all_history = history_manager.get_combined_history();
        
        // Count similar commands
        let similar_commands_count = all_history
            .iter()
            .filter(|e| e.command.starts_with(&entry.command[..entry.command.len().min(10)]))
            .count();

        // Calculate success rate for this command
        let same_commands: Vec<_> = all_history
            .iter()
            .filter(|e| e.command == entry.command)
            .collect();
        
        let success_count = same_commands.iter().filter(|e| e.is_successful()).count();
        let success_rate = if same_commands.is_empty() {
            0.0
        } else {
            success_count as f32 / same_commands.len() as f32
        };

        // Calculate average execution time
        let avg_execution_time = {
            let exec_times: Vec<_> = same_commands
                .iter()
                .filter_map(|e| e.execution_time)
                .collect();
            
            if exec_times.is_empty() {
                None
            } else {
                let total_millis: u128 = exec_times.iter().map(|d| d.as_millis()).sum();
                Some(Duration::from_millis((total_millis / exec_times.len() as u128) as u64))
            }
        };

        // Calculate relative time
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let last_used_relative = self.format_relative_time(now - entry.last_run);

        // Determine usage trend (simplified)
        let usage_trend = if entry.run_count > 10 {
            UsageTrend::Stable
        } else if entry.run_count > 5 {
            UsageTrend::Increasing
        } else if entry.run_count == 1 {
            UsageTrend::New
        } else {
            UsageTrend::Decreasing
        };

        ContextInfo {
            similar_commands_count,
            success_rate,
            avg_execution_time,
            last_used_relative,
            usage_trend,
        }
    }

    /// Calculate enhanced relevance score
    fn calculate_enhanced_score(&self, base_score: f64, entry: &HistoryEntry, context: &ContextInfo, query: &str) -> f64 {
        let mut score = base_score;

        // Exact match bonus
        if entry.command.to_lowercase().contains(&query.to_lowercase()) {
            score *= 1.5;
        }

        // Prefix match bonus
        if entry.command.to_lowercase().starts_with(&query.to_lowercase()) {
            score *= 1.3;
        }

        // Frequency bonus
        score += (entry.run_count as f64).log10() * 10.0;

        // Recency bonus
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let age_hours = (now - entry.last_run) / 3600;
        score += 100.0 / (age_hours as f64 + 1.0);

        // Success rate bonus
        score += context.success_rate as f64 * 20.0;

        // Bookmark bonus
        if entry.bookmarked {
            score += 50.0;
        }

        // Usage trend bonus
        match context.usage_trend {
            UsageTrend::Increasing => score += 30.0,
            UsageTrend::Stable => score += 10.0,
            UsageTrend::New => score += 5.0,
            UsageTrend::Decreasing => score -= 10.0,
        }

        score
    }

    /// Get recent results when no search query
    fn get_recent_results(&self, history_manager: &HistoryManager) -> Vec<HistorySearchResult> {
        let recent_entries = history_manager.get_recent_commands(20);
        
        recent_entries
            .into_iter()
            .enumerate()
            .map(|(i, entry)| {
                let context_info = self.generate_context_info(entry, history_manager);
                HistorySearchResult {
                    entry: entry.clone(),
                    match_score: 100.0 - i as f64,
                    match_positions: Vec::new(),
                    context_info,
                }
            })
            .collect()
    }

    /// Refresh analytics data
    fn refresh_analytics(&mut self, history_manager: &HistoryManager) {
        let all_history = history_manager.get_combined_history();
        
        if all_history.is_empty() {
            self.analytics = HistoryAnalytics::default();
            return;
        }

        let total_commands = all_history.len();
        let unique_commands = all_history
            .iter()
            .map(|e| &e.command)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let success_count = all_history.iter().filter(|e| e.is_successful()).count();
        let success_rate = success_count as f32 / total_commands as f32;

        let avg_execution_time = {
            let exec_times: Vec<_> = all_history
                .iter()
                .filter_map(|e| e.execution_time)
                .collect();
            
            if exec_times.is_empty() {
                Duration::from_millis(0)
            } else {
                let total_millis: u128 = exec_times.iter().map(|d| d.as_millis()).sum();
                Duration::from_millis((total_millis / exec_times.len() as u128) as u64)
            }
        };

        // Calculate most used commands
        let mut command_counts: HashMap<String, u32> = HashMap::new();
        for entry in &all_history {
            *command_counts.entry(entry.command.clone()).or_insert(0) += entry.run_count;
        }
        
        let mut most_used_commands: Vec<_> = command_counts.into_iter().collect();
        most_used_commands.sort_by(|a, b| b.1.cmp(&a.1));
        most_used_commands.truncate(10);

        self.analytics = HistoryAnalytics {
            total_commands,
            unique_commands,
            success_rate,
            avg_execution_time,
            most_used_commands,
            command_trends: HashMap::new(), // TODO: Implement trend analysis
            daily_activity: Vec::new(),     // TODO: Implement daily activity
            directory_usage: HashMap::new(), // TODO: Implement directory usage
            error_patterns: Vec::new(),     // TODO: Implement error pattern analysis
        };

        info!("Analytics refreshed: {} commands, {:.1}% success rate", total_commands, success_rate * 100.0);
    }

    /// Refresh smart suggestions
    fn refresh_suggestions(&mut self) {
        // Generate smart suggestions based on patterns
        // This is a simplified implementation
        self.suggestions = vec![
            CommandSuggestion {
                command: "git status".to_string(),
                confidence: 0.9,
                reason: SuggestionReason::FrequentlyUsed,
                estimated_usage: 50,
                related_commands: vec!["git add".to_string(), "git commit".to_string()],
            },
            CommandSuggestion {
                command: "ls -la".to_string(),
                confidence: 0.8,
                reason: SuggestionReason::PatternBased { pattern: "directory listing".to_string() },
                estimated_usage: 30,
                related_commands: vec!["pwd".to_string(), "cd".to_string()],
            },
        ];

        debug!("Suggestions refreshed: {} suggestions", self.suggestions.len());
    }

    /// Format relative time for display
    fn format_relative_time(&self, seconds: u64) -> String {
        match seconds {
            0..=59 => "just now".to_string(),
            60..=3599 => format!("{}m ago", seconds / 60),
            3600..=86399 => format!("{}h ago", seconds / 3600),
            86400..=2591999 => format!("{}d ago", seconds / 86400),
            _ => format!("{}w ago", seconds / 604800),
        }
    }

    /// Create the main view based on current mode
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
            self.create_view_tabs(font_size),
            horizontal_rule(1),
            self.create_main_content(font, font_size),
            horizontal_rule(1),
            self.create_footer(font_size),
        ]
        .spacing(0);

        container(content)
            .style(theme::Container::Custom(Box::new(HistoryPanelStyle)))
            .width(Length::Fixed(800.0))
            .height(Length::Fixed(600.0))
            .center_x()
            .center_y()
            .into()
    }

    /// Create header with title and stats
    fn create_header(&self, font_size: u16) -> Element<Message> {
        let stats_text = format!(
            "📊 {} commands • {:.1}% success rate • {} results",
            self.analytics.total_commands,
            self.analytics.success_rate * 100.0,
            self.search_results.len()
        );

        row![
            column![
                text("🕒 Command History & Search")
                    .size(font_size + 4)
                    .style(Color::from_rgb(0.95, 0.95, 0.95)),
                text(stats_text)
                    .size(font_size - 2)
                    .style(Color::from_rgb(0.7, 0.7, 0.7))
            ],
            Space::with_width(Length::Fill),
            button(text("✕").size(font_size))
                .on_press(Message::ToggleCommandSearch)
                .style(theme::Button::Custom(Box::new(CloseButtonStyle)))
        ]
        .align_items(Alignment::Center)
        .padding([12, 20])
        .into()
    }

    /// Create search input area
    fn create_search_input(&self, font_size: u16) -> Element<Message> {
        row![
            text("🔍")
                .size(font_size + 2)
                .style(Color::from_rgb(0.6, 0.6, 0.6)),
            Space::with_width(12),
            text_input(
                "Search command history...",
                &self.search_query
            )
            .on_input(Message::CommandSearchQueryChanged)
            .padding(12)
            .size(font_size)
            .width(Length::Fill)
        ]
        .align_items(Alignment::Center)
        .padding([8, 20])
        .into()
    }

    /// Create filter bar
    fn create_filter_bar(&self, font_size: u16) -> Element<Message> {
        let filters = row![]
            .push(self.create_filter_button("All", None, font_size))
            .push(self.create_filter_button("Success", Some("success"), font_size))
            .push(self.create_filter_button("Failed", Some("failed"), font_size))
            .push(self.create_filter_button("Recent", Some("recent"), font_size))
            .push(self.create_filter_button("Frequent", Some("frequent"), font_size))
            .push(self.create_filter_button("Bookmarked", Some("bookmarked"), font_size))
            .spacing(8);

        container(scrollable(filters.align_items(Alignment::Center)))
            .padding([8, 20])
            .into()
    }

    /// Create filter button
    fn create_filter_button(&self, label: &str, _filter_type: Option<&str>, font_size: u16) -> Element<Message> {
        let is_active = false; // TODO: Check if this filter is active

        button(text(label).size(font_size - 2))
            .style(if is_active {
                theme::Button::Primary
            } else {
                theme::Button::Custom(Box::new(FilterButtonStyle))
            })
            .padding([6, 12])
            .into()
    }

    /// Create view mode tabs
    fn create_view_tabs(&self, font_size: u16) -> Element<Message> {
        row![
            self.create_tab_button("📋 List", HistoryViewMode::List, font_size),
            self.create_tab_button("📈 Timeline", HistoryViewMode::Timeline, font_size),
            self.create_tab_button("📊 Analytics", HistoryViewMode::Analytics, font_size),
            self.create_tab_button("💡 Suggestions", HistoryViewMode::Suggestions, font_size),
        ]
        .spacing(4)
        .padding([8, 20])
        .into()
    }

    /// Create tab button
    fn create_tab_button(&self, label: &str, mode: HistoryViewMode, font_size: u16) -> Element<Message> {
        let is_active = self.view_mode == mode;

        button(text(label).size(font_size - 2))
            .style(if is_active {
                theme::Button::Primary
            } else {
                theme::Button::Custom(Box::new(TabButtonStyle))
            })
            .padding([8, 16])
            .into()
    }

    /// Create main content based on view mode
    fn create_main_content(&self, font: Font, font_size: u16) -> Element<Message> {
        match self.view_mode {
            HistoryViewMode::List => self.create_list_view(font, font_size),
            HistoryViewMode::Timeline => self.create_timeline_view(font, font_size),
            HistoryViewMode::Analytics => self.create_analytics_view(font, font_size),
            HistoryViewMode::Suggestions => self.create_suggestions_view(font, font_size),
        }
    }

    /// Create list view of search results
    fn create_list_view(&self, font: Font, font_size: u16) -> Element<Message> {
        if self.search_results.is_empty() {
            return container(
                column![
                    text("🤔")
                        .size(font_size * 3)
                        .style(Color::from_rgb(0.4, 0.4, 0.4)),
                    Space::with_height(16),
                    text("No commands found")
                        .size(font_size + 2)
                        .style(Color::from_rgb(0.6, 0.6, 0.6)),
                    text("Try adjusting your search query or filters")
                        .size(font_size - 2)
                        .style(Color::from_rgb(0.5, 0.5, 0.5))
                ]
                .align_items(Alignment::Center)
                .spacing(8),
            )
            .center_x()
            .center_y()
            .height(Length::Fixed(300.0))
            .width(Length::Fill)
            .into();
        }

        let mut results_column = column![].spacing(2).padding([0, 12]);

        for (index, result) in self.search_results.iter().enumerate() {
            let is_selected = index == self.selected_index;
            let result_item = self.create_result_item(result, is_selected, font, font_size);
            results_column = results_column.push(result_item);
        }

        scrollable(results_column)
            .height(Length::Fixed(300.0))
            .into()
    }

    /// Create individual result item
    fn create_result_item(&self, result: &HistorySearchResult, is_selected: bool, _font: Font, font_size: u16) -> Element<Message> {
        let entry = &result.entry;
        
        // Status indicator
        let status_icon = if entry.exit_code.is_none() {
            "⏳" // Running
        } else if entry.is_successful() {
            "✅" // Success
        } else {
            "❌" // Failed
        };

        // Command text with highlighting
        let command_text = if result.match_positions.is_empty() {
            text(&entry.command).size(font_size)
        } else {
            // TODO: Implement highlighting
            text(&entry.command).size(font_size)
        };

        // Context info
        let context_text = format!(
            "{} • {} • {} • {}x",
            status_icon,
            result.context_info.last_used_relative,
            entry.format_execution_time(),
            entry.run_count
        );

        let header_row = row![
            command_text,
            Space::with_width(Length::Fill),
            text(context_text).size(font_size - 4).style(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
        .align_items(Alignment::Center);

        let directory_row = row![
            text("📁").size(font_size - 6),
            Space::with_width(4),
            text(entry.directory.to_string_lossy()).size(font_size - 4).style(Color::from_rgb(0.5, 0.5, 0.6)),
            Space::with_width(Length::Fill),
            if entry.bookmarked {
                text("⭐").size(font_size - 4).style(Color::from_rgb(1.0, 0.8, 0.0))
            } else {
                text("").size(font_size - 4)
            }
        ];

        let content = column![header_row, directory_row]
            .spacing(4)
            .padding(12);

        button(content)
            .style(if is_selected {
                theme::Button::Custom(Box::new(SelectedResultStyle))
            } else {
                theme::Button::Custom(Box::new(UnselectedResultStyle))
            })
            .width(Length::Fill)
            .into()
    }

    /// Create timeline view (placeholder)
    fn create_timeline_view(&self, _font: Font, font_size: u16) -> Element<Message> {
        container(
            text("📈 Timeline view coming soon...")
                .size(font_size)
                .style(Color::from_rgb(0.6, 0.6, 0.6))
        )
        .center_x()
        .center_y()
        .height(Length::Fixed(300.0))
        .width(Length::Fill)
        .into()
    }

    /// Create analytics view
    fn create_analytics_view(&self, _font: Font, font_size: u16) -> Element<Message> {
        let mut analytics_column = column![].spacing(16).padding(20);

        // Most used commands
        if !self.analytics.most_used_commands.is_empty() {
            analytics_column = analytics_column.push(
                column![
                    text("📊 Most Used Commands").size(font_size + 2).style(Color::from_rgb(0.9, 0.9, 0.9)),
                    Space::with_height(8),
                    {
                        let mut commands_list = column![].spacing(4);
                        for (i, (command, count)) in self.analytics.most_used_commands.iter().take(5).enumerate() {
                            let _bar_width = (*count as f32 / self.analytics.most_used_commands[0].1 as f32) * 200.0;
                            commands_list = commands_list.push(
                                row![
                                    text(format!("{}.", i + 1)).size(font_size - 2).style(Color::from_rgb(0.5, 0.5, 0.6)),
                                    Space::with_width(8),
                                    text(command).size(font_size - 1),
                                    Space::with_width(Length::Fill),
                                    text(format!("{}x", count)).size(font_size - 2).style(Color::from_rgb(0.7, 0.7, 0.8)),
                                ]
                                .align_items(Alignment::Center)
                            );
                        }
                        commands_list
                    }
                ]
            );
        }

        scrollable(analytics_column)
            .height(Length::Fixed(300.0))
            .into()
    }

    /// Create suggestions view
    fn create_suggestions_view(&self, _font: Font, font_size: u16) -> Element<Message> {
        if self.suggestions.is_empty() {
            return container(
                text("💡 No suggestions available yet")
                    .size(font_size)
                    .style(Color::from_rgb(0.6, 0.6, 0.6))
            )
            .center_x()
            .center_y()
            .height(Length::Fixed(300.0))
            .width(Length::Fill)
            .into();
        }

        let mut suggestions_column = column![].spacing(8).padding(20);

        suggestions_column = suggestions_column.push(
            text("💡 Smart Suggestions")
                .size(font_size + 2)
                .style(Color::from_rgb(0.9, 0.9, 0.9))
        );

        for suggestion in &self.suggestions {
            let confidence_bar = progress_bar(0.0..=1.0, suggestion.confidence);
            
            let reason_text = match &suggestion.reason {
                SuggestionReason::FrequentlyUsed => "Frequently used",
                SuggestionReason::RecentlyUsed => "Recently used",
                SuggestionReason::PatternBased { pattern } => &format!("Pattern: {}", pattern),
                SuggestionReason::ContextBased { context } => &format!("Context: {}", context),
                SuggestionReason::TimeBasedHabit { time_pattern } => &format!("Time pattern: {}", time_pattern),
                SuggestionReason::DirectoryBased { directory } => &format!("Directory: {}", directory),
            };

            let suggestion_item = column![
                row![
                    text(&suggestion.command).size(font_size),
                    Space::with_width(Length::Fill),
                    text(format!("{:.0}%", suggestion.confidence * 100.0))
                        .size(font_size - 2)
                        .style(Color::from_rgb(0.7, 0.7, 0.8)),
                ],
                row![
                    text(reason_text).size(font_size - 2).style(Color::from_rgb(0.6, 0.6, 0.7)),
                    Space::with_width(Length::Fill),
                    text(format!("~{} uses", suggestion.estimated_usage))
                        .size(font_size - 3)
                        .style(Color::from_rgb(0.5, 0.5, 0.6)),
                ],
                confidence_bar,
            ]
            .spacing(4);

            suggestions_column = suggestions_column.push(
                container(suggestion_item)
                    .style(theme::Container::Custom(Box::new(SuggestionItemStyle)))
                    .padding(12)
                    .width(Length::Fill)
            );
        }

        scrollable(suggestions_column)
            .height(Length::Fixed(300.0))
            .into()
    }

    /// Create footer with keyboard shortcuts
    fn create_footer(&self, font_size: u16) -> Element<Message> {
        let shortcuts = row![
            self.create_shortcut_hint("↑↓", "Navigate", font_size),
            Space::with_width(16),
            self.create_shortcut_hint("↵", "Execute", font_size),
            Space::with_width(16),
            self.create_shortcut_hint("Ctrl+B", "Bookmark", font_size),
            Space::with_width(16),
            self.create_shortcut_hint("Esc", "Close", font_size),
        ]
        .align_items(Alignment::Center);

        row![shortcuts, Space::with_width(Length::Fill)]
            .align_items(Alignment::Center)
            .padding([12, 20])
            .into()
    }

    /// Create keyboard shortcut hint
    fn create_shortcut_hint(&self, key: &str, description: &str, font_size: u16) -> Element<Message> {
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
}

impl Default for HistoryAnalytics {
    fn default() -> Self {
        Self {
            total_commands: 0,
            unique_commands: 0,
            success_rate: 0.0,
            avg_execution_time: Duration::from_millis(0),
            most_used_commands: Vec::new(),
            command_trends: HashMap::new(),
            daily_activity: Vec::new(),
            directory_usage: HashMap::new(),
            error_patterns: Vec::new(),
        }
    }
}

impl Default for CommandHistoryUI {
    fn default() -> Self {
        Self::new()
    }
}

// Custom styles for the history interface
struct HistoryPanelStyle;
struct CloseButtonStyle;
struct FilterButtonStyle;
struct TabButtonStyle;
struct SelectedResultStyle;
struct UnselectedResultStyle;
struct SuggestionItemStyle;
struct ShortcutKeyStyle;

impl iced::widget::container::StyleSheet for HistoryPanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
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

impl iced::widget::button::StyleSheet for CloseButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.4, 0.4),
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.8, 0.4, 0.4, 0.2))),
            border: Border {
                color: Color::from_rgb(0.8, 0.4, 0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Color::from_rgb(1.0, 0.5, 0.5),
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }
}

impl iced::widget::button::StyleSheet for FilterButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.6))),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.8),
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.8))),
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }
}

impl iced::widget::button::StyleSheet for TabButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.2, 0.8))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.9),
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.25, 0.9))),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.9, 0.9, 1.0),
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }
}

impl iced::widget::button::StyleSheet for SelectedResultStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
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

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }
}

impl iced::widget::button::StyleSheet for UnselectedResultStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.6))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }
}

impl iced::widget::container::StyleSheet for SuggestionItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.12, 0.18))),
            border: Border {
                color: Color::from_rgb(0.2, 0.3, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Some(Color::from_rgb(0.9, 0.9, 1.0)),
            shadow: Shadow::default(),
        }
    }
}

impl iced::widget::container::StyleSheet for ShortcutKeyStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
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
