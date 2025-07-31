use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeOperation {
    Create {
        file_path: PathBuf,
        content: String,
        language: String,
    },
    Edit {
        file_path: PathBuf,
        changes: Vec<CodeChange>,
    },
    BatchEdit {
        file_paths: Vec<PathBuf>,
        pattern: String,
        replacement: String,
    },
    FixError {
        file_path: PathBuf,
        error_context: ErrorContext,
        suggested_fix: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub line_start: usize,
    pub line_end: usize,
    pub old_content: String,
    pub new_content: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Insert,
    Delete,
    Replace,
    Move,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_message: String,
    pub error_line: Option<usize>,
    pub error_column: Option<usize>,
    pub stack_trace: Option<String>,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationRequest {
    pub id: Uuid,
    pub prompt: String,
    pub context: GenerationContext,
    pub preferences: CodePreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext {
    pub current_directory: PathBuf,
    pub open_files: Vec<PathBuf>,
    pub git_context: Option<GitContext>,
    pub project_type: Option<String>,
    pub existing_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub branch: String,
    pub recent_commits: Vec<String>,
    pub modified_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePreferences {
    pub language: Option<String>,
    pub style_guide: Option<String>,
    pub indentation: String,
    pub line_endings: String,
    pub max_line_length: usize,
}

impl Default for CodePreferences {
    fn default() -> Self {
        Self {
            language: None,
            style_guide: None,
            indentation: "    ".to_string(), // 4 spaces
            line_endings: "\n".to_string(),
            max_line_length: 100,
        }
    }
}

pub struct AICodeGenerator {
    api_client: Option<MockAIClient>,
    #[allow(dead_code)]
    templates: HashMap<String, CodeTemplate>,
    #[allow(dead_code)]
    context_analyzer: ContextAnalyzer,
}


#[derive(Debug, Clone)]
pub struct CodeTemplate {
    pub name: String,
    pub language: String,
    pub template: String,
    pub variables: Vec<String>,
}

pub struct ContextAnalyzer {
    #[allow(dead_code)]
    language_patterns: HashMap<String, Vec<String>>,
    #[allow(dead_code)]
    framework_detectors: HashMap<String, Vec<String>>,
}


impl AICodeGenerator {
    pub fn new() -> Self {
        Self {
            api_client: None,
            templates: Self::load_default_templates(),
            context_analyzer: ContextAnalyzer::new(),
        }
    }

    pub fn with_api_client(mut self, client: MockAIClient) -> Self {
        self.api_client = Some(client);
        self
    }

    pub async fn process_code_request(&self, prompt: &str, context: GenerationContext) -> Result<CodeOperation> {
        let request_type = self.analyze_request_type(prompt)?;
        let preferences = self.detect_code_preferences(&context).await?;
        
        let request = CodeGenerationRequest {
            id: Uuid::new_v4(),
            prompt: prompt.to_string(),
            context,
            preferences,
        };

        match request_type {
            RequestType::Create => self.handle_create_request(&request).await,
            RequestType::Edit => self.handle_edit_request(&request).await,
            RequestType::BatchEdit => self.handle_batch_edit_request(&request).await,
            RequestType::FixError => self.handle_fix_error_request(&request).await,
        }
    }

    async fn handle_create_request(&self, request: &CodeGenerationRequest) -> Result<CodeOperation> {
        let language = self.detect_language(&request.prompt, &request.context)?;
        let file_path = self.suggest_file_path(&request.prompt, &language, &request.context)?;
        
        let content = if let Some(client) = &self.api_client {
            client.generate_code(request).await?
        } else {
            self.generate_from_template(&request.prompt, &language)?
        };

        Ok(CodeOperation::Create {
            file_path,
            content,
            language,
        })
    }

    async fn handle_edit_request(&self, request: &CodeGenerationRequest) -> Result<CodeOperation> {
        let file_path = self.extract_file_path_from_context(&request.context)?;
        let current_content = fs::read_to_string(&file_path).await?;
        
        let changes = if let Some(client) = &self.api_client {
            self.generate_changes_with_ai(client, request, &current_content).await?
        } else {
            self.generate_basic_changes(&request.prompt, &current_content)?
        };

        Ok(CodeOperation::Edit {
            file_path,
            changes,
        })
    }

    async fn handle_batch_edit_request(&self, request: &CodeGenerationRequest) -> Result<CodeOperation> {
        let (pattern, replacement) = self.extract_pattern_replacement(&request.prompt)?;
        let file_paths = self.find_target_files(&request.context, &pattern).await?;

        Ok(CodeOperation::BatchEdit {
            file_paths,
            pattern,
            replacement,
        })
    }

    async fn handle_fix_error_request(&self, request: &CodeGenerationRequest) -> Result<CodeOperation> {
        let error_context = self.extract_error_context(&request.prompt)?;
        let file_path = self.extract_file_path_from_context(&request.context)?;
        
        let suggested_fix = if let Some(client) = &self.api_client {
            client.analyze_error(&error_context).await?
        } else {
            self.generate_basic_error_fix(&error_context)?
        };

        Ok(CodeOperation::FixError {
            file_path,
            error_context,
            suggested_fix,
        })
    }

    fn analyze_request_type(&self, prompt: &str) -> Result<RequestType> {
        let prompt_lower = prompt.to_lowercase();
        
        if prompt_lower.contains("write") || prompt_lower.contains("create") || prompt_lower.contains("generate") {
            Ok(RequestType::Create)
        } else if prompt_lower.contains("fix") && (prompt_lower.contains("error") || prompt_lower.contains("bug")) {
            Ok(RequestType::FixError)
        } else if prompt_lower.contains("all") && (prompt_lower.contains("files") || prompt_lower.contains("directory")) {
            Ok(RequestType::BatchEdit)
        } else if prompt_lower.contains("update") || prompt_lower.contains("change") || prompt_lower.contains("edit") {
            Ok(RequestType::Edit)
        } else {
            Ok(RequestType::Create) // Default to create
        }
    }

    fn detect_language(&self, prompt: &str, context: &GenerationContext) -> Result<String> {
        // Check if language is explicitly mentioned in prompt
        let prompt_lower = prompt.to_lowercase();
        let languages = vec![
            "javascript", "typescript", "python", "rust", "go", "java", 
            "c++", "c#", "php", "ruby", "swift", "kotlin", "dart"
        ];
        
        for lang in &languages {
            if prompt_lower.contains(lang) {
                return Ok(lang.to_string());
            }
        }

        // Try to detect from context
        if let Some(project_type) = &context.project_type {
            return Ok(project_type.clone());
        }

        // Detect from open files
        for file_path in &context.open_files {
            if let Some(extension) = file_path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    match ext_str {
                        "js" | "mjs" => return Ok("javascript".to_string()),
                        "ts" => return Ok("typescript".to_string()),
                        "py" => return Ok("python".to_string()),
                        "rs" => return Ok("rust".to_string()),
                        "go" => return Ok("go".to_string()),
                        "java" => return Ok("java".to_string()),
                        _ => continue,
                    }
                }
            }
        }

        Ok("javascript".to_string()) // Default fallback
    }

    fn suggest_file_path(&self, prompt: &str, language: &str, context: &GenerationContext) -> Result<PathBuf> {
        // Try to extract filename from prompt
        if let Some(filename) = self.extract_filename_from_prompt(prompt) {
            return Ok(context.current_directory.join(filename));
        }

        // Generate filename based on function/class name
        let base_name = self.extract_function_name(prompt)
            .unwrap_or_else(|| "generated_code".to_string());
        
        let extension = match language {
            "javascript" => "js",
            "typescript" => "ts",
            "python" => "py",
            "rust" => "rs",
            "go" => "go",
            "java" => "java",
            _ => "txt",
        };

        Ok(context.current_directory.join(format!("{}.{}", base_name, extension)))
    }

    fn extract_filename_from_prompt(&self, prompt: &str) -> Option<String> {
        // Look for patterns like "create file.js" or "write utils.py"
        let words: Vec<&str> = prompt.split_whitespace().collect();
        for i in 0..words.len() {
            if words[i].contains('.') && self.is_valid_filename(words[i]) {
                return Some(words[i].to_string());
            }
        }
        None
    }

    fn extract_function_name(&self, prompt: &str) -> Option<String> {
        // Look for patterns like "function to debounce" -> "debounce"
        let words: Vec<&str> = prompt.split_whitespace().collect();
        for i in 0..words.len() {
            if i > 0 && (words[i-1] == "to" || words[i-1] == "for") {
                return Some(words[i].replace(" ", "_"));
            }
        }
        None
    }

    fn is_valid_filename(&self, filename: &str) -> bool {
        filename.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
    }

    async fn detect_code_preferences(&self, context: &GenerationContext) -> Result<CodePreferences> {
        let mut preferences = CodePreferences::default();
        
        // Try to read existing code style from project files
        for file_path in &context.open_files {
            if let Ok(content) = fs::read_to_string(file_path).await {
                // Detect indentation
                if content.contains("\t") {
                    preferences.indentation = "\t".to_string();
                } else {
                    let spaces = self.detect_space_indentation(&content);
                    preferences.indentation = " ".repeat(spaces);
                }
                break;
            }
        }

        Ok(preferences)
    }

    fn detect_space_indentation(&self, content: &str) -> usize {
        let lines: Vec<&str> = content.lines().collect();
        let mut indent_counts = HashMap::new();
        
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            
            let leading_spaces = line.len() - line.trim_start().len();
            if leading_spaces > 0 {
                *indent_counts.entry(leading_spaces).or_insert(0) += 1;
            }
        }
        
        // Find the most common indentation that's a multiple of 2 or 4
        for &size in &[2, 4, 8] {
            if indent_counts.contains_key(&size) {
                return size;
            }
        }
        
        4 // Default to 4 spaces
    }

    fn generate_from_template(&self, prompt: &str, language: &str) -> Result<String> {
        // Simple template-based generation for fallback
        match language {
            "javascript" => self.generate_javascript_template(prompt),
            "typescript" => self.generate_typescript_template(prompt),
            "python" => self.generate_python_template(prompt),
            "rust" => self.generate_rust_template(prompt),
            _ => Ok(format!("// Generated code for: {}\n// Language: {}\n", prompt, language)),
        }
    }

    fn generate_javascript_template(&self, prompt: &str) -> Result<String> {
        if prompt.to_lowercase().contains("debounce") {
            Ok(r#"/**
 * Debounce function that delays execution until after delay milliseconds
 * have elapsed since the last time it was invoked.
 * @param {Function} func - The function to debounce
 * @param {number} delay - The number of milliseconds to delay
 * @returns {Function} The debounced function
 */
function debounce(func, delay) {
    let timeoutId;
    return function (...args) {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => func.apply(this, args), delay);
    };
}

// Example usage:
// const debouncedSearch = debounce(searchFunction, 300);
"#.to_string())
        } else {
            Ok(format!("// Generated JavaScript code for: {}\nfunction generatedFunction() {{\n    // TODO: Implement functionality\n}}\n", prompt))
        }
    }

    fn generate_typescript_template(&self, prompt: &str) -> Result<String> {
        if prompt.to_lowercase().contains("debounce") {
            Ok(r#"/**
 * Debounce function that delays execution until after delay milliseconds
 * have elapsed since the last time it was invoked.
 */
function debounce<T extends (...args: any[]) => any>(
    func: T,
    delay: number
): (...args: Parameters<T>) => void {
    let timeoutId: NodeJS.Timeout | undefined;
    
    return function (...args: Parameters<T>): void {
        if (timeoutId) {
            clearTimeout(timeoutId);
        }
        timeoutId = setTimeout(() => func.apply(this, args), delay);
    };
}

// Example usage:
// const debouncedSearch = debounce(searchFunction, 300);
"#.to_string())
        } else {
            Ok(format!("// Generated TypeScript code for: {}\nfunction generatedFunction(): void {{\n    // TODO: Implement functionality\n}}\n", prompt))
        }
    }

    fn generate_python_template(&self, prompt: &str) -> Result<String> {
        if prompt.to_lowercase().contains("debounce") {
            Ok(r#"""
Debounce decorator that delays execution until after delay seconds
have elapsed since the last time it was invoked.
"""
import time
from functools import wraps
from typing import Callable, Any

def debounce(delay: float):
    """
    Decorator that debounces a function call.
    
    Args:
        delay: The number of seconds to delay
    
    Returns:
        The debounced function
    """
    def decorator(func: Callable) -> Callable:
        last_called = [0.0]
        
        @wraps(func)
        def wrapper(*args, **kwargs) -> Any:
            now = time.time()
            if now - last_called[0] >= delay:
                last_called[0] = now
                return func(*args, **kwargs)
        
        return wrapper
    return decorator

# Example usage:
# @debounce(0.3)
# def search_function(query):
#     print(f"Searching for: {query}")
"#.to_string())
        } else {
            Ok(format!("# Generated Python code for: {}\ndef generated_function():\n    \"\"\"TODO: Implement functionality\"\"\"\n    pass\n", prompt))
        }
    }

    fn generate_rust_template(&self, prompt: &str) -> Result<String> {
        Ok(format!("// Generated Rust code for: {}\npub fn generated_function() {{\n    // TODO: Implement functionality\n}}\n", prompt))
    }

    async fn generate_changes_with_ai(&self, client: &MockAIClient, request: &CodeGenerationRequest, current_content: &str) -> Result<Vec<CodeChange>> {
        let enhanced_request = CodeGenerationRequest {
            context: GenerationContext {
                existing_code: Some(current_content.to_string()),
                ..request.context.clone()
            },
            ..request.clone()
        };
        
        let ai_response = client.generate_code(&enhanced_request).await?;
        self.parse_changes_from_response(&ai_response, current_content)
    }

    fn generate_basic_changes(&self, prompt: &str, current_content: &str) -> Result<Vec<CodeChange>> {
        let mut changes = Vec::new();
        let lines: Vec<&str> = current_content.lines().collect();
        
        // Handle simple replacements
        if prompt.to_lowercase().contains("var") && prompt.to_lowercase().contains("let") {
            for (line_num, line) in lines.iter().enumerate() {
                if line.contains("var ") {
                    let new_line = line.replace("var ", "let ");
                    changes.push(CodeChange {
                        line_start: line_num + 1,
                        line_end: line_num + 1,
                        old_content: line.to_string(),
                        new_content: new_line,
                        change_type: ChangeType::Replace,
                    });
                }
            }
        }
        
        Ok(changes)
    }

    fn parse_changes_from_response(&self, response: &str, current_content: &str) -> Result<Vec<CodeChange>> {
        // This would parse AI response into structured changes
        // For now, return a simple replacement
        Ok(vec![CodeChange {
            line_start: 1,
            line_end: current_content.lines().count(),
            old_content: current_content.to_string(),
            new_content: response.to_string(),
            change_type: ChangeType::Replace,
        }])
    }

    fn extract_file_path_from_context(&self, context: &GenerationContext) -> Result<PathBuf> {
        context.open_files
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No file specified in context"))
    }

    fn extract_pattern_replacement(&self, prompt: &str) -> Result<(String, String)> {
        // Parse prompts like "Update all instances of 'var' to 'let'"
        if let Some(from_pos) = prompt.find("'") {
            if let Some(to_pos) = prompt.rfind("'") {
                let quoted_parts: Vec<&str> = prompt[from_pos..=to_pos].split("' to '").collect();
                if quoted_parts.len() == 2 {
                    let pattern = quoted_parts[0].trim_start_matches('\'');
                    let replacement = quoted_parts[1].trim_end_matches('\'');
                    return Ok((pattern.to_string(), replacement.to_string()));
                }
            }
        }
        
        Err(anyhow!("Could not extract pattern and replacement from prompt"))
    }

    async fn find_target_files(&self, context: &GenerationContext, pattern: &str) -> Result<Vec<PathBuf>> {
        // Find files based on context and pattern
        let mut files = Vec::new();
        
        // For now, just use open files
        for file_path in &context.open_files {
            if let Ok(content) = fs::read_to_string(file_path).await {
                if content.contains(pattern) {
                    files.push(file_path.clone());
                }
            }
        }
        
        Ok(files)
    }

    fn extract_error_context(&self, prompt: &str) -> Result<ErrorContext> {
        // Parse error information from prompt
        Ok(ErrorContext {
            error_message: prompt.to_string(),
            error_line: None,
            error_column: None,
            stack_trace: None,
            language: "javascript".to_string(), // Default
        })
    }

    fn generate_basic_error_fix(&self, error_context: &ErrorContext) -> Result<String> {
        // Basic error fix suggestions
        let message = &error_context.error_message.to_lowercase();
        
        if message.contains("undefined") {
            Ok("// Check if the variable is properly declared and initialized\n// Consider adding null/undefined checks".to_string())
        } else if message.contains("syntax") {
            Ok("// Check for missing semicolons, brackets, or quotes\n// Verify proper syntax according to language rules".to_string())
        } else {
            Ok("// Review the error message and check the relevant code section\n// Consider consulting language documentation".to_string())
        }
    }

    fn load_default_templates() -> HashMap<String, CodeTemplate> {
        let mut templates = HashMap::new();
        
        templates.insert("debounce_js".to_string(), CodeTemplate {
            name: "JavaScript Debounce".to_string(),
            language: "javascript".to_string(),
            template: include_str!("../templates/debounce.js").to_string(),
            variables: vec!["function_name".to_string(), "delay".to_string()],
        });
        
        templates
    }
}

impl ContextAnalyzer {
    fn new() -> Self {
        let mut language_patterns = HashMap::new();
        language_patterns.insert("javascript".to_string(), vec![
            "function".to_string(),
            "const".to_string(),
            "let".to_string(),
            "var".to_string(),
        ]);
        
        let mut framework_detectors = HashMap::new();
        framework_detectors.insert("react".to_string(), vec![
            "import React".to_string(),
            "useState".to_string(),
            "useEffect".to_string(),
        ]);
        
        Self {
            language_patterns,
            framework_detectors,
        }
    }
}

#[derive(Debug, Clone)]
enum RequestType {
    Create,
    Edit,
    BatchEdit,
    FixError,
}

// Mock API client for testing
pub struct MockAIClient;

impl MockAIClient {
    pub async fn generate_code(&self, request: &CodeGenerationRequest) -> Result<String> {
        Ok(format!("// Generated code for: {}\n// TODO: Implement", request.prompt))
    }

    pub async fn analyze_error(&self, error_context: &ErrorContext) -> Result<String> {
        Ok(format!("// Suggested fix for: {}", error_context.error_message))
    }

    pub async fn suggest_improvements(&self, _code: &str, _language: &str) -> Result<Vec<String>> {
        Ok(vec!["Consider adding error handling".to_string()])
    }
}
