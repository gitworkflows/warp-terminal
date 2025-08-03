use crate::ai::batch_processor::{BatchProcessor, BatchParams, BatchOperationType};
use anyhow::Result;
use std::path::PathBuf;

pub struct AIIntegrator {
    pub processor: BatchProcessor,
}

impl AIIntegrator {
    pub fn new() -> Self {
        Self {
            processor: BatchProcessor::new(),
        }
    }

    pub async fn perform_batch_operation(&self, operation_type: BatchOperationType, directory: &str, params: BatchParams) -> Result<()> {
        let directory_path = PathBuf::from(directory);
        let operation = self.processor.execute_batch_operation(operation_type, &directory_path, params).await?;
        println!("Batch operation completed: {} files processed, {} errors", operation.summary.files_processed, operation.summary.errors.len());

        if !operation.summary.errors.is_empty() {
            println!("Errors:");
            for error in operation.summary.errors {
                println!("- {}", error);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::batch_processor::BatchOperationType::*;

    #[tokio::test]
    async fn test_batch_operation() {
        let integrator = AIIntegrator::new();
        let params = BatchParams {
            pattern: Some("foo".to_string()),
            replacement: Some("bar".to_string()),
            header_content: None,
            target_extensions: vec!["rs".to_string()],
            exclude_patterns: vec![],
            dry_run: true,
            preserve_formatting: false,
        };

        let result = integrator.perform_batch_operation(ReplacePattern, "./src", params).await;
        assert!(result.is_ok());
    }
}
