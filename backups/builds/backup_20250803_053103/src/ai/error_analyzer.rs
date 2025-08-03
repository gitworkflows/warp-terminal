use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub file_path: Option<PathBuf>,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
    pub message: String,
    pub context: String,
    pub suggested_fixes: Vec<SuggestedFix>,
    pub related_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ErrorType {
    CompilationError,
    RuntimeError,
    SyntaxError,
    TypeError,
    ImportError,
    LintingError,
    TestFailure,
    PerformanceIssue,
    ConfigurationError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub description: String,
    pub code_change: Option<CodeChange>,
    pub confidence: f64,
    pub automated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub old_content: String,
    pub new_content: String,
}

pub struct ErrorAnalyzer {
    parsers: Vec<Box<dyn ErrorParser>>,
    fix_generators: HashMap<ErrorType, Box<dyn FixGenerator>>,
}

pub trait ErrorParser: Send + Sync {
    fn can_parse(&self, output: &str) -> bool;
    fn parse(&self, output: &str) -> Vec<ErrorAnalysis>;
    fn get_parser_name(&self) -> &str;
}

pub trait FixGenerator: Send + Sync {
    fn generate_fixes(&self, error: &ErrorAnalysis) -> Vec<SuggestedFix>;
    fn get_generator_name(&self) -> &str;
}

