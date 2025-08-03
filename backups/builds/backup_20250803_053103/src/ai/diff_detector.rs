use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOpportunity {
    pub id: String,
    pub file_path: PathBuf,
    pub opportunity_type: OpportunityType,
    pub confidence: f64,
    pub description: String,
    pub suggested_action: String,
    pub line_range: Option<(usize, usize)>,
    pub context: DiffContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    CodeImprovement,
    ErrorFix,
    StyleInconsistency,
    SecurityIssue,
    PerformanceOptimization,
    Refactoring,
    Documentation,
    TypeSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffContext {
    pub surrounding_code: String,
    pub file_type: String,
    pub project_patterns: Vec<String>,
    pub related_files: Vec<PathBuf>,
}

pub struct DiffDetector {
    patterns: HashMap<String, Vec<DiffPattern>>,
    analyzers: Vec<Box<dyn CodeAnalyzer>>,
}

#[derive(Debug, Clone)]
pub struct DiffPattern {
    pub name: String,
    pub regex: Regex,
    pub opportunity_type: OpportunityType,
    pub description: String,
    pub suggested_fix: String,
    pub confidence: f64,
}

pub trait CodeAnalyzer: Send + Sync {
    fn analyze(&self, content: &str, file_path: &PathBuf) -> Vec<DiffOpportunity>;
    fn get_analyzer_name(&self) -> &str;
}

impl DiffDetector {
    pub fn new() -> Result<Self> {
        let mut detector = Self {
            patterns: HashMap::new(),
            analyzers: Vec::new(),
        };

        detector.load_default_patterns()?;
        detector.add_default_analyzers();
        
        Ok(detector)
    }

    pub fn analyze_file(&self, file_path: PathBuf, content: &str) -> Result<Vec<DiffOpportunity>> {
        let mut opportunities = Vec::new();

        // Pattern-based detection
        opportunities.extend(self.detect_pattern_opportunities(&file_path, content)?);

        // Analyzer-based detection
        for analyzer in &self.analyzers {
            opportunities.extend(analyzer.analyze(content, &file_path));
        }

        // Sort by confidence
        opportunities.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(opportunities)
    }

    pub fn analyze_terminal_output(&self, output: &str) -> Vec<DiffOpportunity> {
        let mut opportunities = Vec::new();

        // Detect compilation errors
        if let Some(error_opportunity) = self.detect_compilation_error(output) {
            opportunities.push(error_opportunity);
        }

        // Detect runtime errors
        if let Some(runtime_opportunity) = self.detect_runtime_error(output) {
            opportunities.push(runtime_opportunity);
        }

        // Detect test failures
        opportunities.extend(self.detect_test_failures(output));

        // Detect linting issues
        opportunities.extend(self.detect_linting_issues(output));

        opportunities
    }

    fn detect_pattern_opportunities(&self, file_path: &PathBuf, content: &str) -> Result<Vec<DiffOpportunity>> {
        let mut opportunities = Vec::new();
        let file_extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if let Some(patterns) = self.patterns.get(file_extension) {
            for pattern in patterns {
                for mat in pattern.regex.find_iter(content) {
                    let line_number = content[..mat.start()].lines().count() + 1;
                    let surrounding_lines = self.get_surrounding_lines(content, line_number, 3);
                    
                    opportunities.push(DiffOpportunity {
                        id: format!("{}_{}", pattern.name, line_number),
                        file_path: file_path.clone(),
                        opportunity_type: pattern.opportunity_type.clone(),
                        confidence: pattern.confidence,
                        description: pattern.description.clone(),
                        suggested_action: pattern.suggested_fix.clone(),
                        line_range: Some((line_number, line_number)),
                        context: DiffContext {
                            surrounding_code: surrounding_lines,
                            file_type: file_extension.to_string(),
                            project_patterns: Vec::new(),
                            related_files: Vec::new(),
                        },
                    });
                }
            }
        }

        Ok(opportunities)
    }

    fn detect_compilation_error(&self, output: &str) -> Option<DiffOpportunity> {
        // TypeScript/JavaScript error patterns
        if let Some(caps) = Regex::new(r"(?m)^(.+):(\d+):(\d+) - error TS(\d+): (.+)$").unwrap().captures(output) {
            let file_path = PathBuf::from(&caps[1]);
            let line = caps[2].parse::<usize>().unwrap_or(1);
            let error_message = &caps[5];

            return Some(DiffOpportunity {
                id: format!("ts_error_{}", line),
                file_path,
                opportunity_type: OpportunityType::ErrorFix,
                confidence: 0.9,
                description: format!("TypeScript compilation error: {}", error_message),
                suggested_action: self.suggest_typescript_fix(error_message),
                line_range: Some((line, line)),
                context: DiffContext {
                    surrounding_code: String::new(),
                    file_type: "typescript".to_string(),
                    project_patterns: Vec::new(),
                    related_files: Vec::new(),
                },
            });
        }

        // Rust error patterns
        if let Some(caps) = Regex::new(r"(?m)^error\[E(\d+)\]: (.+)").unwrap().captures(output) {
            let error_code = &caps[1];
            let error_message = &caps[2];

            return Some(DiffOpportunity {
                id: format!("rust_error_{}", error_code),
                file_path: PathBuf::from("src/main.rs"), // Default, would need better parsing
                opportunity_type: OpportunityType::ErrorFix,
                confidence: 0.95,
                description: format!("Rust compiler error: {}", error_message),
                suggested_action: self.suggest_rust_fix(error_code, error_message),
                line_range: None,
                context: DiffContext {
                    surrounding_code: String::new(),
                    file_type: "rust".to_string(),
                    project_patterns: Vec::new(),
                    related_files: Vec::new(),
                },
            });
        }

        None
    }

    fn detect_runtime_error(&self, output: &str) -> Option<DiffOpportunity> {
        // JavaScript/Node.js error patterns
        if let Some(caps) = Regex::new(r"(?m)^(\w+Error): (.+)\n\s+at .+ \((.+):(\d+):(\d+)\)").unwrap().captures(output) {
            let error_type = &caps[1];
            let error_message = &caps[2];
            let file_path = PathBuf::from(&caps[3]);
            let line = caps[4].parse::<usize>().unwrap_or(1);

            return Some(DiffOpportunity {
                id: format!("runtime_error_{}", line),
                file_path,
                opportunity_type: OpportunityType::ErrorFix,
                confidence: 0.85,
                description: format!("{}: {}", error_type, error_message),
                suggested_action: self.suggest_runtime_fix(error_type, error_message),
                line_range: Some((line, line)),
                context: DiffContext {
                    surrounding_code: String::new(),
                    file_type: "javascript".to_string(),
                    project_patterns: Vec::new(),
                    related_files: Vec::new(),
                },
            });
        }

        None
    }

    fn detect_test_failures(&self, output: &str) -> Vec<DiffOpportunity> {
        let mut opportunities = Vec::new();

        // Jest test failures
        let jest_regex = Regex::new(r"(?m)^\s+✕ (.+) \((\d+) ms\)").unwrap();
        for caps in jest_regex.captures_iter(output) {
            let test_name = &caps[1];
            
            opportunities.push(DiffOpportunity {
                id: format!("test_failure_{}", test_name.replace(" ", "_")),
                file_path: PathBuf::from("test_file.js"), // Would need better parsing
                opportunity_type: OpportunityType::ErrorFix,
                confidence: 0.7,
                description: format!("Test failure: {}", test_name),
                suggested_action: "Review test expectations and implementation".to_string(),
                line_range: None,
                context: DiffContext {
                    surrounding_code: String::new(),
                    file_type: "javascript".to_string(),
                    project_patterns: Vec::new(),
                    related_files: Vec::new(),
                },
            });
        }

        opportunities
    }

    fn detect_linting_issues(&self, output: &str) -> Vec<DiffOpportunity> {
        let mut opportunities = Vec::new();

        // ESLint issues
        let eslint_regex = Regex::new(r"(?m)^(.+):(\d+):(\d+): (.+) \((.+)\)").unwrap();
        for caps in eslint_regex.captures_iter(output) {
            let file_path = PathBuf::from(&caps[1]);
            let line = caps[2].parse::<usize>().unwrap_or(1);
            let message = &caps[4];
            let rule = &caps[5];

            opportunities.push(DiffOpportunity {
                id: format!("eslint_{}_{}", rule, line),
                file_path,
                opportunity_type: OpportunityType::StyleInconsistency,
                confidence: 0.6,
                description: format!("ESLint: {}", message),
                suggested_action: self.suggest_eslint_fix(rule, message),
                line_range: Some((line, line)),
                context: DiffContext {
                    surrounding_code: String::new(),
                    file_type: "javascript".to_string(),
                    project_patterns: Vec::new(),
                    related_files: Vec::new(),
                },
            });
        }

        opportunities
    }

    fn suggest_typescript_fix(&self, error_message: &str) -> String {
        if error_message.contains("Cannot find name") {
            "Add import statement or declare the variable/type".to_string()
        } else if error_message.contains("Type") && error_message.contains("is not assignable") {
            "Check type compatibility and add type assertions if needed".to_string()
        } else if error_message.contains("Property") && error_message.contains("does not exist") {
            "Check object structure and property names".to_string()
        } else {
            "Review the error and TypeScript documentation".to_string()
        }
    }

    fn suggest_rust_fix(&self, error_code: &str, _error_message: &str) -> String {
        match error_code {
            "E0425" => "Variable not found - check spelling and scope".to_string(),
            "E0308" => "Type mismatch - ensure types are compatible".to_string(),
            "E0382" => "Use after move - consider cloning or borrowing".to_string(),
            "E0499" => "Multiple mutable borrows - restructure borrow usage".to_string(),
            _ => format!("Review Rust error E{} documentation", error_code),
        }
    }

    fn suggest_runtime_fix(&self, error_type: &str, _error_message: &str) -> String {
        match error_type {
            "ReferenceError" => "Check variable declarations and scope".to_string(),
            "TypeError" => "Verify object types and method calls".to_string(),
            "SyntaxError" => "Check code syntax and structure".to_string(),
            _ => "Review error message and stack trace".to_string(),
        }
    }

    fn suggest_eslint_fix(&self, rule: &str, _message: &str) -> String {
        match rule {
            "no-unused-vars" => "Remove unused variable or add underscore prefix".to_string(),
            "no-console" => "Replace console statements with proper logging".to_string(),
            "prefer-const" => "Change 'let' to 'const' for variables that aren't reassigned".to_string(),
            "eqeqeq" => "Use strict equality (===) instead of loose equality (==)".to_string(),
            _ => format!("Follow ESLint rule: {}", rule),
        }
    }

    fn get_surrounding_lines(&self, content: &str, target_line: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = target_line.saturating_sub(context_lines + 1);
        let end = (target_line + context_lines).min(lines.len());
        
        lines[start..end].join("\n")
    }

    fn load_default_patterns(&mut self) -> Result<()> {
        // JavaScript/TypeScript patterns
        let mut js_patterns = Vec::new();
        
        js_patterns.push(DiffPattern {
            name: "var_to_let".to_string(),
            regex: Regex::new(r"\bvar\s+")?,
            opportunity_type: OpportunityType::CodeImprovement,
            description: "Use 'let' or 'const' instead of 'var'".to_string(),
            suggested_fix: "Replace 'var' with 'let' or 'const'".to_string(),
            confidence: 0.8,
        });

        js_patterns.push(DiffPattern {
            name: "console_log".to_string(),
            regex: Regex::new(r"\bconsole\.log\(")?,
            opportunity_type: OpportunityType::CodeImprovement,
            description: "Console.log found in production code".to_string(),
            suggested_fix: "Replace with proper logging or remove".to_string(),
            confidence: 0.6,
        });

        js_patterns.push(DiffPattern {
            name: "loose_equality".to_string(),
            regex: Regex::new(r"[^=!]==([^=])")?,
            opportunity_type: OpportunityType::CodeImprovement,
            description: "Use strict equality (===) instead of loose equality (==)".to_string(),
            suggested_fix: "Replace '==' with '==='".to_string(),
            confidence: 0.7,
        });

        self.patterns.insert("js".to_string(), js_patterns.clone());
        self.patterns.insert("ts".to_string(), js_patterns);

        // Python patterns
        let mut py_patterns = Vec::new();
        
        py_patterns.push(DiffPattern {
            name: "print_statement".to_string(),
            regex: Regex::new(r"\bprint\(")?,
            opportunity_type: OpportunityType::CodeImprovement,
            description: "Print statement found in production code".to_string(),
            suggested_fix: "Replace with proper logging".to_string(),
            confidence: 0.6,
        });

        self.patterns.insert("py".to_string(), py_patterns);

        Ok(())
    }

    fn add_default_analyzers(&mut self) {
        self.analyzers.push(Box::new(SecurityAnalyzer));
        self.analyzers.push(Box::new(PerformanceAnalyzer));
        self.analyzers.push(Box::new(DocumentationAnalyzer));
    }
}

// Security analyzer
pub struct SecurityAnalyzer;

impl CodeAnalyzer for SecurityAnalyzer {
    fn analyze(&self, content: &str, file_path: &PathBuf) -> Vec<DiffOpportunity> {
        let mut opportunities = Vec::new();

        // Check for potential SQL injection
        if let Ok(sql_regex) = Regex::new(r#"["']\s*\+\s*\w+\s*\+\s*["']"#) {
            for mat in sql_regex.find_iter(content) {
                let line_number = content[..mat.start()].lines().count() + 1;
                opportunities.push(DiffOpportunity {
                    id: format!("sql_injection_{}", line_number),
                    file_path: file_path.clone(),
                    opportunity_type: OpportunityType::SecurityIssue,
                    confidence: 0.7,
                    description: "Potential SQL injection vulnerability".to_string(),
                    suggested_action: "Use parameterized queries or prepared statements".to_string(),
                    line_range: Some((line_number, line_number)),
                    context: DiffContext {
                        surrounding_code: String::new(),
                        file_type: file_path.extension().unwrap_or_default().to_string_lossy().to_string(),
                        project_patterns: Vec::new(),
                        related_files: Vec::new(),
                    },
                });
            }
        }

        opportunities
    }

    fn get_analyzer_name(&self) -> &str {
        "SecurityAnalyzer"
    }
}

// Performance analyzer
pub struct PerformanceAnalyzer;

impl CodeAnalyzer for PerformanceAnalyzer {
    fn analyze(&self, content: &str, file_path: &PathBuf) -> Vec<DiffOpportunity> {
        let mut opportunities = Vec::new();

        // Check for inefficient loops
        if let Ok(loop_regex) = Regex::new(r"for\s*\(\s*\w+\s*=\s*0\s*;\s*\w+\s*<\s*\w+\.length\s*;\s*\w+\+\+\s*\)") {
            for mat in loop_regex.find_iter(content) {
                let line_number = content[..mat.start()].lines().count() + 1;
                opportunities.push(DiffOpportunity {
                    id: format!("inefficient_loop_{}", line_number),
                    file_path: file_path.clone(),
                    opportunity_type: OpportunityType::PerformanceOptimization,
                    confidence: 0.6,
                    description: "Inefficient array length access in loop".to_string(),
                    suggested_action: "Cache array length or use for...of loop".to_string(),
                    line_range: Some((line_number, line_number)),
                    context: DiffContext {
                        surrounding_code: String::new(),
                        file_type: file_path.extension().unwrap_or_default().to_string_lossy().to_string(),
                        project_patterns: Vec::new(),
                        related_files: Vec::new(),
                    },
                });
            }
        }

        opportunities
    }

    fn get_analyzer_name(&self) -> &str {
        "PerformanceAnalyzer"
    }
}

// Documentation analyzer
pub struct DocumentationAnalyzer;

impl CodeAnalyzer for DocumentationAnalyzer {
    fn analyze(&self, content: &str, file_path: &PathBuf) -> Vec<DiffOpportunity> {
        let mut opportunities = Vec::new();

        // Check for functions without documentation
        if let Ok(func_regex) = Regex::new(r"(?m)^(?:\s*export\s+)?(?:async\s+)?function\s+(\w+)\s*\(") {
            for caps in func_regex.captures_iter(content) {
                let line_number = content[..caps.get(0).unwrap().start()].lines().count() + 1;
                let func_name = &caps[1];
                
                // Check if there's a comment block above
                let lines: Vec<&str> = content.lines().collect();
                let has_doc = if line_number > 1 {
                    let prev_line = lines.get(line_number - 2).unwrap_or(&"");
                    prev_line.trim().starts_with("/**") || prev_line.trim().starts_with("//")
                } else {
                    false
                };

                if !has_doc {
                    opportunities.push(DiffOpportunity {
                        id: format!("missing_doc_{}", func_name),
                        file_path: file_path.clone(),
                        opportunity_type: OpportunityType::Documentation,
                        confidence: 0.4,
                        description: format!("Function '{}' lacks documentation", func_name),
                        suggested_action: "Add JSDoc or inline comments".to_string(),
                        line_range: Some((line_number, line_number)),
                        context: DiffContext {
                            surrounding_code: String::new(),
                            file_type: file_path.extension().unwrap_or_default().to_string_lossy().to_string(),
                            project_patterns: Vec::new(),
                            related_files: Vec::new(),
                        },
                    });
                }
            }
        }

        opportunities
    }

    fn get_analyzer_name(&self) -> &str {
        "DocumentationAnalyzer"
    }
}
