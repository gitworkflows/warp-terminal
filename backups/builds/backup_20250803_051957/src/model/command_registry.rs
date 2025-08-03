use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandCategory {
    Pane,
    Settings,
    Theme,
    History,
    Search,
    Custom,
    Workflow,
}

impl CommandCategory {
    /// Get all available categories
    pub fn all() -> Vec<CommandCategory> {
        vec![
            CommandCategory::Pane,
            CommandCategory::Settings,
            CommandCategory::Theme,
            CommandCategory::History,
            CommandCategory::Search,
            CommandCategory::Custom,
            CommandCategory::Workflow,
        ]
    }

    /// Get display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            CommandCategory::Pane => "Pane Management",
            CommandCategory::Settings => "Settings",
            CommandCategory::Theme => "Themes",
            CommandCategory::History => "Command History",
            CommandCategory::Search => "Search",
            CommandCategory::Custom => "Custom Commands",
            CommandCategory::Workflow => "Workflows",
        }
    }

    /// Get icon for the category
    pub fn icon(&self) -> &'static str {
        match self {
            CommandCategory::Pane => "⚡",
            CommandCategory::Settings => "⚙️",
            CommandCategory::Theme => "🎨",
            CommandCategory::History => "📚",
            CommandCategory::Search => "🔍",
            CommandCategory::Custom => "🔧",
            CommandCategory::Workflow => "🚀",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: CommandCategory,
    pub shortcut: String,
    pub keywords: Vec<String>,
    pub enabled: bool,
    pub priority: u8, // 0-255, higher = more priority
}

impl Command {
    /// Create a new command with validation
    pub fn new(
        id: String,
        title: String,
        description: String,
        category: CommandCategory,
        shortcut: String,
        keywords: Vec<String>,
    ) -> Result<Self, String> {
        if id.is_empty() {
            return Err("Command ID cannot be empty".to_string());
        }
        if title.is_empty() {
            return Err("Command title cannot be empty".to_string());
        }
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
        {
            return Err("Command ID contains invalid characters".to_string());
        }

        Ok(Command {
            id,
            title,
            description,
            category,
            shortcut,
            keywords,
            enabled: true,
            priority: 100, // Default medium priority
        })
    }

    /// Create a command builder for easier construction
    pub fn builder(id: &str, title: &str) -> CommandBuilder {
        CommandBuilder::new(id, title)
    }

    /// Check if command matches a query string
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        self.title.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self
                .keywords
                .iter()
                .any(|k| k.to_lowercase().contains(&query_lower))
            || format!("{:?}", self.category)
                .to_lowercase()
                .contains(&query_lower)
    }
}

/// Builder pattern for creating commands
#[derive(Debug)]
pub struct CommandBuilder {
    id: String,
    title: String,
    description: String,
    category: CommandCategory,
    shortcut: String,
    keywords: Vec<String>,
    enabled: bool,
    priority: u8,
}

impl CommandBuilder {
    fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: String::new(),
            category: CommandCategory::Custom,
            shortcut: String::new(),
            keywords: Vec::new(),
            enabled: true,
            priority: 100,
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn category(mut self, category: CommandCategory) -> Self {
        self.category = category;
        self
    }

    pub fn shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = shortcut.to_string();
        self
    }

    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn keyword(mut self, keyword: &str) -> Self {
        self.keywords.push(keyword.to_string());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> Result<Command, String> {
        Command::new(
            self.id,
            self.title,
            self.description,
            self.category,
            self.shortcut,
            self.keywords,
        )
        .map(|mut cmd| {
            cmd.enabled = self.enabled;
            cmd.priority = self.priority;
            cmd
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    categories: HashMap<CommandCategory, Vec<String>>,
    aliases: HashMap<String, String>,      // alias -> command_id
    execution_count: HashMap<String, u32>, // command_id -> count
    #[serde(skip)]
    fuzzy_matcher: Option<SkimMatcherV2>, // Not serialized, not cloned
}

impl std::fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistry")
            .field("commands", &self.commands)
            .field("categories", &self.categories)
            .field("aliases", &self.aliases)
            .field("execution_count", &self.execution_count)
            .finish()
    }
}

impl Clone for CommandRegistry {
    fn clone(&self) -> Self {
        CommandRegistry {
            commands: self.commands.clone(),
            categories: self.categories.clone(),
            aliases: self.aliases.clone(),
            execution_count: self.execution_count.clone(),
            fuzzy_matcher: None, // Do not clone
        }
    }
}

/// Search result with detailed match information
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub command: Command,
    pub score: f64,
    pub match_positions: Vec<usize>,
    pub match_field: MatchField,
}

