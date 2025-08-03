//! Synchronization model for managing synchronized inputs across multiple panes/sessions.
//!
//! This module provides the core logic for Warp's Synchronized Inputs feature,
//! allowing users to type commands once and have them sync to multiple terminal sessions.

use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Defines the scope of input synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SynchronizationScope {
    /// Synchronize all panes in the current tab only
    CurrentTab,
    /// Synchronize all panes across all tabs
    AllTabs,
    /// No synchronization active
    None,
}

impl Default for SynchronizationScope {
    fn default() -> Self {
        Self::None
    }
}

impl SynchronizationScope {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::CurrentTab | Self::AllTabs)
    }
}

/// Represents a terminal pane that can participate in synchronization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneInfo {
    pub pane_id: Uuid,
    pub tab_id: Uuid,
    pub is_active: bool,
    pub editor_type: EditorType,
}

/// Different editor types that can be synchronized
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorType {
    /// Standard shell input
    Shell,
    /// Vim editor mode
    Vim,
    /// Emacs editor mode
    Emacs,
    /// Other editor types
    Other(u8),
}

impl Default for EditorType {
    fn default() -> Self {
        Self::Shell
    }
}

/// Manages the state of input synchronization across panes
#[derive(Debug, Clone)]
pub struct SynchronizationManager {
    /// Current synchronization scope
    pub scope: SynchronizationScope,
    /// All registered panes
    registered_panes: HashMap<Uuid, PaneInfo>,
    /// Currently active pane
    active_pane_id: Option<Uuid>,
    /// Current synchronized input
    synchronized_input: String,
    /// Whether synchronization is temporarily paused
    is_paused: bool,
}

impl Default for SynchronizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SynchronizationManager {
    /// Create a new synchronization manager
    pub fn new() -> Self {
        Self {
            scope: SynchronizationScope::None,
            registered_panes: HashMap::new(),
            active_pane_id: None,
            synchronized_input: String::new(),
            is_paused: false,
        }
    }

    /// Register a new pane for potential synchronization
    pub fn register_pane(&mut self, pane_info: PaneInfo) {
        self.registered_panes.insert(pane_info.pane_id, pane_info);
    }

    /// Unregister a pane from synchronization
    pub fn unregister_pane(&mut self, pane_id: Uuid) {
        self.registered_panes.remove(&pane_id);
        if self.active_pane_id == Some(pane_id) {
            self.active_pane_id = None;
        }
    }

    /// Set the active pane (the one receiving direct input)
    pub fn set_active_pane(&mut self, pane_id: Uuid) {
        if self.registered_panes.contains_key(&pane_id) {
            // Update active status
            for pane in self.registered_panes.values_mut() {
                pane.is_active = pane.pane_id == pane_id;
            }
            self.active_pane_id = Some(pane_id);
        }
    }

    /// Start synchronization with the specified scope
    pub fn start_synchronization(&mut self, scope: SynchronizationScope) {
        self.scope = scope;
        self.is_paused = false;
    }

    /// Stop all synchronization
    pub fn stop_synchronization(&mut self) {
        self.scope = SynchronizationScope::None;
        self.synchronized_input.clear();
        self.is_paused = false;
    }

    /// Toggle synchronization scope (cycles through None -> CurrentTab -> AllTabs -> None)
    pub fn toggle_synchronization(&mut self) {
        self.scope = match self.scope {
            SynchronizationScope::None => SynchronizationScope::CurrentTab,
            SynchronizationScope::CurrentTab => SynchronizationScope::AllTabs,
            SynchronizationScope::AllTabs => SynchronizationScope::None,
        };
        
        if !self.scope.is_active() {
            self.synchronized_input.clear();
        }
        self.is_paused = false;
    }

    /// Pause synchronization temporarily without changing scope
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume synchronization
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Toggle synchronization pause state
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Update the synchronized input
    pub fn update_input(&mut self, input: String) -> Vec<Uuid> {
        if !self.should_synchronize() {
            return vec![];
        }

        self.synchronized_input = input;
        self.get_target_panes()
    }

