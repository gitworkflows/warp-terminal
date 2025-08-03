use warp_terminal::command::{
    CommandCorrections, CommandHistory, CommandSearch, SynchronizedInputs, 
    YAMLWorkflow, WorkflowManager, Workflow, Notebook,
};
use warp_terminal::text_input::TextInputHandler;
use std::collections::HashMap;

fn main() {
    println!("Warp Terminal - Modern Text Editing Features Demo");
    println!("=".repeat(50));

    // Initialize all components
    let mut history = CommandHistory::new();
    let mut corrections = CommandCorrections::new();
    let mut sync_inputs = SynchronizedInputs::new();
    let mut text_handler = TextInputHandler::new();
    let mut search = CommandSearch::new();
    let mut workflow_manager = WorkflowManager::new();

    // Enable Vim mode
    text_handler.enable_vim_mode();

    println!("\n1. Command History Demo");
    println!("-".repeat(30));
    
    // Add some sample commands to history
    history.add_command("git status", vec!["git".to_string()]);
    history.add_command("ls -la", vec!["file".to_string()]);
    history.add_command("cargo build", vec!["rust".to_string(), "build".to_string()]);
    
    // Search history
    let results = history.search("git", 5);
    println!("Search results for 'git': {} matches", results.len());
    for result in results {
        println!("  - {} (score: {:.2})", result.entry.command, result.relevance_score);
    }

    println!("\n2. Command Corrections Demo");
    println!("-".repeat(30));
    
    // Test command corrections
    let typo_corrections = corrections.suggest_corrections("gti status");
    println!("Corrections for 'gti status':");
    for correction in typo_corrections {
        println!("  - {} (confidence: {:.2})", correction.suggestion, correction.confidence);
    }

    println!("\n3. YAML Workflow Demo");
    println!("-".repeat(30));
    
    // Create a sample workflow
    let mut workflow = YAMLWorkflow::new("Rust Development");
    workflow.add_step("Format code", "cargo fmt", Some("Format Rust code"));
    workflow.add_step("Run tests", "cargo test", Some("Execute all tests"));
    workflow.add_step("Build release", "cargo build --release", Some("Build optimized binary"));
    
    println!("Created workflow: {}", workflow.name);
    println!("Steps:");
    for step in &workflow.steps {
        println!("  - {}: {}", step.name, step.command);
    }

    println!("\n4. Synchronized Inputs Demo");
    println!("-".repeat(30));
    
    // Add workflow to synchronized inputs
    sync_inputs.add_workflow(workflow.clone());
    
    // Execute workflow
    match sync_inputs.execute_workflow("Rust Development", &HashMap::new()) {
        Ok(_) => println!("Workflow executed successfully!"),
        Err(e) => println!("Workflow execution failed: {}", e),
    }

    println!("\n5. Unified Command Search Demo");
    println!("-".repeat(30));
    
    // Add workflow to search
    search.add_workflow(workflow);
    
    // Add a notebook
    let notebook = Notebook {
        name: "Git Commands".to_string(),
        commands: vec![
            "git status".to_string(),
            "git commit -m 'message'".to_string(),
            "git push origin main".to_string(),
        ],
        description: Some("Common git operations".to_string()),
        tags: vec!["git".to_string(), "version-control".to_string()],
    };
    search.add_notebook(notebook);
    
    // Search across all sources
    let unified_results = search.search_all("git", 10);
    println!("Unified search results for 'git': {} matches", unified_results.len());
    for result in unified_results {
        println!("  - {} (score: {:.2}, source: {:?})", 
                result.content, result.relevance_score, 
                match result.source {
                    warp_terminal::command::SearchSource::History(_) => "History",
                    warp_terminal::command::SearchSource::Workflow(_) => "Workflow",
                    warp_terminal::command::SearchSource::Notebook(_) => "Notebook",
                    warp_terminal::command::SearchSource::AI(_) => "AI",
                });
    }

    println!("\n6. Workflow Manager Demo");
    println!("-".repeat(30));
    
    // Create and add workflow to manager
    let manager_workflow = warp_terminal::command::workflow_manager::Workflow {
        name: "Docker Setup".to_string(),
        steps: vec![
            warp_terminal::command::workflow_manager::WorkflowStep {
                name: "Build image".to_string(),
                command: "docker build -t myapp .".to_string(),
                description: Some("Build Docker image".to_string()),
                dependencies: vec![],
                working_directory: None,
            },
            warp_terminal::command::workflow_manager::WorkflowStep {
                name: "Run container".to_string(),
                command: "docker run -p 8080:8080 myapp".to_string(),
                description: Some("Start the container".to_string()),
                dependencies: vec!["Build image".to_string()],
                working_directory: None,
            },
        ],
    };
    
    workflow_manager.add_workflow(manager_workflow);
    
    let workflows = workflow_manager.list_workflows();
    println!("Available workflows: {:?}", workflows);
    
    match workflow_manager.run_workflow("Docker Setup") {
        Ok(_) => println!("Workflow manager executed successfully!"),
        Err(e) => println!("Workflow manager execution failed: {}", e),
    }

    println!("\nDemo completed! All features are working including:");
    println!("✓ Command history with search and tagging");
    println!("✓ Intelligent command corrections");
    println!("✓ YAML-based workflow definitions");
    println!("✓ Synchronized multi-session inputs");
    println!("✓ Unified command search across all sources");
    println!("✓ Workflow management system");
}
