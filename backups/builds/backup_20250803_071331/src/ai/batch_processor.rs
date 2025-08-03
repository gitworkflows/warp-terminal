use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use walkdir::WalkDir;
use regex::Regex;
use chrono::Datelike;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub id: String,
    pub operation_type: BatchOperationType,
    pub target_files: Vec<PathBuf>,
    pub changes: Vec<FileChange>,
    pub summary: BatchSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BatchOperationType {
    AddHeaders,
    UpdateImports,
    ReplacePattern,
    CodeFormatting,
    LicenseAddition,
    DependencyUpdate,
    RefactorRename,
    StyleNormalization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
    pub old_content: Option<String>,
    pub new_content: String,
    pub line_changes: Vec<LineChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    HeaderAddition,
    ContentReplacement,
    PatternUpdate,
    FormatChange,
    StructuralChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineChange {
    pub line_number: usize,
    pub change_type: LineChangeType,
    pub old_line: Option<String>,
    pub new_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineChangeType {
    Insert,
    Delete,
    Modify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub files_processed: usize,
    pub files_changed: usize,
    pub total_changes: usize,
    pub errors: Vec<String>,
    pub execution_time: Option<std::time::Duration>,
}

pub struct BatchProcessor {
    file_filters: HashMap<String, Box<dyn FileFilter>>,
    processors: HashMap<BatchOperationType, Box<dyn OperationProcessor>>,
}

pub trait FileFilter: Send + Sync {
    fn should_process(&self, file_path: &PathBuf) -> bool;
    fn get_filter_name(&self) -> &str;
}

pub trait OperationProcessor: Send + Sync {
    fn process_file(&self, file_path: &PathBuf, content: &str, params: &BatchParams) -> Result<FileChange>;
    fn get_processor_name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchParams {
    pub pattern: Option<String>,
    pub replacement: Option<String>,
    pub header_content: Option<String>,
    pub target_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub dry_run: bool,
    pub preserve_formatting: bool,
}

impl BatchProcessor {
    pub fn new() -> Self {
        let mut processor = Self {
            file_filters: HashMap::new(),
            processors: HashMap::new(),
        };

        processor.add_default_filters();
        processor.add_default_processors();
        
        processor
    }

    pub async fn execute_batch_operation(
        &self, 
        operation_type: BatchOperationType,
        directory: &PathBuf,
        params: BatchParams,
    ) -> Result<BatchOperation> {
        let start_time = std::time::Instant::now();
        let operation_id = uuid::Uuid::new_v4().to_string();

        // Find target files
        let target_files = self.find_target_files(directory, &params).await?;
        
        if target_files.is_empty() {
            return Err(anyhow!("No files found matching the criteria"));
        }

        // Get the appropriate processor
        let processor = self.processors.get(&operation_type)
            .ok_or_else(|| anyhow!("No processor found for operation type: {:?}", operation_type))?;

        let mut changes = Vec::new();
        let mut errors = Vec::new();
        let mut files_changed = 0;

        // Process each file
        for file_path in &target_files {
            match fs::read_to_string(file_path).await {
                Ok(content) => {
                    match processor.process_file(file_path, &content, &params) {
                        Ok(change) => {
                            if !params.dry_run {
                                // Apply the change
                                if let Err(e) = fs::write(file_path, &change.new_content).await {
                                    errors.push(format!("Failed to write {}: {}", file_path.display(), e));
                                } else {
                                    files_changed += 1;
                                }
                            }
                            changes.push(change);
                        }
                        Err(e) => {
                            errors.push(format!("Failed to process {}: {}", file_path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read {}: {}", file_path.display(), e));
                }
            }
        }

        let execution_time = start_time.elapsed();
        let total_changes: usize = changes.iter().map(|c| c.line_changes.len()).sum();

        let files_processed = target_files.len();
        
        Ok(BatchOperation {
            id: operation_id,
            operation_type,
            target_files,
            changes,
            summary: BatchSummary {
                files_processed,
                files_changed,
                total_changes,
                errors,
                execution_time: Some(execution_time),
            },
        })
    }

    async fn find_target_files(&self, directory: &PathBuf, params: &BatchParams) -> Result<Vec<PathBuf>> {
        let mut target_files = Vec::new();

        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            
            if !path.is_file() {
                continue;
            }

            // Check extension filter
            if !params.target_extensions.is_empty() {
                if let Some(extension) = path.extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if !params.target_extensions.contains(&ext_str.to_string()) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Check exclude patterns
            let path_str = path.to_string_lossy();
            if params.exclude_patterns.iter().any(|pattern| {
                if let Ok(regex) = Regex::new(pattern) {
                    regex.is_match(&path_str)
                } else {
                    path_str.contains(pattern)
                }
            }) {
                continue;
            }

            // Apply file filters
            let should_include = if params.target_extensions.is_empty() {
                // If no extensions specified, use default filters
                self.file_filters.values().any(|filter| filter.should_process(&path))
            } else {
                true // Already filtered by extension
            };

            if should_include {
                target_files.push(path);
            }
        }

        Ok(target_files)
    }

    fn add_default_filters(&mut self) {
        self.file_filters.insert("code".to_string(), Box::new(CodeFileFilter));
        self.file_filters.insert("python".to_string(), Box::new(PythonFileFilter));
        self.file_filters.insert("javascript".to_string(), Box::new(JavaScriptFileFilter));
        self.file_filters.insert("rust".to_string(), Box::new(RustFileFilter));
    }

    fn add_default_processors(&mut self) {
        self.processors.insert(BatchOperationType::AddHeaders, Box::new(HeaderProcessor));
        self.processors.insert(BatchOperationType::ReplacePattern, Box::new(PatternProcessor));
        self.processors.insert(BatchOperationType::UpdateImports, Box::new(ImportProcessor));
        self.processors.insert(BatchOperationType::CodeFormatting, Box::new(FormattingProcessor));
        self.processors.insert(BatchOperationType::LicenseAddition, Box::new(LicenseProcessor));
    }
}

// File Filters
pub struct CodeFileFilter;

impl FileFilter for CodeFileFilter {
    fn should_process(&self, file_path: &PathBuf) -> bool {
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                matches!(ext_str, "rs" | "js" | "ts" | "py" | "go" | "java" | "cpp" | "c" | "h")
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_filter_name(&self) -> &str {
        "CodeFileFilter"
    }
}

pub struct PythonFileFilter;

impl FileFilter for PythonFileFilter {
    fn should_process(&self, file_path: &PathBuf) -> bool {
        if let Some(extension) = file_path.extension() {
            extension.to_str() == Some("py")
        } else {
            false
        }
    }

    fn get_filter_name(&self) -> &str {
        "PythonFileFilter"
    }
}

pub struct JavaScriptFileFilter;

impl FileFilter for JavaScriptFileFilter {
    fn should_process(&self, file_path: &PathBuf) -> bool {
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                matches!(ext_str, "js" | "ts" | "jsx" | "tsx")
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_filter_name(&self) -> &str {
        "JavaScriptFileFilter"
    }
}

pub struct RustFileFilter;

impl FileFilter for RustFileFilter {
    fn should_process(&self, file_path: &PathBuf) -> bool {
        if let Some(extension) = file_path.extension() {
            extension.to_str() == Some("rs")
        } else {
            false
        }
    }

    fn get_filter_name(&self) -> &str {
        "RustFileFilter"
    }
}

// Operation Processors
pub struct HeaderProcessor;

impl OperationProcessor for HeaderProcessor {
    fn process_file(&self, file_path: &PathBuf, content: &str, params: &BatchParams) -> Result<FileChange> {
        let header_content = params.header_content
            .as_ref()
            .ok_or_else(|| anyhow!("Header content not provided"))?;

        // Check if header already exists
        if content.trim_start().starts_with(header_content.trim()) {
            return Ok(FileChange {
                file_path: file_path.clone(),
                change_type: ChangeType::HeaderAddition,
                old_content: Some(content.to_string()),
                new_content: content.to_string(),
                line_changes: Vec::new(),
            });
        }

        let new_content = format!("{}\n\n{}", header_content, content);
        let line_changes = vec![
            LineChange {
                line_number: 1,
                change_type: LineChangeType::Insert,
                old_line: None,
                new_line: header_content.clone(),
            }
        ];

        Ok(FileChange {
            file_path: file_path.clone(),
            change_type: ChangeType::HeaderAddition,
            old_content: Some(content.to_string()),
            new_content,
            line_changes,
        })
    }

    fn get_processor_name(&self) -> &str {
        "HeaderProcessor"
    }
}

pub struct PatternProcessor;

impl OperationProcessor for PatternProcessor {
    fn process_file(&self, file_path: &PathBuf, content: &str, params: &BatchParams) -> Result<FileChange> {
        let pattern = params.pattern
            .as_ref()
            .ok_or_else(|| anyhow!("Pattern not provided"))?;
        let replacement = params.replacement
            .as_ref()
            .ok_or_else(|| anyhow!("Replacement not provided"))?;

        let regex = Regex::new(pattern)
            .map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;

        let mut line_changes = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if regex.is_match(line) {
                let new_line = regex.replace_all(line, replacement).to_string();
                line_changes.push(LineChange {
                    line_number: line_num + 1,
                    change_type: LineChangeType::Modify,
                    old_line: Some(line.to_string()),
                    new_line: new_line.clone(),
                });
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }

        let new_content = new_lines.join("\n");

        Ok(FileChange {
            file_path: file_path.clone(),
            change_type: ChangeType::PatternUpdate,
            old_content: Some(content.to_string()),
            new_content,
            line_changes,
        })
    }

    fn get_processor_name(&self) -> &str {
        "PatternProcessor"
    }
}

pub struct ImportProcessor;

impl OperationProcessor for ImportProcessor {
    fn process_file(&self, file_path: &PathBuf, content: &str, _params: &BatchParams) -> Result<FileChange> {
        // This is a simplified import processor
        // In practice, this would be much more sophisticated and language-specific
        
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut line_changes = Vec::new();
        let mut imports_section_end = 0;

        // Find the end of imports section
        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
                imports_section_end = i + 1;
            } else if !line.trim().is_empty() && imports_section_end > 0 {
                break;
            }
        }

        // Process lines
        for (line_num, line) in lines.iter().enumerate() {
            // Example: Update import statements
            if line.trim().starts_with("import ") && line.contains("old_module") {
                let new_line = line.replace("old_module", "new_module");
                line_changes.push(LineChange {
                    line_number: line_num + 1,
                    change_type: LineChangeType::Modify,
                    old_line: Some(line.to_string()),
                    new_line: new_line.clone(),
                });
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }

        let new_content = new_lines.join("\n");

        Ok(FileChange {
            file_path: file_path.clone(),
            change_type: ChangeType::ContentReplacement,
            old_content: Some(content.to_string()),
            new_content,
            line_changes,
        })
    }

    fn get_processor_name(&self) -> &str {
        "ImportProcessor"
    }
}

pub struct FormattingProcessor;

impl OperationProcessor for FormattingProcessor {
    fn process_file(&self, file_path: &PathBuf, content: &str, _params: &BatchParams) -> Result<FileChange> {
        // Simple formatting: normalize whitespace and indentation
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut line_changes = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            // Remove trailing whitespace
            let trimmed = line.trim_end();
            
            if trimmed != *line {
                line_changes.push(LineChange {
                    line_number: line_num + 1,
                    change_type: LineChangeType::Modify,
                    old_line: Some(line.to_string()),
                    new_line: trimmed.to_string(),
                });
            }
            
            new_lines.push(trimmed.to_string());
        }

        let new_content = new_lines.join("\n");

        Ok(FileChange {
            file_path: file_path.clone(),
            change_type: ChangeType::FormatChange,
            old_content: Some(content.to_string()),
            new_content,
            line_changes,
        })
    }

    fn get_processor_name(&self) -> &str {
        "FormattingProcessor"
    }
}

pub struct LicenseProcessor;

impl OperationProcessor for LicenseProcessor {
    fn process_file(&self, file_path: &PathBuf, content: &str, _params: &BatchParams) -> Result<FileChange> {
        let license_header = self.get_license_header(file_path)?;
        
        // Check if license already exists
        if content.contains("Copyright") || content.contains("LICENSE") {
            return Ok(FileChange {
                file_path: file_path.clone(),
                change_type: ChangeType::HeaderAddition,
                old_content: Some(content.to_string()),
                new_content: content.to_string(),
                line_changes: Vec::new(),
            });
        }

        let new_content = format!("{}\n\n{}", license_header, content);
        let line_changes = vec![
            LineChange {
                line_number: 1,
                change_type: LineChangeType::Insert,
                old_line: None,
                new_line: license_header,
            }
        ];

        Ok(FileChange {
            file_path: file_path.clone(),
            change_type: ChangeType::HeaderAddition,
            old_content: Some(content.to_string()),
            new_content,
            line_changes,
        })
    }

    fn get_processor_name(&self) -> &str {
        "LicenseProcessor"
    }
}

impl LicenseProcessor {
    fn get_license_header(&self, file_path: &PathBuf) -> Result<String> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let comment_style = match extension {
            "rs" | "js" | "ts" | "cpp" | "c" | "h" | "java" => "//",
            "py" | "sh" => "#",
            _ => "//",
        };

        let year = chrono::Utc::now().year();
        
        Ok(format!(
            "{} Copyright {} Your Organization\n{} Licensed under the MIT License",
            comment_style, year, comment_style
        ))
    }
}

// Utility functions
impl BatchProcessor {
    pub fn preview_changes(&self, operation: &BatchOperation) -> String {
        let mut preview = String::new();
        
        preview.push_str(&format!("Batch Operation: {:?}\n", operation.operation_type));
        preview.push_str(&format!("Files to process: {}\n", operation.target_files.len()));
        preview.push_str(&format!("Total changes: {}\n\n", operation.summary.total_changes));

        for (i, change) in operation.changes.iter().take(5).enumerate() {
            preview.push_str(&format!("{}. {}\n", i + 1, change.file_path.display()));
            preview.push_str(&format!("   Change type: {:?}\n", change.change_type));
            preview.push_str(&format!("   Line changes: {}\n", change.line_changes.len()));
        }

        if operation.changes.len() > 5 {
            preview.push_str(&format!("... and {} more files\n", operation.changes.len() - 5));
        }

        preview
    }

    pub fn validate_params(&self, operation_type: &BatchOperationType, params: &BatchParams) -> Result<()> {
        match operation_type {
            BatchOperationType::AddHeaders => {
                if params.header_content.is_none() {
                    return Err(anyhow!("Header content is required for AddHeaders operation"));
                }
            }
            BatchOperationType::ReplacePattern => {
                if params.pattern.is_none() || params.replacement.is_none() {
                    return Err(anyhow!("Pattern and replacement are required for ReplacePattern operation"));
                }
                // Validate regex
                if let Some(pattern) = &params.pattern {
                    Regex::new(pattern)
                        .map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                }
            }
            _ => {} // Other operations may have different validation rules
        }
        
        Ok(())
    }
}