    /// Get the current synchronized input
    pub fn get_synchronized_input(&self) -> &str {
        &self.synchronized_input
    }

    /// Check if synchronization should occur
    pub fn should_synchronize(&self) -> bool {
        self.scope.is_active() && !self.is_paused && self.active_pane_id.is_some()
    }

    /// Get panes that should receive synchronized input
    pub fn get_target_panes(&self) -> Vec<Uuid> {
        if !self.should_synchronize() {
            return vec![];
        }

        let active_pane = match self.active_pane_id.and_then(|id| self.registered_panes.get(&id)) {
            Some(pane) => pane,
            None => return vec![],
        };

        let mut target_panes = Vec::new();

        for pane in self.registered_panes.values() {
            // Don't sync to the active pane (it's already receiving direct input)
            if pane.pane_id == active_pane.pane_id {
                continue;
            }

            // Only sync to panes with the same editor type
            if pane.editor_type != active_pane.editor_type {
                continue;
            }

            // Apply scope filtering
            let should_include = match self.scope {
                SynchronizationScope::CurrentTab => pane.tab_id == active_pane.tab_id,
                SynchronizationScope::AllTabs => true,
                SynchronizationScope::None => false,
            };

            if should_include {
                target_panes.push(pane.pane_id);
            }
        }

        target_panes
    }

    /// Get synchronization status for UI display
    pub fn get_status(&self) -> SynchronizationStatus {
        let target_count = if self.should_synchronize() {
            self.get_target_panes().len()
        } else {
            0
        };

        SynchronizationStatus {
            scope: self.scope,
            is_active: self.should_synchronize(),
            is_paused: self.is_paused,
            target_pane_count: target_count,
            active_pane_id: self.active_pane_id,
        }
    }

    /// Get all registered panes
    pub fn get_all_panes(&self) -> &HashMap<Uuid, PaneInfo> {
        &self.registered_panes
    }

    /// Get panes in a specific tab
    pub fn get_panes_in_tab(&self, tab_id: Uuid) -> Vec<&PaneInfo> {
        self.registered_panes
            .values()
            .filter(|pane| pane.tab_id == tab_id)
            .collect()
    }

    /// Check if a specific pane would be included in synchronization
    pub fn is_pane_synchronized(&self, pane_id: Uuid) -> bool {
        if !self.should_synchronize() {
            return false;
        }
        self.get_target_panes().contains(&pane_id)
    }
}

/// Status information about the current synchronization state
#[derive(Debug, Clone)]
pub struct SynchronizationStatus {
    pub scope: SynchronizationScope,
    pub is_active: bool,
    pub is_paused: bool,
    pub target_pane_count: usize,
    pub active_pane_id: Option<Uuid>,
}

impl SynchronizationStatus {
    /// Get a human-readable description of the current status
    pub fn description(&self) -> String {
        if !self.is_active {
            "Synchronization off".to_string()
        } else if self.is_paused {
            "Synchronization paused".to_string()
        } else {
            match self.scope {
                SynchronizationScope::CurrentTab => {
                    format!("Syncing to {} panes in current tab", self.target_pane_count)
                }
                SynchronizationScope::AllTabs => {
                    format!("Syncing to {} panes across all tabs", self.target_pane_count)
                }
                SynchronizationScope::None => "Synchronization off".to_string(),
            }
        }
    }

