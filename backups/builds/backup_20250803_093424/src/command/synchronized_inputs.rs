use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct SynchronizedInputs {
    sessions: HashMap<String, Vec<String>>, // session ID -> list of commands
    enabled: bool,
}

impl SynchronizedInputs {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            enabled: true,
        }
    }

    pub fn add_command(&mut self, session_id: &str, command: String) {
        if self.enabled {
            self.sessions.entry(session_id.to_string())
                .or_insert_with(Vec::new)
                .push(command);
        }
    }

    pub fn get_commands(&self, session_id: &str) -> Option<&Vec<String>> {
        self.sessions.get(session_id)
    }

    pub fn execute_synchronized(&self) {
        if self.enabled {
            for (session_id, commands) in &self.sessions {
                println!("Executing commands for session {}:", session_id);
                for command in commands {
                    println!("{}
", command);
                    // Logic to execute commands would go here
                }
            }
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YAMLWorkflow {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

impl YAMLWorkflow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }

    pub fn add_step(&mut self, name: &str, command: &str, description: Option<&str>) {
        self.steps.push(WorkflowStep {
            name: name.to_string(),
            command: command.to_string(),
            description: description.map(|d| d.to_string()),
        });
    }

    pub fn execute(&self) {
        println!("Executing workflow: {}", self.name);
        for step in &self.steps {
            println!("Executing step: {}", step.name);
            println!("Command: {}", step.command);
            // Logic to execute step.command would go here
        }
    }

    pub fn export_to_yaml(&self, file_path: &str) -> std::io::Result<()> {
        let yaml = serde_yaml::to_string(self).expect("Failed to serialize to YAML");
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn import_from_yaml(file_path: &str) -> std::io::Result<Self> {
        let yaml_content = fs::read_to_string(file_path)?;
        Ok(serde_yaml::from_str(&yaml_content).expect("Failed to deserialize YAML"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synchronized_inputs() {
        let mut sync_inputs = SynchronizedInputs::new();
        sync_inputs.add_command("session1", "echo Hello".to_string());
        sync_inputs.add_command("session1", "ls -la".to_string());

        let commands = sync_inputs.get_commands("session1").unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], "echo Hello");
    }

    #[test]
    fn test_yaml_workflow() {
        let file_path = "workflow.yaml";
        let mut workflow = YAMLWorkflow::new("Test Workflow");
        workflow.add_step("Step 1", "echo Hello", Some("Greet"));
        workflow.add_step("Step 2", "ls -la", None);

        workflow.export_to_yaml(file_path).unwrap();

        let imported_workflow = YAMLWorkflow::import_from_yaml(file_path).unwrap();
        assert_eq!(imported_workflow.name, "Test Workflow");
        assert_eq!(imported_workflow.steps.len(), 2);
        
        fs::remove_file(file_path).unwrap(); // Clean up
    }
}
