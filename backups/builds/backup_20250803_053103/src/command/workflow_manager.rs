use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkflowManager {
    workflows: HashMap<String, Workflow>,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub working_directory: Option<PathBuf>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    pub fn add_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.name.clone(), workflow);
    }

    pub fn run_workflow(&self, name: &str) -> Result<(), String> {
        if let Some(workflow) = self.workflows.get(name) {
            for step in &workflow.steps {
                println!("Running step: {}", step.name);
                // Here you would execute the command, handle directories and dependencies
                // For now, just printing the command
                println!("Executing command: {}", step.command);
            }
            Ok(())
        } else {
            Err(format!("Workflow '{}' not found.", name))
        }
    }

    pub fn list_workflows(&self) -> Vec<String> {
        self.workflows.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_run_workflow() {
        let mut manager = WorkflowManager::new();

        let workflow = Workflow {
            name: "Build Project".to_string(),
            steps: vec![WorkflowStep {
                name: "Compile".to_string(),
                command: "cargo build".to_string(),
                description: Some("Compile the project".to_string()),
                dependencies: vec![],
                working_directory: None,
            }],
        };
        manager.add_workflow(workflow);

        assert!(manager.run_workflow("Build Project").is_ok());
    }

    #[test]
    fn test_list_workflows() {
        let mut manager = WorkflowManager::new();

        manager.add_workflow(Workflow {
            name: "Test Workflow 1".to_string(),
            steps: vec![],
        });
        manager.add_workflow(Workflow {
            name: "Test Workflow 2".to_string(),
            steps: vec![],
        });

        let workflows = manager.list_workflows();
        assert_eq!(workflows.len(), 2);
        assert!(workflows.contains(&"Test Workflow 1".to_string()));
        assert!(workflows.contains(&"Test Workflow 2".to_string()));
    }
}