    /// Get a short status indicator for the UI
    pub fn short_indicator(&self) -> String {
        if !self.is_active {
            "⚫".to_string() // Off
        } else if self.is_paused {
            "⏸".to_string() // Paused
        } else {
            match self.scope {
                SynchronizationScope::CurrentTab => "🔗".to_string(), // Tab sync
                SynchronizationScope::AllTabs => "🌐".to_string(),    // Global sync
                SynchronizationScope::None => "⚫".to_string(),        // Off
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pane(pane_id: Uuid, tab_id: Uuid, editor_type: EditorType) -> PaneInfo {
        PaneInfo {
            pane_id,
            tab_id,
            is_active: false,
            editor_type,
        }
    }

    #[test]
    fn test_synchronization_manager_basic() {
        let mut manager = SynchronizationManager::new();
        assert_eq!(manager.scope, SynchronizationScope::None);
        assert!(!manager.should_synchronize());
    }

    #[test]
    fn test_pane_registration() {
        let mut manager = SynchronizationManager::new();
        let pane_id = Uuid::new_v4();
        let tab_id = Uuid::new_v4();
        
        let pane = create_test_pane(pane_id, tab_id, EditorType::Shell);
        manager.register_pane(pane.clone());
        
        assert!(manager.get_all_panes().contains_key(&pane_id));
    }

    #[test]
    fn test_synchronization_scope_toggle() {
        let mut manager = SynchronizationManager::new();
        
        // None -> CurrentTab
        manager.toggle_synchronization();
        assert_eq!(manager.scope, SynchronizationScope::CurrentTab);
        
        // CurrentTab -> AllTabs
        manager.toggle_synchronization();
        assert_eq!(manager.scope, SynchronizationScope::AllTabs);
        
        // AllTabs -> None
        manager.toggle_synchronization();
        assert_eq!(manager.scope, SynchronizationScope::None);
    }

    #[test]
    fn test_target_panes_current_tab() {
        let mut manager = SynchronizationManager::new();
        let tab1_id = Uuid::new_v4();
        let tab2_id = Uuid::new_v4();
        
        let pane1 = create_test_pane(Uuid::new_v4(), tab1_id, EditorType::Shell);
        let pane2 = create_test_pane(Uuid::new_v4(), tab1_id, EditorType::Shell);
        let pane3 = create_test_pane(Uuid::new_v4(), tab2_id, EditorType::Shell);
        
        manager.register_pane(pane1.clone());
        manager.register_pane(pane2.clone());
        manager.register_pane(pane3.clone());
        
        manager.set_active_pane(pane1.pane_id);
        manager.start_synchronization(SynchronizationScope::CurrentTab);
        
        let targets = manager.get_target_panes();
        assert_eq!(targets.len(), 1);
        assert!(targets.contains(&pane2.pane_id));
        assert!(!targets.contains(&pane3.pane_id));
    }

    #[test]
    fn test_target_panes_all_tabs() {
        let mut manager = SynchronizationManager::new();
        let tab1_id = Uuid::new_v4();
        let tab2_id = Uuid::new_v4();
        
        let pane1 = create_test_pane(Uuid::new_v4(), tab1_id, EditorType::Shell);
        let pane2 = create_test_pane(Uuid::new_v4(), tab1_id, EditorType::Shell);
        let pane3 = create_test_pane(Uuid::new_v4(), tab2_id, EditorType::Shell);
        
        manager.register_pane(pane1.clone());
        manager.register_pane(pane2.clone());
        manager.register_pane(pane3.clone());
        
        manager.set_active_pane(pane1.pane_id);
        manager.start_synchronization(SynchronizationScope::AllTabs);
        
        let targets = manager.get_target_panes();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&pane2.pane_id));
        assert!(targets.contains(&pane3.pane_id));
    }

    #[test]
    fn test_editor_type_filtering() {
        let mut manager = SynchronizationManager::new();
        let tab_id = Uuid::new_v4();
        
        let shell_pane1 = create_test_pane(Uuid::new_v4(), tab_id, EditorType::Shell);
        let shell_pane2 = create_test_pane(Uuid::new_v4(), tab_id, EditorType::Shell);
        let vim_pane = create_test_pane(Uuid::new_v4(), tab_id, EditorType::Vim);
        
        manager.register_pane(shell_pane1.clone());
        manager.register_pane(shell_pane2.clone());
        manager.register_pane(vim_pane.clone());
        
        manager.set_active_pane(shell_pane1.pane_id);
        manager.start_synchronization(SynchronizationScope::CurrentTab);
        
        let targets = manager.get_target_panes();
        assert_eq!(targets.len(), 1);
        assert!(targets.contains(&shell_pane2.pane_id));
        assert!(!targets.contains(&vim_pane.pane_id));
    }
}
