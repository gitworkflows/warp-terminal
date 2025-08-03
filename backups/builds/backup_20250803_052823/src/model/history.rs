use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a single command entry in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub command: String,
    pub exit_code: Option<i32>,
    pub directory: PathBuf,
    pub session_id: Uuid,
    pub timestamp: u64,
    pub execution_time: Option<Duration>,
    pub last_run: u64,
    pub run_count: u32,
    pub thread_id: String,
    pub tags: Vec<String>,
    pub bookmarked: bool,
}

impl HistoryEntry {
    pub fn new(command: String, directory: PathBuf, session_id: Uuid) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: Uuid::new_v4(),
            command,
            exit_code: None,
            directory,
            session_id,
            timestamp: now,
            execution_time: None,
            last_run: now,
            run_count: 1,
            thread_id: format!("{:?}", std::thread::current().id()),
            tags: Vec::new(),
            bookmarked: false,
        }
    }

    pub fn update_completion(&mut self, exit_code: i32, execution_time: Duration) {
        self.exit_code = Some(exit_code);
        self.execution_time = Some(execution_time);
        self.last_run = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.run_count += 1;
    }

    pub fn format_timestamp(&self) -> String {
        let dt = std::time::UNIX_EPOCH + std::time::Duration::from_secs(self.timestamp);
        format!("{:?}", dt) // Simple formatting for now
    }

    pub fn format_execution_time(&self) -> String {
        match self.execution_time {
            Some(duration) => format!("{:.2}ms", duration.as_millis()),
            None => "Running...".to_string(),
        }
    }

    pub fn is_successful(&self) -> bool {
        self.exit_code.unwrap_or(-1) == 0
    }
}