/// Field that matched during search
#[derive(Debug, Clone, PartialEq)]
pub enum MatchField {
    Title,
    Description,
    Keyword,
    Category,
    Alias,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            categories: HashMap::new(),
            aliases: HashMap::new(),
            execution_count: HashMap::new(),
            fuzzy_matcher: Some(SkimMatcherV2::default()),
        }
    }

    /// Create registry with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: HashMap::with_capacity(capacity),
            categories: HashMap::new(),
            aliases: HashMap::new(),
            execution_count: HashMap::new(),
            fuzzy_matcher: Some(SkimMatcherV2::default()),
        }
    }

    pub fn register(&mut self, command: Command) -> Result<(), String> {
        // Validate command
        if self.commands.contains_key(&command.id) {
            return Err(format!("Command with ID '{}' already exists", command.id));
        }

        let category = command.category.clone();
        let id = command.id.clone();

        self.commands.insert(id.clone(), command);

        // Update category index
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(id.clone());

        debug!("Registered command: {}", id);
        Ok(())
    }

    /// Register a command, replacing existing if present
    pub fn register_or_replace(&mut self, command: Command) {
        let category = command.category.clone();
        let id = command.id.clone();

        // Remove from old category if exists
        if let Some(old_command) = self.commands.get(&id) {
            if let Some(old_category_commands) = self.categories.get_mut(&old_command.category) {
                old_category_commands.retain(|cid| cid != &id);
            }
        }

        self.commands.insert(id.clone(), command);

        // Update category index
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(id.clone());

        debug!("Registered/replaced command: {}", id);
    }

    /// Add an alias for a command
    pub fn add_alias(&mut self, alias: String, command_id: String) -> Result<(), String> {
        if !self.commands.contains_key(&command_id) {
            return Err(format!("Command '{}' does not exist", command_id));
        }
        if self.aliases.contains_key(&alias) {
            return Err(format!("Alias '{}' already exists", alias));
        }

        self.aliases.insert(alias.clone(), command_id.clone());
        debug!("Added alias '{}' for command '{}'", alias, command_id);
        Ok(())
    }

    /// Remove an alias
    pub fn remove_alias(&mut self, alias: &str) -> Option<String> {
        let result = self.aliases.remove(alias);
        if result.is_some() {
            debug!("Removed alias: {}", alias);
        }
        result
    }

    pub fn unregister(&mut self, command_id: &str) -> Option<Command> {
        if let Some(command) = self.commands.remove(command_id) {
            // Remove from category index
            if let Some(category_commands) = self.categories.get_mut(&command.category) {
                category_commands.retain(|id| id != command_id);
            }
            Some(command)
        } else {
            None
        }
    }

    pub fn get(&self, command_id: &str) -> Option<&Command> {
        self.commands.get(command_id)
    }

    /// Get command by alias
    pub fn get_by_alias(&self, alias: &str) -> Option<&Command> {
        if let Some(command_id) = self.aliases.get(alias) {
            self.commands.get(command_id)
        } else {
            None
        }
    }

    pub fn search(&self, query: &str) -> Vec<(Command, Option<Vec<usize>>, &'static str)> {
        let advanced_results = self.advanced_search(query);

        // Convert to legacy format for backward compatibility
        advanced_results
            .into_iter()
            .map(|result| {
                let field_str = match result.match_field {
                    MatchField::Title => "title",
                    MatchField::Description => "description",
                    MatchField::Keyword => "keyword",
                    MatchField::Category => "category",
                    MatchField::Alias => "alias",
                };

                (result.command, Some(result.match_positions), field_str)
            })
            .collect()
    }

    /// Advanced search with detailed results
    pub fn advanced_search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let matcher = self.fuzzy_matcher.as_ref().unwrap_or_else(|| {
            static DEFAULT_MATCHER: std::sync::LazyLock<SkimMatcherV2> = std::sync::LazyLock::new(|| SkimMatcherV2::default());
            &DEFAULT_MATCHER
        });
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Search through commands
        for command in self.commands.values() {
            if !command.enabled {
                continue;
            }

            if let Some(search_result) = self.score_command(command, &query_lower, matcher) {
                results.push(search_result);
            }
        }

        // Search through aliases
        for (alias, command_id) in &self.aliases {
            if let Some(command) = self.commands.get(command_id) {
                if !command.enabled {
                    continue;
                }

                if let Some((score, positions)) = matcher.fuzzy_indices(alias, &query_lower) {
                    results.push(SearchResult {
                        command: command.clone(),
                        score: score as f64 * 0.8, // Slightly lower score for alias matches
                        match_positions: positions,
                        match_field: MatchField::Alias,
                    });
                }
            }
        }

        // Sort by score (highest first), then by execution count, then by priority
        results.sort_by(|a, b| {
            let score_cmp = b
                .score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal);
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }

            let a_count = self.execution_count.get(&a.command.id).unwrap_or(&0);
            let b_count = self.execution_count.get(&b.command.id).unwrap_or(&0);
            let count_cmp = b_count.cmp(a_count);
            if count_cmp != std::cmp::Ordering::Equal {
                return count_cmp;
            }

            b.command.priority.cmp(&a.command.priority)
        });

        results
    }

    /// Score a single command against a query
    fn score_command(
        &self,
        command: &Command,
        query: &str,
        matcher: &SkimMatcherV2,
    ) -> Option<SearchResult> {
        let mut best_score = 0.0;
        let mut best_positions = Vec::new();
        let mut best_field = MatchField::Title;

        // Score title match
        if let Some((score, positions)) =
            matcher.fuzzy_indices(&command.title.to_lowercase(), query)
        {
            let weighted_score = score as f64 * 3.0; // Title matches are most important
            if weighted_score > best_score {
                best_score = weighted_score;
                best_positions = positions;
                best_field = MatchField::Title;
            }
        }

        // Score description match
        if !command.description.is_empty() {
            if let Some((score, positions)) =
                matcher.fuzzy_indices(&command.description.to_lowercase(), query)
            {
                let weighted_score = score as f64 * 1.5;
                if weighted_score > best_score {
                    best_score = weighted_score;
                    best_positions = positions;
                    best_field = MatchField::Description;
                }
            }
        }

        // Score keyword matches
        for keyword in &command.keywords {
            if let Some((score, positions)) = matcher.fuzzy_indices(&keyword.to_lowercase(), query)
            {
                let weighted_score = score as f64 * 2.0;
                if weighted_score > best_score {
                    best_score = weighted_score;
                    best_positions = positions;
                    best_field = MatchField::Keyword;
                }
            }
        }

        // Score category match
        let category_str = format!("{:?}", command.category).to_lowercase();
        if category_str.contains(query) {
            let score = 100.0; // Fixed score for exact category name matches
            if score > best_score {
                best_score = score;
                best_positions = Vec::new(); // No specific positions for category matches
                best_field = MatchField::Category;
            }
        }

        // Apply priority and execution count boosts
        if best_score > 0.0 {
            let priority_boost = (command.priority as f64 / 255.0) * 0.2; // Up to 20% boost
            let execution_count = self.execution_count.get(&command.id).unwrap_or(&0);
            let popularity_boost = (*execution_count as f64 * 0.1).min(0.5); // Up to 50% boost

            best_score *= 1.0 + priority_boost + popularity_boost;

            Some(SearchResult {
                command: command.clone(),
                score: best_score,
                match_positions: best_positions,
                match_field: best_field,
            })
        } else {
            None
        }
    }

    pub fn get_by_category(&self, category: CommandCategory) -> Vec<Command> {
        if let Some(command_ids) = self.categories.get(&category) {
            command_ids
                .iter()
                .filter_map(|id| self.commands.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_recent_commands(&self, recent_ids: &[String]) -> Vec<Command> {
        recent_ids
            .iter()
            .filter_map(|id| self.commands.get(id))
            .cloned()
            .collect()
    }

    pub fn get_all_commands(&self) -> Vec<Command> {
        self.commands.values().cloned().collect()
    }

    pub fn get_categories(&self) -> Vec<CommandCategory> {
        self.categories.keys().cloned().collect()
    }

    /// Record execution of a command
    pub fn record_execution(&mut self, command_id: &str) {
        *self
            .execution_count
            .entry(command_id.to_string())
            .or_insert(0) += 1;
        debug!("Recorded execution for command: {}", command_id);
    }

    /// Get execution count for a command
    pub fn get_execution_count(&self, command_id: &str) -> u32 {
        *self.execution_count.get(command_id).unwrap_or(&0)
    }

    /// Get most popular commands
    pub fn get_popular_commands(&self, limit: usize) -> Vec<Command> {
        let mut commands_with_count: Vec<_> = self
            .commands
            .values()
            .filter(|cmd| cmd.enabled)
            .map(|cmd| {
                let count = self.execution_count.get(&cmd.id).unwrap_or(&0);
                (cmd.clone(), *count)
            })
            .collect();

        commands_with_count.sort_by(|a, b| b.1.cmp(&a.1));

        commands_with_count
            .into_iter()
            .take(limit)
            .map(|(cmd, _)| cmd)
            .collect()
    }

    /// Enable or disable a command
    pub fn set_command_enabled(&mut self, command_id: &str, enabled: bool) -> Result<(), String> {
        if let Some(command) = self.commands.get_mut(command_id) {
            command.enabled = enabled;
            debug!("Set command '{}' enabled: {}", command_id, enabled);
            Ok(())
        } else {
            Err(format!("Command '{}' not found", command_id))
        }
    }

    /// Set command priority
    pub fn set_command_priority(&mut self, command_id: &str, priority: u8) -> Result<(), String> {
        if let Some(command) = self.commands.get_mut(command_id) {
            command.priority = priority;
            debug!("Set command '{}' priority: {}", command_id, priority);
            Ok(())
        } else {
            Err(format!("Command '{}' not found", command_id))
        }
    }

    /// Get commands by priority range
    pub fn get_commands_by_priority(&self, min_priority: u8, max_priority: u8) -> Vec<Command> {
        self.commands
            .values()
            .filter(|cmd| {
                cmd.enabled && cmd.priority >= min_priority && cmd.priority <= max_priority
            })
            .cloned()
            .collect()
    }

    /// Get registry statistics
    pub fn get_statistics(&self) -> CommandRegistryStats {
        let enabled_count = self.commands.values().filter(|cmd| cmd.enabled).count();
        let total_executions = self.execution_count.values().sum();

        CommandRegistryStats {
            total_commands: self.commands.len(),
            enabled_commands: enabled_count,
            disabled_commands: self.commands.len() - enabled_count,
            categories_count: self.categories.len(),
            aliases_count: self.aliases.len(),
            total_executions,
        }
    }

    /// Validate all commands in the registry
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for command in self.commands.values() {
            if command.id.is_empty() {
                errors.push(format!("Command has empty ID: '{}'", command.title));
            }
            if command.title.is_empty() {
                errors.push(format!("Command '{}' has empty title", command.id));
            }
        }

        // Check for orphaned aliases
        for (alias, command_id) in &self.aliases {
            if !self.commands.contains_key(command_id) {
                errors.push(format!(
                    "Alias '{}' points to non-existent command '{}'",
                    alias, command_id
                ));
            }
        }

        errors
    }
}

/// Statistics about the command registry
#[derive(Debug, Clone)]
pub struct CommandRegistryStats {
    pub total_commands: usize,
    pub enabled_commands: usize,
    pub disabled_commands: usize,
    pub categories_count: usize,
    pub aliases_count: usize,
    pub total_executions: u32,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_command() {
        let mut registry = CommandRegistry::new();

        let command = Command::builder("test.command", "Test Command")
            .description("A test command")
            .category(CommandCategory::Custom)
            .shortcut("Ctrl+T")
            .keyword("test")
            .build()
            .unwrap();

        registry.register(command.clone()).unwrap();

        let retrieved = registry.get("test.command").unwrap();
        assert_eq!(retrieved.id, command.id);
        assert_eq!(retrieved.title, command.title);
    }

    #[test]
    fn test_search_functionality() {
        let mut registry = CommandRegistry::new();

        registry
            .register(
                Command::builder("pane.split", "Split Pane")
                    .description("Split the current pane")
                    .category(CommandCategory::Pane)
                    .keywords(vec!["split".to_string(), "pane".to_string()])
                    .build()
                    .unwrap(),
            )
            .unwrap();

        registry
            .register(
                Command::builder("settings.open", "Open Settings")
                    .description("Open the settings panel")
                    .category(CommandCategory::Settings)
                    .keywords(vec!["settings".to_string(), "config".to_string()])
                    .build()
                    .unwrap(),
            )
            .unwrap();

        let results = registry.search("split");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "pane.split");

        let results = registry.search("open");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "settings.open");
    }

    #[test]
    fn test_aliases() {
        let mut registry = CommandRegistry::new();

        let command = Command::builder("test.cmd", "Test Command")
            .description("A test command")
            .category(CommandCategory::Custom)
            .build()
            .unwrap();

        registry.register(command).unwrap();
        registry
            .add_alias("tc".to_string(), "test.cmd".to_string())
            .unwrap();

        // Should be able to get command by alias
        let aliased_command = registry.get_by_alias("tc").unwrap();
        assert_eq!(aliased_command.id, "test.cmd");
    }
}
