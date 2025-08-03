use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    sessions: HashMap<String, SessionHistory>,
    current_session_id: String,
    max_history_size: usize,
    search_index: HashMap<String, Vec<usize>>, // Word -> command indices
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    pub session_id: String,
    pub commands: VecDeque<HistoryEntry>,
    pub created_at: u64,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: usize,
    pub command: String,
    pub timestamp: u64,
    pub duration: Option<u64>,
    pub exit_code: Option<i32>,
    pub working_directory: PathBuf,
    pub session_id: String,
    pub tags: Vec<String>,  // User-defined tags
    pub frequency: u32,     // How often this command is used
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: HistoryEntry,
    pub relevance_score: f32,
    pub match_type: MatchType,
}

#[derive(Debug, Clone)]
pub enum MatchType {
    ExactMatch,
    PrefixMatch,
    SubstringMatch,
    FuzzyMatch,
    TagMatch,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            current_session_id: Self::generate_session_id(),
            max_history_size: 10000,
            search_index: HashMap::new(),
        }
    }

    pub fn add_command(
        &mut self,
        command: String,
        exit_code: Option<i32>,
        duration: Option<u64>,
        working_directory: PathBuf,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = self.generate_entry_id();
        let current_session_id = self.current_session_id.clone();
        let max_history_size = self.max_history_size;
        
        let session = self.get_or_create_current_session();

        // Check if this is a duplicate of the last command
        let is_duplicate = session.commands.back()
            .map_or(false, |last| last.command == command);

        if is_duplicate {
            // Update the existing entry's frequency
            if let Some(last_entry) = session.commands.back_mut() {
                last_entry.frequency += 1;
                last_entry.timestamp = timestamp;
            }
        } else {
            let entry = HistoryEntry {
                id,
                command: command.clone(),
                timestamp,
                duration,
                exit_code,
                working_directory,
                session_id: current_session_id,
                tags: Vec::new(),
                frequency: 1,
            };

            session.commands.push_back(entry.clone());
            
            // Limit history size
            if session.commands.len() > max_history_size {
                if let Some(removed) = session.commands.pop_front() {
                    self.remove_from_search_index(&removed);
                }
            }
            
            self.update_search_index(&entry);
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for session in self.sessions.values() {
            for entry in &session.commands {
                if let Some(result) = self.calculate_match_score(entry, &query_lower) {
                    results.push(result);
                }
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(limit);

        results
    }

    pub fn search_in_session(&self, session_id: &str, query: &str, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        if let Some(session) = self.sessions.get(session_id) {
            for entry in &session.commands {
                if let Some(result) = self.calculate_match_score(entry, &query_lower) {
                    results.push(result);
                }
            }
        }

        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(limit);

        results
    }

    pub fn get_recent_commands(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut all_commands: Vec<&HistoryEntry> = Vec::new();

        for session in self.sessions.values() {
            all_commands.extend(session.commands.iter());
        }

        all_commands.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_commands.truncate(limit);

        all_commands
    }

    pub fn get_most_used_commands(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut all_commands: Vec<&HistoryEntry> = Vec::new();

        for session in self.sessions.values() {
            all_commands.extend(session.commands.iter());
        }

        all_commands.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        all_commands.truncate(limit);

        all_commands
    }

    pub fn get_session_commands(&self, session_id: &str) -> Vec<&HistoryEntry> {
        if let Some(session) = self.sessions.get(session_id) {
            session.commands.iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn new_session(&mut self, working_directory: PathBuf) -> String {
        let session_id = Self::generate_session_id();
        let session = SessionHistory {
            session_id: session_id.clone(),
            commands: VecDeque::new(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            working_directory,
            environment: std::env::vars().collect(),
        };

        self.sessions.insert(session_id.clone(), session);
        self.current_session_id = session_id.clone();

        session_id
    }

    pub fn switch_session(&mut self, session_id: &str) -> bool {
        if self.sessions.contains_key(session_id) {
            self.current_session_id = session_id.to_string();
            true
        } else {
            false
        }
    }

    pub fn get_current_session_id(&self) -> &str {
        &self.current_session_id
    }

    pub fn get_all_sessions(&self) -> Vec<&SessionHistory> {
        self.sessions.values().collect()
    }

    pub fn delete_session(&mut self, session_id: &str) -> bool {
        if session_id == self.current_session_id {
            return false; // Can't delete current session
        }

        if let Some(session) = self.sessions.remove(session_id) {
            // Remove from search index
            for entry in &session.commands {
                self.remove_from_search_index(entry);
            }
            true
        } else {
            false
        }
    }

    pub fn add_tag_to_command(&mut self, command_id: usize, tag: String) {
        for session in self.sessions.values_mut() {
            for entry in &mut session.commands {
                if entry.id == command_id {
                    if !entry.tags.contains(&tag) {
                        entry.tags.push(tag);
                    }
                    return;
                }
            }
        }
    }

    pub fn remove_tag_from_command(&mut self, command_id: usize, tag: &str) {
        for session in self.sessions.values_mut() {
            for entry in &mut session.commands {
                if entry.id == command_id {
                    entry.tags.retain(|t| t != tag);
                    return;
                }
            }
        }
    }

    pub fn get_commands_by_tag(&self, tag: &str) -> Vec<&HistoryEntry> {
        let mut commands = Vec::new();
        
        for session in self.sessions.values() {
            for entry in &session.commands {
                if entry.tags.contains(&tag.to_string()) {
                    commands.push(entry);
                }
            }
        }

        commands.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        commands
    }

    pub fn get_command_statistics(&self) -> HistoryStatistics {
        let mut total_commands = 0;
        let mut unique_commands = std::collections::HashSet::new();
        let mut total_sessions = self.sessions.len();
        let mut commands_by_hour = HashMap::new();
        let mut exit_codes = HashMap::new();

        for session in self.sessions.values() {
            for entry in &session.commands {
                total_commands += 1;
                unique_commands.insert(entry.command.clone());

                // Hour of day analysis
                let hour = (entry.timestamp / 3600) % 24;
                *commands_by_hour.entry(hour).or_insert(0) += 1;

                // Exit code analysis
                if let Some(code) = entry.exit_code {
                    *exit_codes.entry(code).or_insert(0) += 1;
                }
            }
        }

        HistoryStatistics {
            total_commands,
            unique_commands: unique_commands.len(),
            total_sessions,
            commands_by_hour,
            exit_codes,
        }
    }

    pub fn export_history(&self, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(self)
                    .map_err(|e| format!("JSON export failed: {}", e))
            },
            ExportFormat::Csv => {
                let mut csv = String::from("timestamp,command,exit_code,duration,session_id,working_directory\n");
                
                for session in self.sessions.values() {
                    for entry in &session.commands {
                        csv.push_str(&format!(
                            "{},{},{},{},{},{}\n",
                            entry.timestamp,
                            entry.command.replace(',', "\\,"),
                            entry.exit_code.unwrap_or(-1),
                            entry.duration.unwrap_or(0),
                            entry.session_id,
                            entry.working_directory.display()
                        ));
                    }
                }
                
                Ok(csv)
            },
            ExportFormat::PlainText => {
                let mut text = String::new();
                
                for session in self.sessions.values() {
                    text.push_str(&format!("=== Session {} ===\n", session.session_id));
                    for entry in &session.commands {
                        text.push_str(&format!("{}\n", entry.command));
                    }
                    text.push('\n');
                }
                
                Ok(text)
            }
        }
    }

    // Private methods

    fn get_or_create_current_session(&mut self) -> &mut SessionHistory {
        if !self.sessions.contains_key(&self.current_session_id) {
            let session = SessionHistory {
                session_id: self.current_session_id.clone(),
                commands: VecDeque::new(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                working_directory: std::env::current_dir().unwrap_or_default(),
                environment: std::env::vars().collect(),
            };
            self.sessions.insert(self.current_session_id.clone(), session);
        }

        self.sessions.get_mut(&self.current_session_id).unwrap()
    }

    fn generate_session_id() -> String {
        format!("session_{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }

    fn generate_entry_id(&self) -> usize {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize
    }

    fn update_search_index(&mut self, entry: &HistoryEntry) {
        let words: Vec<&str> = entry.command.split_whitespace().collect();
        
        for word in words {
            let word_lower = word.to_lowercase();
            self.search_index.entry(word_lower)
                .or_insert_with(Vec::new)
                .push(entry.id);
        }
    }

    fn remove_from_search_index(&mut self, entry: &HistoryEntry) {
        let words: Vec<&str> = entry.command.split_whitespace().collect();
        
        for word in words {
            let word_lower = word.to_lowercase();
            if let Some(indices) = self.search_index.get_mut(&word_lower) {
                indices.retain(|&id| id != entry.id);
                if indices.is_empty() {
                    self.search_index.remove(&word_lower);
                }
            }
        }
    }

    fn calculate_match_score(&self, entry: &HistoryEntry, query: &str) -> Option<SearchResult> {
        let command_lower = entry.command.to_lowercase();
        
        // Exact match
        if command_lower == query {
            return Some(SearchResult {
                entry: entry.clone(),
                relevance_score: 1.0,
                match_type: MatchType::ExactMatch,
            });
        }

        // Prefix match
        if command_lower.starts_with(query) {
            return Some(SearchResult {
                entry: entry.clone(),
                relevance_score: 0.9,
                match_type: MatchType::PrefixMatch,
            });
        }

        // Substring match
        if command_lower.contains(query) {
            return Some(SearchResult {
                entry: entry.clone(),
                relevance_score: 0.7,
                match_type: MatchType::SubstringMatch,
            });
        }

        // Tag match
        for tag in &entry.tags {
            if tag.to_lowercase().contains(query) {
                return Some(SearchResult {
                    entry: entry.clone(),
                    relevance_score: 0.6,
                    match_type: MatchType::TagMatch,
                });
            }
        }

        // Fuzzy match
        let similarity = self.calculate_fuzzy_similarity(&command_lower, query);
        if similarity > 0.5 {
            return Some(SearchResult {
                entry: entry.clone(),
                relevance_score: similarity * 0.5,
                match_type: MatchType::FuzzyMatch,
            });
        }

        None
    }

    fn calculate_fuzzy_similarity(&self, s1: &str, s2: &str) -> f32 {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let distance = self.levenshtein_distance(s1, s2);
        let max_len = len1.max(len2);
        
        1.0 - (distance as f32 / max_len as f32)
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }

        matrix[len1][len2]
    }
}

#[derive(Debug, Clone)]
pub struct HistoryStatistics {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub total_sessions: usize,
    pub commands_by_hour: HashMap<u64, usize>,
    pub exit_codes: HashMap<i32, usize>,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    PlainText,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new();
        let cwd = PathBuf::from("/test");
        
        history.add_command("ls -la".to_string(), Some(0), Some(100), cwd.clone());
        
        let recent = history.get_recent_commands(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].command, "ls -la");
    }

    #[test]
    fn test_search() {
        let mut history = CommandHistory::new();
        let cwd = PathBuf::from("/test");
        
        history.add_command("git status".to_string(), Some(0), Some(100), cwd.clone());
        history.add_command("git commit -m test".to_string(), Some(0), Some(200), cwd.clone());
        history.add_command("ls -la".to_string(), Some(0), Some(50), cwd.clone());
        
        let results = history.search("git", 10);
        assert_eq!(results.len(), 2);
        assert!(results[0].entry.command.contains("git"));
    }

    #[test]
    fn test_session_isolation() {
        let mut history = CommandHistory::new();
        let cwd = PathBuf::from("/test");
        
        // Add command to current session
        history.add_command("command1".to_string(), Some(0), Some(100), cwd.clone());
        
        // Create new session
        let new_session = history.new_session(cwd.clone());
        history.add_command("command2".to_string(), Some(0), Some(100), cwd.clone());
        
        // Check session isolation
        let current_commands = history.get_session_commands(&new_session);
        assert_eq!(current_commands.len(), 1);
        assert_eq!(current_commands[0].command, "command2");
    }

    #[test]
    fn test_tagging() {
        let mut history = CommandHistory::new();
        let cwd = PathBuf::from("/test");
        
        history.add_command("git status".to_string(), Some(0), Some(100), cwd.clone());
        
        let recent = history.get_recent_commands(1);
        let command_id = recent[0].id;
        
        history.add_tag_to_command(command_id, "git".to_string());
        
        let tagged_commands = history.get_commands_by_tag("git");
        assert_eq!(tagged_commands.len(), 1);
        assert_eq!(tagged_commands[0].command, "git status");
    }
}
