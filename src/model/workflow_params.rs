use std::collections::HashMap;
use warp_workflows_types::{Workflow, Argument};
use regex::Regex;

/// Handles parameterization of workflow commands
#[derive(Debug, Clone)]
pub struct WorkflowParameterHandler {
    /// Current workflow being processed
    pub workflow: Option<Workflow>,
    /// Parameter values provided by the user
    pub parameter_values: HashMap<String, String>,
    /// Current parameter being filled (for UI state)
    pub current_param_index: usize,
}

impl Default for WorkflowParameterHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowParameterHandler {
    pub fn new() -> Self {
        Self {
            workflow: None,
            parameter_values: HashMap::new(),
            current_param_index: 0,
        }
    }

    /// Set the current workflow and reset parameter state
    pub fn set_workflow(&mut self, workflow: Workflow) {
        self.workflow = Some(workflow);
        self.parameter_values.clear();
        self.current_param_index = 0;
        
        // Initialize with default values if available
        if let Some(ref wf) = self.workflow {
            for arg in &wf.arguments {
                if let Some(default_value) = &arg.default_value {
                    self.parameter_values.insert(arg.name.clone(), default_value.clone());
                }
            }
        }
    }

    /// Check if the current workflow has parameters
    pub fn has_parameters(&self) -> bool {
        self.workflow.as_ref().map_or(false, |wf| !wf.arguments.is_empty())
    }

    /// Get the current parameter being filled
    pub fn get_current_parameter(&self) -> Option<&Argument> {
        self.workflow.as_ref()
            .and_then(|wf| wf.arguments.get(self.current_param_index))
    }

    /// Set the value for the current parameter and advance to the next
    pub fn set_current_parameter_value(&mut self, value: String) {
        if let Some(param) = self.get_current_parameter() {
            self.parameter_values.insert(param.name.clone(), value);
            self.current_param_index += 1;
        }
    }

    /// Go back to the previous parameter
    pub fn previous_parameter(&mut self) {
        if self.current_param_index > 0 {
            self.current_param_index -= 1;
        }
    }

    /// Check if all parameters have been filled
    pub fn all_parameters_filled(&self) -> bool {
        if let Some(ref workflow) = self.workflow {
            for arg in &workflow.arguments {
                if !self.parameter_values.contains_key(&arg.name) {
                    return false;
                }
            }
            true
        } else {
            true
        }
    }

    /// Get the progress of parameter filling (filled_count, total_count)
    pub fn get_parameter_progress(&self) -> (usize, usize) {
        if let Some(ref workflow) = self.workflow {
            let total = workflow.arguments.len();
            let filled = workflow.arguments.iter()
                .filter(|arg| self.parameter_values.contains_key(&arg.name))
                .count();
            (filled, total)
        } else {
            (0, 0)
        }
    }

    /// Replace parameters in the command string with their values
    pub fn render_command(&self) -> Option<String> {
        if let Some(ref workflow) = self.workflow {
            let mut command = workflow.command.clone();
            
            // Replace each parameter with its value
            for (param_name, param_value) in &self.parameter_values {
                let pattern = format!("{{{{{}}}}}", param_name);
                command = command.replace(&pattern, param_value);
            }
            
            Some(command)
        } else {
            None
        }
    }

    /// Extract parameter names from a command string
    pub fn extract_parameter_names(command: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        re.captures_iter(command)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Check if a command has parameters
    pub fn command_has_parameters(command: &str) -> bool {
        let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        re.is_match(command)
    }

    /// Get all remaining parameters that need to be filled
    pub fn get_remaining_parameters(&self) -> Vec<&Argument> {
        if let Some(ref workflow) = self.workflow {
            workflow.arguments.iter()
                .skip(self.current_param_index)
                .collect()
        } else {
            vec![]
        }
    }

    /// Reset parameter handler state
    pub fn reset(&mut self) {
        self.workflow = None;
        self.parameter_values.clear();
        self.current_param_index = 0;
    }

    /// Get a preview of the command with current parameter values
    pub fn preview_command(&self) -> Option<String> {
        if let Some(ref workflow) = self.workflow {
            let mut command = workflow.command.clone();
            
            // Replace filled parameters
            for (param_name, param_value) in &self.parameter_values {
                let pattern = format!("{{{{{}}}}}", param_name);
                command = command.replace(&pattern, param_value);
            }
            
            // Show placeholder for unfilled parameters
            let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
            for cap in re.captures_iter(&command.clone()) {
                let param_name = &cap[1];
                let placeholder = format!("<{}>", param_name);
                command = command.replace(&cap[0], &placeholder);
            }
            
            Some(command)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp_workflows_types::{Workflow, Argument};

    fn create_test_workflow() -> Workflow {
        Workflow::new("Test Workflow", "echo {{message}} > {{file}}")
            .with_arguments(vec![
                Argument::new("message").with_description("Message to echo"),
                Argument::new("file").with_description("Output file"),
            ])
    }

    #[test]
    fn test_parameter_extraction() {
        let command = "git commit -m '{{message}}' && git push {{remote}} {{branch}}";
        let params = WorkflowParameterHandler::extract_parameter_names(command);
        assert_eq!(params, vec!["message", "remote", "branch"]);
    }

    #[test]
    fn test_workflow_parameterization() {
        let mut handler = WorkflowParameterHandler::new();
        let workflow = create_test_workflow();
        
        handler.set_workflow(workflow);
        
        assert!(handler.has_parameters());
        assert_eq!(handler.get_parameter_progress(), (0, 2));
        
        handler.set_current_parameter_value("Hello World".to_string());
        assert_eq!(handler.get_parameter_progress(), (1, 2));
        
        handler.set_current_parameter_value("output.txt".to_string());
        assert_eq!(handler.get_parameter_progress(), (2, 2));
        assert!(handler.all_parameters_filled());
        
        let rendered = handler.render_command().unwrap();
        assert_eq!(rendered, "echo Hello World > output.txt");
    }

    #[test]
    fn test_command_preview() {
        let mut handler = WorkflowParameterHandler::new();
        let workflow = create_test_workflow();
        
        handler.set_workflow(workflow);
        
        // No parameters filled yet
        let preview = handler.preview_command().unwrap();
        assert_eq!(preview, "echo <message> > <file>");
        
        // Fill one parameter
        handler.set_current_parameter_value("Hello".to_string());
        let preview = handler.preview_command().unwrap();
        assert_eq!(preview, "echo Hello > <file>");
    }
}
