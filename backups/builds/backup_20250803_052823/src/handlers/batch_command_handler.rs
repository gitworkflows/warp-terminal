use crate::ai::batch_processor::{BatchProcessor, BatchParams, BatchOperationType};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub struct BatchCommandHandler {
    processor: Arc<BatchProcessor>,
}
impl BatchCommandHandler {
    pub fn new() -> Self {
        Self {
            processor: Arc::new(BatchProcessor::new()),
        }
    }

    pub async fn handle_batch_command(&self, command_id: &str, directory: &str) -> Result<String> {
        match command_id {
            "batch.add_headers" => {
                self.handle_add_headers(directory).await
            }
            "batch.replace_pattern" => {
                self.handle_replace_pattern(directory).await
            }
            "batch.update_imports" => {
                self.handle_update_imports(directory).await
            }
            "batch.format_code" => {
                self.handle_format_code(directory).await
            }
            "batch.add_license" => {
                self.handle_add_license(directory).await
            }
            _ => Err(anyhow::anyhow!("Unknown batch command: {}", command_id)),
        }
    }

    async fn handle_add_headers(&self, directory: &str) -> Result<String> {
        let params = BatchParams {
            pattern: None,
            replacement: None,
            header_content: Some("// Generated header content".to_string()),
            target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string()],
            exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
            dry_run: false,
            preserve_formatting: true,
        };

        let directory_path = PathBuf::from(directory);
        let operation = self.processor.execute_batch_operation(
            BatchOperationType::AddHeaders,
            &directory_path,
            params,
        ).await?;

        Ok(format!(
            "Added headers to {} files with {} changes",
            operation.summary.files_changed,
            operation.summary.total_changes
        ))
    }

    async fn handle_replace_pattern(&self, directory: &str) -> Result<String> {
        let params = BatchParams {
            pattern: Some(r"console\.log\(.*\)".to_string()),
            replacement: Some("// Removed console.log".to_string()),
            header_content: None,
            target_extensions: vec!["js".to_string(), "ts".to_string()],
            exclude_patterns: vec!["node_modules/".to_string()],
            dry_run: false,
            preserve_formatting: true,
        };

        let directory_path = PathBuf::from(directory);
        let operation = self.processor.execute_batch_operation(
            BatchOperationType::ReplacePattern,
            &directory_path,
            params,
        ).await?;

        Ok(format!(
            "Replaced patterns in {} files with {} changes",
            operation.summary.files_changed,
            operation.summary.total_changes
        ))
    }

    async fn handle_update_imports(&self, directory: &str) -> Result<String> {
        let params = BatchParams {
            pattern: None,
            replacement: None,
            header_content: None,
            target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
            exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string(), "__pycache__/".to_string()],
            dry_run: false,
            preserve_formatting: true,
        };

        let directory_path = PathBuf::from(directory);
        let operation = self.processor.execute_batch_operation(
            BatchOperationType::UpdateImports,
            &directory_path,
            params,
        ).await?;

        Ok(format!(
            "Updated imports in {} files with {} changes",
            operation.summary.files_changed,
            operation.summary.total_changes
        ))
    }

    async fn handle_format_code(&self, directory: &str) -> Result<String> {
        let params = BatchParams {
            pattern: None,
            replacement: None,
            header_content: None,
            target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
            exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
            dry_run: false,
            preserve_formatting: false, // We want to format, so don't preserve
        };

        let directory_path = PathBuf::from(directory);
        let operation = self.processor.execute_batch_operation(
            BatchOperationType::CodeFormatting,
            &directory_path,
            params,
        ).await?;

        Ok(format!(
            "Formatted {} files with {} changes",
            operation.summary.files_changed,
            operation.summary.total_changes
        ))
    }

    async fn handle_add_license(&self, directory: &str) -> Result<String> {
        let params = BatchParams {
            pattern: None,
            replacement: None,
            header_content: None,
            target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
            exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
            dry_run: false,
            preserve_formatting: true,
        };

        let directory_path = PathBuf::from(directory);
        let operation = self.processor.execute_batch_operation(
            BatchOperationType::LicenseAddition,
            &directory_path,
            params,
        ).await?;

        Ok(format!(
            "Added license headers to {} files with {} changes",
            operation.summary.files_changed,
            operation.summary.total_changes
        ))
    }

    pub async fn preview_batch_operation(&self, command_id: &str, directory: &str) -> Result<String> {
        let params = self.get_params_for_command(command_id)?;
        let operation_type = self.get_operation_type_for_command(command_id)?;
        
        let directory_path = PathBuf::from(directory);
        let mut preview_params = params;
        preview_params.dry_run = true; // Always preview as dry run
        
        let operation = self.processor.execute_batch_operation(
            operation_type,
            &directory_path,
            preview_params,
        ).await?;

        Ok(self.processor.preview_changes(&operation))
    }

    fn get_params_for_command(&self, command_id: &str) -> Result<BatchParams> {
        match command_id {
            "batch.add_headers" => Ok(BatchParams {
                pattern: None,
                replacement: None,
                header_content: Some("// Generated header content".to_string()),
                target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string()],
                exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
                dry_run: true,
                preserve_formatting: true,
            }),
            "batch.replace_pattern" => Ok(BatchParams {
                pattern: Some(r"console\.log\(.*\)".to_string()),
                replacement: Some("// Removed console.log".to_string()),
                header_content: None,
                target_extensions: vec!["js".to_string(), "ts".to_string()],
                exclude_patterns: vec!["node_modules/".to_string()],
                dry_run: true,
                preserve_formatting: true,
            }),
            "batch.update_imports" => Ok(BatchParams {
                pattern: None,
                replacement: None,
                header_content: None,
                target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
                exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string(), "__pycache__/".to_string()],
                dry_run: true,
                preserve_formatting: true,
            }),
            "batch.format_code" => Ok(BatchParams {
                pattern: None,
                replacement: None,
                header_content: None,
                target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
                exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
                dry_run: true,
                preserve_formatting: false,
            }),
            "batch.add_license" => Ok(BatchParams {
                pattern: None,
                replacement: None,
                header_content: None,
                target_extensions: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
                exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
                dry_run: true,
                preserve_formatting: true,
            }),
            _ => Err(anyhow::anyhow!("Unknown batch command: {}", command_id)),
        }
    }

    fn get_operation_type_for_command(&self, command_id: &str) -> Result<BatchOperationType> {
        match command_id {
            "batch.add_headers" => Ok(BatchOperationType::AddHeaders),
            "batch.replace_pattern" => Ok(BatchOperationType::ReplacePattern),
            "batch.update_imports" => Ok(BatchOperationType::UpdateImports),
            "batch.format_code" => Ok(BatchOperationType::CodeFormatting),
            "batch.add_license" => Ok(BatchOperationType::LicenseAddition),
            _ => Err(anyhow::anyhow!("Unknown batch command: {}", command_id)),
        }
    }
}

impl Default for BatchCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_batch_command_handler() {
        let handler = BatchCommandHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.js");
        
        fs::write(&test_file, "console.log('test');").await.unwrap();

        let result = handler.preview_batch_operation(
            "batch.replace_pattern",
            temp_dir.path().to_str().unwrap(),
        ).await;

        assert!(result.is_ok());
        let preview = result.unwrap();
        assert!(preview.contains("Batch Operation: ReplacePattern"));
    }
}