impl ErrorAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            parsers: Vec::new(),
            fix_generators: HashMap::new(),
        };

        analyzer.add_default_parsers();
        analyzer.add_default_fix_generators();
        
        analyzer
    }

    pub fn analyze_terminal_output(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut all_errors = Vec::new();

        for parser in &self.parsers {
            if parser.can_parse(output) {
                let mut errors = parser.parse(output);
                
                // Generate fixes for each error
                for error in &mut errors {
                    if let Some(generator) = self.fix_generators.get(&error.error_type) {
                        error.suggested_fixes = generator.generate_fixes(error);
                    }
                }
                
                all_errors.extend(errors);
            }
        }

        // Remove duplicates and sort by severity
        self.deduplicate_and_sort(all_errors)
    }

    fn deduplicate_and_sort(&self, mut errors: Vec<ErrorAnalysis>) -> Vec<ErrorAnalysis> {
        // Sort by severity (critical first)
        errors.sort_by(|a, b| {
            let severity_order = |s: &ErrorSeverity| match s {
                ErrorSeverity::Critical => 0,
                ErrorSeverity::High => 1,
                ErrorSeverity::Medium => 2,
                ErrorSeverity::Low => 3,
                ErrorSeverity::Warning => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        // Remove duplicates based on message and file path
        let mut seen = std::collections::HashSet::new();
        errors.retain(|error| {
            let key = (error.message.clone(), error.file_path.clone());
            seen.insert(key)
        });

        errors
    }

    fn add_default_parsers(&mut self) {
        self.parsers.push(Box::new(TypeScriptErrorParser));
        self.parsers.push(Box::new(RustErrorParser));
        self.parsers.push(Box::new(JavaScriptErrorParser));
        self.parsers.push(Box::new(PythonErrorParser));
        self.parsers.push(Box::new(ESLintParser));
        self.parsers.push(Box::new(JestTestParser));
        self.parsers.push(Box::new(CargoErrorParser));
    }

    fn add_default_fix_generators(&mut self) {
        self.fix_generators.insert(ErrorType::CompilationError, Box::new(CompilationFixGenerator));
        self.fix_generators.insert(ErrorType::RuntimeError, Box::new(RuntimeFixGenerator));
        self.fix_generators.insert(ErrorType::SyntaxError, Box::new(SyntaxFixGenerator));
        self.fix_generators.insert(ErrorType::TypeError, Box::new(TypeFixGenerator));
        self.fix_generators.insert(ErrorType::ImportError, Box::new(ImportFixGenerator));
        self.fix_generators.insert(ErrorType::LintingError, Box::new(LintingFixGenerator));
    }
}

// TypeScript Error Parser
pub struct TypeScriptErrorParser;

impl ErrorParser for TypeScriptErrorParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("error TS") || output.contains("TypeScript")
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        let ts_regex = Regex::new(r"(?m)^(.+):(\d+):(\d+) - error TS(\d+): (.+)$").unwrap();
        
        for caps in ts_regex.captures_iter(output) {
            let file_path = PathBuf::from(&caps[1]);
            let line = caps[2].parse::<usize>().unwrap_or(1);
            let column = caps[3].parse::<usize>().unwrap_or(1);
            let error_code = &caps[4];
            let message = &caps[5];

            let severity = match error_code {
                "2304" | "2322" | "2339" => ErrorSeverity::High, // Cannot find name, Type not assignable, Property does not exist
                "2571" | "2307" => ErrorSeverity::Medium, // Object is possibly null, Cannot find module
                _ => ErrorSeverity::Medium,
            };

            errors.push(ErrorAnalysis {
                error_type: ErrorType::CompilationError,
                severity,
                file_path: Some(file_path),
                line_number: Some(line),
                column_number: Some(column),
                message: format!("TS{}: {}", error_code, message),
                context: self.extract_context(output, &caps[0]),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "TypeScriptErrorParser"
    }
}

impl TypeScriptErrorParser {
    fn extract_context(&self, output: &str, error_line: &str) -> String {
        // Extract a few lines around the error for context
        let lines: Vec<&str> = output.lines().collect();
        if let Some(pos) = lines.iter().position(|&line| line == error_line) {
            let start = pos.saturating_sub(2);
            let end = (pos + 3).min(lines.len());
            lines[start..end].join("\n")
        } else {
            error_line.to_string()
        }
    }
}

// Rust Error Parser
pub struct RustErrorParser;

impl ErrorParser for RustErrorParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("error[E") || output.contains("rustc")
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        let rust_regex = Regex::new(r"(?m)^error\[E(\d+)\]: (.+)$").unwrap();
        
        for caps in rust_regex.captures_iter(output) {
            let error_code = &caps[1];
            let message = &caps[2];

            let severity = match error_code {
                "0425" | "0308" | "0382" => ErrorSeverity::High,
                "0499" | "0502" => ErrorSeverity::Medium,
                _ => ErrorSeverity::Medium,
            };

            errors.push(ErrorAnalysis {
                error_type: ErrorType::CompilationError,
                severity,
                file_path: None, // Would need more complex parsing
                line_number: None,
                column_number: None,
                message: format!("E{}: {}", error_code, message),
                context: String::new(),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "RustErrorParser"
    }
}

// JavaScript Runtime Error Parser
pub struct JavaScriptErrorParser;

impl ErrorParser for JavaScriptErrorParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("Error:") && (output.contains("at ") || output.contains("node:"))
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        let js_error_regex = Regex::new(r"(?m)^(\w+Error): (.+)\n\s+at .+ \((.+):(\d+):(\d+)\)$").unwrap();
        
        for caps in js_error_regex.captures_iter(output) {
            let error_type_str = &caps[1];
            let message = &caps[2];
            let file_path = PathBuf::from(&caps[3]);
            let line = caps[4].parse::<usize>().unwrap_or(1);
            let column = caps[5].parse::<usize>().unwrap_or(1);

            let error_type = match error_type_str {
                "TypeError" => ErrorType::TypeError,
                "SyntaxError" => ErrorType::SyntaxError,
                "ReferenceError" => ErrorType::RuntimeError,
                _ => ErrorType::RuntimeError,
            };

            let severity = match error_type_str {
                "SyntaxError" => ErrorSeverity::High,
                "TypeError" | "ReferenceError" => ErrorSeverity::Medium,
                _ => ErrorSeverity::Low,
            };

            errors.push(ErrorAnalysis {
                error_type,
                severity,
                file_path: Some(file_path),
                line_number: Some(line),
                column_number: Some(column),
                message: format!("{}: {}", error_type_str, message),
                context: String::new(),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "JavaScriptErrorParser"
    }
}

// Python Error Parser
pub struct PythonErrorParser;

impl ErrorParser for PythonErrorParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("Traceback") || output.contains("File \"")
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        let python_error_regex = Regex::new(r"(?m)^(\w+Error): (.+)$").unwrap();
        let file_regex = Regex::new(r#"File "(.+)", line (\d+)"#).unwrap();
        
        for caps in python_error_regex.captures_iter(output) {
            let error_type_str = &caps[1];
            let message = &caps[2];

            let mut file_path = None;
            let mut line_number = None;

            // Try to find file and line info
            if let Some(file_caps) = file_regex.captures(output) {
                file_path = Some(PathBuf::from(&file_caps[1]));
                line_number = file_caps[2].parse::<usize>().ok();
            }

            let error_type = match error_type_str {
                "SyntaxError" => ErrorType::SyntaxError,
                "TypeError" => ErrorType::TypeError,
                "ImportError" | "ModuleNotFoundError" => ErrorType::ImportError,
                _ => ErrorType::RuntimeError,
            };

            errors.push(ErrorAnalysis {
                error_type,
                severity: ErrorSeverity::Medium,
                file_path,
                line_number,
                column_number: None,
                message: format!("{}: {}", error_type_str, message),
                context: String::new(),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "PythonErrorParser"
    }
}

// ESLint Parser
pub struct ESLintParser;

impl ErrorParser for ESLintParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("eslint") || Regex::new(r"(?m)^.+:\d+:\d+: .+ \(.+\)$").unwrap().is_match(output)
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        let eslint_regex = Regex::new(r"(?m)^(.+):(\d+):(\d+): (.+) \((.+)\)$").unwrap();
        
        for caps in eslint_regex.captures_iter(output) {
            let file_path = PathBuf::from(&caps[1]);
            let line = caps[2].parse::<usize>().unwrap_or(1);
            let column = caps[3].parse::<usize>().unwrap_or(1);
            let _message = &caps[4];
            let rule = &caps[5];

            errors.push(ErrorAnalysis {
                error_type: ErrorType::LintingError,
                severity: ErrorSeverity::Warning,
                file_path: Some(file_path),
                line_number: Some(line),
                column_number: Some(column),
                message: format!("ESLint ({})", rule),
                context: String::new(),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "ESLintParser"
    }
}

// Jest Test Parser
pub struct JestTestParser;

impl ErrorParser for JestTestParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("Jest") || output.contains("✕") || output.contains("FAIL")
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        let jest_fail_regex = Regex::new(r"(?m)^\s+✕ (.+) \((\d+) ms\)$").unwrap();
        
        for caps in jest_fail_regex.captures_iter(output) {
            let test_name = &caps[1];

            errors.push(ErrorAnalysis {
                error_type: ErrorType::TestFailure,
                severity: ErrorSeverity::Medium,
                file_path: None,
                line_number: None,
                column_number: None,
                message: format!("Test failed: {}", test_name),
                context: String::new(),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "JestTestParser"
    }
}

// Cargo Error Parser
pub struct CargoErrorParser;

impl ErrorParser for CargoErrorParser {
    fn can_parse(&self, output: &str) -> bool {
        output.contains("cargo") && output.contains("error:")
    }

    fn parse(&self, output: &str) -> Vec<ErrorAnalysis> {
        let mut errors = Vec::new();
        
        if output.contains("could not compile") {
            errors.push(ErrorAnalysis {
                error_type: ErrorType::CompilationError,
                severity: ErrorSeverity::High,
                file_path: None,
                line_number: None,
                column_number: None,
                message: "Cargo compilation failed".to_string(),
                context: String::new(),
                suggested_fixes: Vec::new(),
                related_errors: Vec::new(),
            });
        }

        errors
    }

    fn get_parser_name(&self) -> &str {
        "CargoErrorParser"
    }
}

// Fix Generators
pub struct CompilationFixGenerator;

impl FixGenerator for CompilationFixGenerator {
    fn generate_fixes(&self, error: &ErrorAnalysis) -> Vec<SuggestedFix> {
        let mut fixes = Vec::new();

        if error.message.contains("Cannot find name") {
            fixes.push(SuggestedFix {
                description: "Add import statement for the missing identifier".to_string(),
                code_change: None,
                confidence: 0.8,
                automated: false,
            });
        }

        if error.message.contains("is not assignable to type") {
            fixes.push(SuggestedFix {
                description: "Add type assertion or fix type compatibility".to_string(),
                code_change: None,
                confidence: 0.7,
                automated: false,
            });
        }

        fixes
    }

    fn get_generator_name(&self) -> &str {
        "CompilationFixGenerator"
    }
}

pub struct RuntimeFixGenerator;

impl FixGenerator for RuntimeFixGenerator {
    fn generate_fixes(&self, error: &ErrorAnalysis) -> Vec<SuggestedFix> {
        let mut fixes = Vec::new();

        if error.message.contains("ReferenceError") {
            fixes.push(SuggestedFix {
                description: "Check variable declaration and scope".to_string(),
                code_change: None,
                confidence: 0.6,
                automated: false,
            });
        }

        fixes
    }

    fn get_generator_name(&self) -> &str {
        "RuntimeFixGenerator"
    }
}

pub struct SyntaxFixGenerator;

impl FixGenerator for SyntaxFixGenerator {
    fn generate_fixes(&self, _error: &ErrorAnalysis) -> Vec<SuggestedFix> {
        vec![SuggestedFix {
            description: "Check syntax: missing brackets, quotes, or semicolons".to_string(),
            code_change: None,
            confidence: 0.5,
            automated: false,
        }]
    }

    fn get_generator_name(&self) -> &str {
        "SyntaxFixGenerator"
    }
}

pub struct TypeFixGenerator;

impl FixGenerator for TypeFixGenerator {
    fn generate_fixes(&self, _error: &ErrorAnalysis) -> Vec<SuggestedFix> {
        vec![SuggestedFix {
            description: "Verify object types and method signatures".to_string(),
            code_change: None,
            confidence: 0.6,
            automated: false,
        }]
    }

    fn get_generator_name(&self) -> &str {
        "TypeFixGenerator"
    }
}

pub struct ImportFixGenerator;

impl FixGenerator for ImportFixGenerator {
    fn generate_fixes(&self, _error: &ErrorAnalysis) -> Vec<SuggestedFix> {
        vec![SuggestedFix {
            description: "Check module path and ensure dependency is installed".to_string(),
            code_change: None,
            confidence: 0.7,
            automated: false,
        }]
    }

    fn get_generator_name(&self) -> &str {
        "ImportFixGenerator"
    }
}

pub struct LintingFixGenerator;

impl FixGenerator for LintingFixGenerator {
    fn generate_fixes(&self, error: &ErrorAnalysis) -> Vec<SuggestedFix> {
        let mut fixes = Vec::new();

        if error.message.contains("no-unused-vars") {
            fixes.push(SuggestedFix {
                description: "Remove unused variable or prefix with underscore".to_string(),
                code_change: None,
                confidence: 0.9,
                automated: true,
            });
        }

        if error.message.contains("prefer-const") {
            fixes.push(SuggestedFix {
                description: "Change 'let' to 'const' for variables that aren't reassigned".to_string(),
                code_change: None,
                confidence: 0.95,
                automated: true,
            });
        }

        fixes
    }

    fn get_generator_name(&self) -> &str {
        "LintingFixGenerator"
    }
}