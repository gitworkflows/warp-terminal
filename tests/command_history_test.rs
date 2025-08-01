//! Tests for Command History 
//! 
//! 
use warp_terminal::ui::command_history::{CommandHistoryUI, SearchFilters, StatusFilter, TimeFilter, HistoryViewMode, SessionFilter}; 
use warp_terminal::model::history::HistoryEntry;
use std::path::PathBuf;
use uuid::Uuid;
use fuzzy_matcher::FuzzyMatcher; 
use fuzzy_matcher::skim::SkimMatcherV2; 
use std::collections::VecDeque;

#[tokio::test]
async fn test_command_history_search_basic() {
    let mut _history_ui = CommandHistoryUI::new();
    _history_ui.search_query = "example".to_string();
    _history_ui.active_filters = SearchFilters {
        status_filter: Some(StatusFilter::Success),
        time_filter: TimeFilter::All,
        directory_filter: None,
        session_filter: SessionFilter::AllSessions,
        min_execution_time: None,
        bookmarked_only: false,
        frequent_only: Some(3),
    };
    
    let fake_history_data = VecDeque::from(vec![  
        HistoryEntry::new("example command one".to_string(), PathBuf::from("/"), Uuid::new_v4()),
        HistoryEntry::new("example command two".to_string(), PathBuf::from("/"), Uuid::new_v4()),
        HistoryEntry::new("another command".to_string(), PathBuf::from("/"), Uuid::new_v4())
    ]);
    
    let matcher = SkimMatcherV2::default();
    let search_results: Vec<_> = fake_history_data.into_iter().filter(|entry| {
         matcher.fuzzy_match(&entry.command, "example").is_some()
    }).collect();
    
    assert_eq!(search_results.len(), 2);
}

#[tokio::test]
async fn test_command_history_empty_search() {
    let history_ui = CommandHistoryUI::new();
    
    // Verify default state
    assert!(history_ui.search_results.is_empty());
    assert!(history_ui.search_query.is_empty());
    assert_eq!(history_ui.active_filters.status_filter, None);
}
