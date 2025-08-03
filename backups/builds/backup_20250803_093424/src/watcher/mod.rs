//! File Watcher Module
//! 
//! Provides file system monitoring and change detection capabilities
//! for real-time updates and synchronization.

pub mod fs_watcher;
pub mod event_handler;
pub mod filters;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub watch_paths: Vec<PathBuf>,
    pub ignore_patterns: Vec<String>,
    pub recursive: bool,
    pub debounce_ms: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from(".")],
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".git/*".to_string(),
                "node_modules/*".to_string(),
                "target/*".to_string(),
            ],
            recursive: true,
            debounce_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchEvent {
    Created { path: PathBuf },
    Modified { path: PathBuf },
    Deleted { path: PathBuf },
    Renamed { from: PathBuf, to: PathBuf },
}

pub trait WatchEventHandler: Send + Sync {
    fn handle_event(&self, event: &WatchEvent);
}

pub struct FileWatcher {
    config: WatcherConfig,
    handlers: Vec<Box<dyn WatchEventHandler>>,
    watched_paths: HashSet<PathBuf>,
}

impl FileWatcher {
    pub fn new(config: WatcherConfig) -> Self {
        Self {
            config,
            handlers: Vec::new(),
            watched_paths: HashSet::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Box<dyn WatchEventHandler>) {
        self.handlers.push(handler);
    }

    pub fn add_watch_path(&mut self, path: PathBuf) {
        self.watched_paths.insert(path);
    }

    pub fn remove_watch_path(&mut self, path: &PathBuf) {
        self.watched_paths.remove(path);
    }

    pub fn start_watching(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation would go here
        // This is a placeholder for the actual file watching logic
        println!("Starting file watcher for paths: {:?}", self.watched_paths);
        Ok(())
    }

    fn notify_handlers(&self, event: &WatchEvent) {
        for handler in &self.handlers {
            handler.handle_event(event);
        }
    }
}