/// Manages command history with session isolation
#[derive(Debug, Clone)]
pub struct HistoryManager {
    /// Current session ID
    current_session: Uuid,
    /// History entries for each session (isolated)
    session_histories: HashMap<Uuid, VecDeque<HistoryEntry>>,
    /// Combined history for global search
    combined_history: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep per session
    max_entries_per_session: usize,
    /// Maximum number of entries in combined history
    max_combined_entries: usize,
    /// Current working directory
    current_directory: PathBuf,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            current_session: Uuid::new_v4(),
            session_histories: HashMap::new(),
            combined_history: VecDeque::new(),
            max_entries_per_session: 1000,
            max_combined_entries: 10000,
            current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }

    /// Add a new command to the current session's history
    pub fn add_command(&mut self, command: String) -> Uuid {
        let entry = HistoryEntry::new(command, self.current_directory.clone(), self.current_session);
        let entry_id = entry.id;

        // Add to current session
        let session_history = self.session_histories
            .entry(self.current_session)
            .or_insert_with(VecDeque::new);

        // Remove if already exists (to move to front)
        session_history.retain(|e| e.command != entry.command);
        session_history.push_front(entry.clone());

        // Trim if exceeds max size
        if session_history.len() > self.max_entries_per_session {
            session_history.truncate(self.max_entries_per_session);
        }

        // Add to combined history
        self.combined_history.retain(|e| e.command != entry.command);
        self.combined_history.push_front(entry);

        // Trim combined history
        if self.combined_history.len() > self.max_combined_entries {
            self.combined_history.truncate(self.max_combined_entries);
        }

        entry_id
    }

    /// Update a command entry when it completes
    pub fn update_command_completion(&mut self, entry_id: Uuid, exit_code: i32, execution_time: Duration) {
        // Update in current session
        if let Some(session_history) = self.session_histories.get_mut(&self.current_session) {
            if let Some(entry) = session_history.iter_mut().find(|e| e.id == entry_id) {
                entry.update_completion(exit_code, execution_time);
            }
        }

        // Update in combined history
        if let Some(entry) = self.combined_history.iter_mut().find(|e| e.id == entry_id) {
            entry.update_completion(exit_code, execution_time);
        }
    }

    /// Get history for the current session
    pub fn get_session_history(&self) -> Vec<&HistoryEntry> {
        self.session_histories
            .get(&self.current_session)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// Get combined history from all sessions
    pub fn get_combined_history(&self) -> Vec<&HistoryEntry> {
        self.combined_history.iter().collect()
    }

    /// Search history with prefix matching for UP arrow functionality
    pub fn search_with_prefix(&self, prefix: &str) -> Vec<&HistoryEntry> {
        self.get_session_history()
            .into_iter()
            .filter(|entry| entry.command.starts_with(prefix))
            .collect()
    }

    /// Fuzzy search across history
    pub fn fuzzy_search(&self, query: &str, include_all_sessions: bool) -> Vec<&HistoryEntry> {
        let history = if include_all_sessions {
            self.get_combined_history()
        } else {
            self.get_session_history()
        };

        let mut results: Vec<_> = history
            .into_iter()
            .filter(|entry| self.fuzzy_match(query, &entry.command))
            .collect();

        // Sort by relevance (simple scoring)
        results.sort_by(|a, b| {
            let score_a = self.calculate_relevance_score(query, a);
            let score_b = self.calculate_relevance_score(query, b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Get recent commands (last N commands from current session)
    pub fn get_recent_commands(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.get_session_history()
            .into_iter()
            .take(limit)
            .collect()
    }

    /// Get frequently used commands
    pub fn get_frequent_commands(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut commands: Vec<_> = self.get_combined_history();
        commands.sort_by(|a, b| b.run_count.cmp(&a.run_count));
        commands.into_iter().take(limit).collect()
    }

    /// Get commands filtered by success/failure
    pub fn get_commands_by_status(&self, successful: bool) -> Vec<&HistoryEntry> {
        self.get_combined_history()
            .into_iter()
            .filter(|entry| entry.is_successful() == successful)
            .collect()
    }

    /// Start a new session (isolates history)
    pub fn start_new_session(&mut self) {
        self.current_session = Uuid::new_v4();
    }

    /// Combine all session histories (called when closing)
    pub fn combine_sessions(&mut self) {
        // This is already maintained in combined_history, but we could
        // implement additional logic here for persistence
    }

    /// Set current working directory
    pub fn set_current_directory(&mut self, path: PathBuf) {
        self.current_directory = path;
    }

    /// Get current session ID
    pub fn current_session_id(&self) -> Uuid {
        self.current_session
    }

    /// Get all active session IDs
    pub fn get_active_sessions(&self) -> Vec<Uuid> {
        self.session_histories.keys().cloned().collect()
    }

    /// Toggle bookmark for a command
    pub fn toggle_bookmark(&mut self, entry_id: Uuid) {
        // Update in current session
        if let Some(session_history) = self.session_histories.get_mut(&self.current_session) {
            if let Some(entry) = session_history.iter_mut().find(|e| e.id == entry_id) {
                entry.bookmarked = !entry.bookmarked;
            }
        }

        // Update in combined history
        if let Some(entry) = self.combined_history.iter_mut().find(|e| e.id == entry_id) {
            entry.bookmarked = !entry.bookmarked;
        }
    }

    /// Get bookmarked commands
    pub fn get_bookmarked_commands(&self) -> Vec<&HistoryEntry> {
        self.get_combined_history()
            .into_iter()
            .filter(|entry| entry.bookmarked)
            .collect()
    }

    // Private helper methods
    
    fn fuzzy_match(&self, query: &str, text: &str) -> bool {
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();
        
        // Simple fuzzy matching - check if all characters in query appear in order
        let mut query_chars = query_lower.chars().peekable();
        
        for text_char in text_lower.chars() {
            if let Some(&query_char) = query_chars.peek() {
                if text_char == query_char {
                    query_chars.next();
                }
            }
        }
        
        query_chars.peek().is_none()
    }

    fn calculate_relevance_score(&self, query: &str, entry: &HistoryEntry) -> f32 {
        let mut score = 0.0;
        let query_lower = query.to_lowercase();
        let command_lower = entry.command.to_lowercase();

        // Exact match bonus
        if command_lower.contains(&query_lower) {
            score += 10.0;
            
            // Prefix match bonus
            if command_lower.starts_with(&query_lower) {
                score += 5.0;
            }
        }

        // Recency bonus (more recent commands score higher)
        let age_days = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - entry.last_run) / 86400;
        score += 1.0 / (age_days as f32 + 1.0);

        // Frequency bonus
        score += entry.run_count as f32 * 0.1;

        // Success rate bonus
        if entry.is_successful() {
            score += 2.0;
        }

        // Bookmark bonus
        if entry.bookmarked {
            score += 3.0;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_manager_creation() {
        let manager = HistoryManager::new();
        assert!(!manager.current_session.is_nil());
        assert!(manager.session_histories.is_empty());
        assert!(manager.combined_history.is_empty());
    }

    #[test]
    fn test_add_command() {
        let mut manager = HistoryManager::new();
        let entry_id = manager.add_command("ls -la".to_string());
        
        assert!(!entry_id.is_nil());
        assert_eq!(manager.get_session_history().len(), 1);
        assert_eq!(manager.get_combined_history().len(), 1);
        assert_eq!(manager.get_session_history()[0].command, "ls -la");
    }

    #[test]
    fn test_session_isolation() {
        let mut manager = HistoryManager::new();
        
        // Add command to first session
        manager.add_command("first command".to_string());
        assert_eq!(manager.get_session_history().len(), 1);
        
        // Start new session
        manager.start_new_session();
        assert_eq!(manager.get_session_history().len(), 0);
        
        // Add command to second session
        manager.add_command("second command".to_string());
        assert_eq!(manager.get_session_history().len(), 1);
        
        // But combined history has both
        assert_eq!(manager.get_combined_history().len(), 2);
    }

    #[test]
    fn test_prefix_search() {
        let mut manager = HistoryManager::new();
        manager.add_command("git status".to_string());
        manager.add_command("git commit".to_string());
        manager.add_command("ls -la".to_string());
        
        let results = manager.search_with_prefix("git");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.command.starts_with("git")));
    }

    #[test]
    fn test_fuzzy_search() {
        let mut manager = HistoryManager::new();
        manager.add_command("git status".to_string());
        manager.add_command("docker ps".to_string());
        manager.add_command("git commit".to_string());
        
        let results = manager.fuzzy_search("gst", false);
        assert!(results.len() > 0);
        assert!(results.iter().any(|e| e.command == "git status"));
    }
}
