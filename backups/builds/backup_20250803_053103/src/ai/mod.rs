pub mod code_generator;
pub mod diff_detector;
pub mod error_analyzer;
pub mod batch_processor;
pub mod ai_integrator;

pub use code_generator::{CodeChange, AICodeGenerator as CodeGenerator};
pub use diff_detector::{DiffDetector, DiffOpportunity, OpportunityType};
pub use error_analyzer::{ErrorAnalysis, ErrorAnalyzer, ErrorSeverity, ErrorType, SuggestedFix};
pub use batch_processor::{BatchOperation, BatchOperationType, BatchProcessor, BatchParams, ChangeType, FileChange, LineChange, LineChangeType};
pub use ai_integrator::{AIIntegrator};
