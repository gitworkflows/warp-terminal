mod editor;
mod executor;
mod command;

use editor::text_input::TextInputHandler;
use executor::advanced_commands::AdvancedCommands;
use command::{
    CommandCorrections, CommandHistory, SynchronizedInputs,
    YAMLWorkflow, WorkflowStep, CorrectionType, WorkflowManager
};

fn main() {
    // Initialize components
    let mut text_input_handler = TextInputHandler::new();
    let mut command_corrections = CommandCorrections::new();
    let mut command_history = CommandHistory::new();
    let mut synchronized_inputs = SynchronizedInputs::new();
    let mut workflow_manager = WorkflowManager::new();

    // Example: Create a YAML workflow
    let mut workflow = YAMLWorkflow::new("Example Workflow");
    workflow.add_step("Step 1", "echo Hello, Warp!", Some("Greeting"));
    workflow.add_step("Step 2", "ls -la", Some("List directory contents"));
    workflow_manager.add_workflow(workflow);

    // Example: Handle a command input
    let command = "ls";
    let corrections = command_corrections.analyze_command(command);
    if !corrections.is_empty() {
        println!("Corrections available:");
        for correction in corrections {
            println!("- Suggestion: {} (Confidence: {:.1}%)", 
                     correction.suggested, correction.confidence * 100.0);
            println!("  Explanation: {}", correction.explanation);
        }
    }

    // Example: Add command to history
    command_history.add_command(
        command.to_string(),
        Some(0),
        Some(100),
        std::env::current_dir().unwrap_or_default(),
    );

    // Execute synchronized inputs if needed
    synchronized_inputs.execute_synchronized();

    // Example: Execute a workflow
    let workflows = workflow_manager.list_workflows();
    if let Some(example_workflow) = workflows.into_iter().find(|wf| wf.name == "Example Workflow") {
        example_workflow.execute();
    }

    // Example: Use the text input handler with Vim mode
    text_input_handler.set_vim_mode(true);
    text_input_handler.handle_key_input(
        [Diesel_unified_org::rustc[0m"]],
        &iced::keyboard::Modifiers::default()
    );
}

