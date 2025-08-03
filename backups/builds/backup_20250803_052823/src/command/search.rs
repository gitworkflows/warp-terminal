use crate::command::history::{CommandHistory, SearchResult as HistorySearchResult, MatchType};
use crate::command::synchronized_inputs::{YAMLWorkflow, WorkflowStep};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CommandSearch {
    history: CommandHistory,
    workflows: Vec<YAMLWorkflow>,
    notebooks: Vec<Notebook>,
    ai_suggestions: AISuggestions,
    enabled_sources: SearchSources,
}

#[derive(Debug, Clone)]
pub struct SearchSources {
    pub history: bool,
    pub workflows: bool,
    pub notebooks: bool,
    pub ai_suggestions: bool,
}

#[derive(Debug, Clone)]
pub struct Notebook {
    pub name: String,
    pub commands: Vec<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AISuggestions {
    suggestions: HashMap<String, Vec<String>>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UnifiedSearchResult {
    pub content: String,
    pub source: SearchSource,
    pub relevance_score: f32,
    pub metadata: SearchMetadata,
}

#[derive(Debug, Clone)]
pub enum SearchSource {
    History(HistorySearchResult),
    Workflow(WorkflowSearchResult),
    Notebook(NotebookSearchResult),
    AI(AISearchResult),
}

#[derive(Debug, Clone)]
pub struct WorkflowSearchResult {
    pub workflow_name: String,
    pub step_name: String,
    pub command: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NotebookSearchResult {
    pub notebook_name: String,
    pub command: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct AISearchResult {
    pub suggestion: String,
    pub explanation: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct SearchMetadata {
    pub timestamp: Option<u64>,
    pub frequency: Option<u32>,
    pub tags: Vec<String>,
    pub context: Option<String>,
}

impl Default for CommandSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandSearch {
    pub fn new() -> Self {
        Self {
            history: CommandHistory::new(),
            workflows: Vec::new(),
            notebooks: Vec::new(),
            ai_suggestions: AISuggestions::new(),
            enabled_sources: SearchSources {
                history: true,
                workflows: true,
                notebooks: true,
                ai_suggestions: true,
            },
        }
    }

    pub fn search_all(&self, query: &str, limit: usize) -> Vec<UnifiedSearchResult> {
        let mut results = Vec::new();

        // Search command history
        if self.enabled_sources.history {
            let history_results = self.history.search(query, limit);
            for result in history_results {
                results.push(UnifiedSearchResult {
                    content: result.entry.command.clone(),
                    source: SearchSource::History(result.clone()),
                    relevance_score: result.relevance_score,
                    metadata: SearchMetadata {
                        timestamp: Some(result.entry.timestamp),
                        frequency: Some(result.entry.frequency),
                        tags: result.entry.tags.clone(),
                        context: Some(format!("Session: {}", result.entry.session_id)),
                    },
                });
            }
        }

        // Search workflows
        if self.enabled_sources.workflows {
            let workflow_results = self.search_workflows(query);
            for result in workflow_results {
                results.push(UnifiedSearchResult {
                    content: result.command.clone(),
                    source: SearchSource::Workflow(result.clone()),
                    relevance_score: self.calculate_workflow_relevance(&result, query),
                    metadata: SearchMetadata {
                        timestamp: None,
                        frequency: None,
                        tags: vec!["workflow".to_string()],
                        context: Some(format!("Workflow: {}", result.workflow_name)),
                    },
                });
            }
        }

        // Search notebooks
        if self.enabled_sources.notebooks {
            let notebook_results = self.search_notebooks(query);
            for result in notebook_results {
                results.push(UnifiedSearchResult {
                    content: result.command.clone(),
                    source: SearchSource::Notebook(result.clone()),
                    relevance_score: self.calculate_notebook_relevance(&result, query),
                    metadata: SearchMetadata {
                        timestamp: None,
                        frequency: None,
                        tags: vec!["notebook".to_string()],
                        context: Some(format!("Notebook: {}", result.notebook_name)),
                    },
                });
            }
        }

        // Get AI suggestions
        if self.enabled_sources.ai_suggestions && self.ai_suggestions.enabled {
            let ai_results = self.ai_suggestions.get_suggestions(query);
            for result in ai_results {
                results.push(UnifiedSearchResult {
                    content: result.suggestion.clone(),
                    source: SearchSource::AI(result.clone()),
                    relevance_score: result.confidence,
                    metadata: SearchMetadata {
                        timestamp: None,
                        frequency: None,
                        tags: vec!["ai".to_string()],
                        context: Some("AI Suggestion".to_string()),
                    },
                });
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(limit);

        results
    }

    pub fn add_workflow(&mut self, workflow: YAMLWorkflow) {
        self.workflows.push(workflow);
    }

    pub fn add_notebook(&mut self, notebook: Notebook) {
        self.notebooks.push(notebook);
    }

    pub fn set_search_sources(&mut self, sources: SearchSources) {
        self.enabled_sources = sources;
    }

    pub fn get_search_statistics(&self) -> SearchStatistics {
        SearchStatistics {
            total_history_entries: self.history.get_recent_commands(usize::MAX).len(),
            total_workflows: self.workflows.len(),
            total_notebooks: self.notebooks.len(),
            ai_suggestions_enabled: self.ai_suggestions.enabled,
        }
    }

    // Private methods

    fn search_workflows(&self, query: &str) -> Vec<WorkflowSearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for workflow in &self.workflows {
            for step in &workflow.steps {
                if step.command.to_lowercase().contains(&query_lower) ||
                   step.name.to_lowercase().contains(&query_lower) ||
                   step.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower)) {
                    
                    results.push(WorkflowSearchResult {
                        workflow_name: workflow.name.clone(),
                        step_name: step.name.clone(),
                        command: step.command.clone(),
                        description: step.description.clone(),
                    });
                }
            }
        }

        results
    }

    fn search_notebooks(&self, query: &str) -> Vec<NotebookSearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for notebook in &self.notebooks {
            for command in &notebook.commands {
                if command.to_lowercase().contains(&query_lower) ||
                   notebook.name.to_lowercase().contains(&query_lower) ||
                   notebook.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                    
                    results.push(NotebookSearchResult {
                        notebook_name: notebook.name.clone(),
                        command: command.clone(),
                        context: notebook.description.clone().unwrap_or_default(),
                    });
                }
            }
        }

        results
    }

    fn calculate_workflow_relevance(&self, result: &WorkflowSearchResult, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let command_lower = result.command.to_lowercase();

        if command_lower == query_lower {
            return 0.95;
        }

        if command_lower.starts_with(&query_lower) {
            return 0.85;
        }

        if command_lower.contains(&query_lower) {
            return 0.75;
        }

        // Check step name and description
        if result.step_name.to_lowercase().contains(&query_lower) {
            return 0.65;
        }

        if let Some(desc) = &result.description {
            if desc.to_lowercase().contains(&query_lower) {
                return 0.55;
            }
        }

        0.0
    }

    fn calculate_notebook_relevance(&self, result: &NotebookSearchResult, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let command_lower = result.command.to_lowercase();

        if command_lower == query_lower {
            return 0.9;
        }

        if command_lower.starts_with(&query_lower) {
            return 0.8;
        }

        if command_lower.contains(&query_lower) {
            return 0.7;
        }

        if result.notebook_name.to_lowercase().contains(&query_lower) {
            return 0.6;
        }

        if result.context.to_lowercase().contains(&query_lower) {
            return 0.5;
        }

        0.0
    }
}

impl AISuggestions {
    pub fn new() -> Self {
        Self {
            suggestions: HashMap::new(),
            enabled: true,
        }
    }

    pub fn get_suggestions(&self, query: &str) -> Vec<AISearchResult> {
        if !self.enabled {
            return Vec::new();
        }

        let mut results = Vec::new();

        // Simple AI suggestion logic (in a real implementation, this would use ML/AI)
        let common_patterns = vec![
            ("list", "ls -la", "List directory contents with details", 0.9),
            ("find", "find . -name \"*{}*\"", "Find files matching pattern", 0.85),
            ("search", "grep -r \"{}\" .", "Search for text in files", 0.8),
            ("git", "git status", "Check git repository status", 0.75),
            ("docker", "docker ps", "List running containers", 0.7),
            ("process", "ps aux | grep {}", "Find processes by name", 0.85),
            ("disk", "df -h", "Check disk usage", 0.8),
            ("memory", "free -h", "Check memory usage", 0.8),
            ("network", "netstat -tuln", "Show network connections", 0.75),
        ];

        let query_lower = query.to_lowercase();
        for (pattern, command, explanation, confidence) in common_patterns {
            if query_lower.contains(pattern) {
                let suggestion = if command.contains("{}") {
                    command.replace("{}", query)
                } else {
                    command.to_string()
                };

                results.push(AISearchResult {
                    suggestion,
                    explanation: explanation.to_string(),
                    confidence,
                });
            }
        }

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        results.truncate(5); // Limit AI suggestions

        results
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug, Clone)]
pub struct SearchStatistics {
    pub total_history_entries: usize,
    pub total_workflows: usize,
    pub total_notebooks: usize,
    pub ai_suggestions_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_search() {
        let mut search = CommandSearch::new();
        
        // Add a workflow
        let mut workflow = YAMLWorkflow::new("Test Workflow");
        workflow.add_step("List files", "ls -la", Some("List all files"));
        search.add_workflow(workflow);

        // Add a notebook
        let notebook = Notebook {
            name: "Git Commands".to_string(),
            commands: vec!["git status".to_string(), "git commit".to_string()],
            description: Some("Common git commands".to_string()),
            tags: vec!["git".to_string(), "version-control".to_string()],
        };
        search.add_notebook(notebook);

        let results = search.search_all("git", 10);
        assert!(!results.is_empty());
        
        // Should find results from notebooks and potentially AI suggestions
        let has_notebook_result = results.iter().any(|r| matches!(r.source, SearchSource::Notebook(_)));
        assert!(has_notebook_result);
    }

    #[test]
    fn test_ai_suggestions() {
        let ai = AISuggestions::new();
        let suggestions = ai.get_suggestions("list files");
        
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].suggestion.contains("ls"));
    }

    #[test]
    fn test_search_sources_toggle() {
        let mut search = CommandSearch::new();
        
        // Disable AI suggestions
        search.set_search_sources(SearchSources {
            history: true,
            workflows: true,
            notebooks: true,
            ai_suggestions: false,
        });

        let results = search.search_all("test", 10);
        
        // Should not contain AI results
        let has_ai_result = results.iter().any(|r| matches!(r.source, SearchSource::AI(_)));
        assert!(!has_ai_result);
    }
}
