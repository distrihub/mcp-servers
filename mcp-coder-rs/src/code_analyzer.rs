use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    pub file_path: String,
    pub language: String,
    pub line_count: usize,
    pub function_count: usize,
    pub struct_count: usize,
    pub complexity_score: f64,
    pub dependencies: Vec<String>,
    pub issues: Vec<CodeIssue>,
    pub metrics: CodeMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeIssue {
    pub severity: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub cyclomatic_complexity: usize,
    pub lines_of_code: usize,
    pub comment_ratio: f64,
    pub maintainability_index: f64,
}

extern "C" {
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_javascript() -> Language;
    fn tree_sitter_python() -> Language;
}

pub struct CodeAnalyzer {
    rust_parser: Parser,
    js_parser: Parser,
    python_parser: Parser,
}

impl CodeAnalyzer {
    pub fn new() -> Result<Self> {
        let mut rust_parser = Parser::new();
        let mut js_parser = Parser::new();
        let mut python_parser = Parser::new();

        unsafe {
            rust_parser.set_language(tree_sitter_rust())?;
            js_parser.set_language(tree_sitter_javascript())?;
            python_parser.set_language(tree_sitter_python())?;
        }

        Ok(Self {
            rust_parser,
            js_parser,
            python_parser,
        })
    }

    pub async fn analyze_file(&self, file_path: &str, language: Option<&str>) -> Result<CodeAnalysisResult> {
        let content = fs::read_to_string(file_path).await?;
        let detected_language = language.unwrap_or_else(|| self.detect_language(file_path));

        let parser = match detected_language {
            "rust" => &self.rust_parser,
            "javascript" | "typescript" => &self.js_parser,
            "python" => &self.python_parser,
            _ => return Err(anyhow!("Unsupported language: {}", detected_language)),
        };

        let tree = parser.parse(&content, None)
            .ok_or_else(|| anyhow!("Failed to parse code"))?;

        let root_node = tree.root_node();
        let line_count = content.lines().count();
        
        // Basic analysis
        let function_count = self.count_functions(&root_node, detected_language);
        let struct_count = self.count_structs(&root_node, detected_language);
        let dependencies = self.extract_dependencies(&content, detected_language);
        let complexity_score = self.calculate_complexity(&root_node, detected_language);
        let issues = self.find_issues(&content, detected_language);
        let metrics = self.calculate_metrics(&content, &root_node, detected_language);

        Ok(CodeAnalysisResult {
            file_path: file_path.to_string(),
            language: detected_language.to_string(),
            line_count,
            function_count,
            struct_count,
            complexity_score,
            dependencies,
            issues,
            metrics,
        })
    }

    fn detect_language(&self, file_path: &str) -> &str {
        let path = Path::new(file_path);
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust",
            Some("js") | Some("mjs") => "javascript",
            Some("ts") => "typescript",
            Some("py") => "python",
            _ => "unknown",
        }
    }

    fn count_functions(&self, node: &tree_sitter::Node, language: &str) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();

        fn count_recursive(cursor: &mut tree_sitter::TreeCursor, language: &str) -> usize {
            let mut count = 0;
            
            if cursor.node().kind() == match language {
                "rust" => "function_item",
                "javascript" | "typescript" => "function_declaration",
                "python" => "function_definition",
                _ => return 0,
            } {
                count += 1;
            }

            if cursor.goto_first_child() {
                count += count_recursive(cursor, language);
                while cursor.goto_next_sibling() {
                    count += count_recursive(cursor, language);
                }
                cursor.goto_parent();
            }

            count
        }

        count_recursive(&mut cursor, language)
    }

    fn count_structs(&self, node: &tree_sitter::Node, language: &str) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();

        fn count_recursive(cursor: &mut tree_sitter::TreeCursor, language: &str) -> usize {
            let mut count = 0;
            
            if cursor.node().kind() == match language {
                "rust" => "struct_item",
                "javascript" | "typescript" => "class_declaration",
                "python" => "class_definition",
                _ => return 0,
            } {
                count += 1;
            }

            if cursor.goto_first_child() {
                count += count_recursive(cursor, language);
                while cursor.goto_next_sibling() {
                    count += count_recursive(cursor, language);
                }
                cursor.goto_parent();
            }

            count
        }

        count_recursive(&mut cursor, language)
    }

    fn extract_dependencies(&self, content: &str, language: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        match language {
            "rust" => {
                for line in content.lines() {
                    if line.trim().starts_with("use ") {
                        if let Some(dep) = line.trim().strip_prefix("use ").and_then(|s| s.split("::").next()) {
                            if !dep.starts_with("crate") && !dep.starts_with("super") && !dep.starts_with("self") {
                                dependencies.push(dep.to_string());
                            }
                        }
                    }
                }
            }
            "javascript" | "typescript" => {
                for line in content.lines() {
                    if line.trim().starts_with("import ") {
                        if let Some(from_pos) = line.find(" from ") {
                            let dep = &line[from_pos + 6..].trim().trim_matches('"').trim_matches('\'');
                            if !dep.starts_with('.') {
                                dependencies.push(dep.to_string());
                            }
                        }
                    } else if line.trim().starts_with("const ") && line.contains("require(") {
                        if let Some(start) = line.find("require(") {
                            if let Some(end) = line[start..].find(')') {
                                let dep = &line[start + 8..start + end].trim().trim_matches('"').trim_matches('\'');
                                if !dep.starts_with('.') {
                                    dependencies.push(dep.to_string());
                                }
                            }
                        }
                    }
                }
            }
            "python" => {
                for line in content.lines() {
                    if line.trim().starts_with("import ") {
                        if let Some(dep) = line.trim().strip_prefix("import ").and_then(|s| s.split_whitespace().next()) {
                            dependencies.push(dep.to_string());
                        }
                    } else if line.trim().starts_with("from ") {
                        if let Some(dep) = line.trim().strip_prefix("from ").and_then(|s| s.split_whitespace().next()) {
                            dependencies.push(dep.to_string());
                        }
                    }
                }
            }
            _ => {}
        }

        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    fn calculate_complexity(&self, node: &tree_sitter::Node, language: &str) -> f64 {
        let mut complexity = 1.0; // Base complexity
        let mut cursor = node.walk();

        fn complexity_recursive(cursor: &mut tree_sitter::TreeCursor, language: &str) -> f64 {
            let mut complexity = 0.0;
            
            // Add complexity for control flow structures
            match cursor.node().kind() {
                "if_expression" | "if_let_expression" | "match_expression" => complexity += 1.0,
                "while_expression" | "for_expression" | "loop_expression" => complexity += 2.0,
                "if_statement" | "while_statement" | "for_statement" => complexity += 1.0,
                "switch_statement" | "try_statement" => complexity += 1.0,
                _ => {}
            }

            if cursor.goto_first_child() {
                complexity += complexity_recursive(cursor, language);
                while cursor.goto_next_sibling() {
                    complexity += complexity_recursive(cursor, language);
                }
                cursor.goto_parent();
            }

            complexity
        }

        complexity + complexity_recursive(&mut cursor, language)
    }

    fn find_issues(&self, content: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Check for long lines
        for (line_num, line) in content.lines().enumerate() {
            if line.len() > 100 {
                issues.push(CodeIssue {
                    severity: "warning".to_string(),
                    message: format!("Line too long ({} characters)", line.len()),
                    line: Some(line_num + 1),
                    column: Some(100),
                });
            }
        }

        // Language-specific checks
        match language {
            "rust" => {
                // Check for unwrap() usage
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains(".unwrap()") {
                        issues.push(CodeIssue {
                            severity: "warning".to_string(),
                            message: "Consider using proper error handling instead of unwrap()".to_string(),
                            line: Some(line_num + 1),
                            column: line.find(".unwrap()"),
                        });
                    }
                }
            }
            "javascript" | "typescript" => {
                // Check for console.log
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains("console.log") {
                        issues.push(CodeIssue {
                            severity: "info".to_string(),
                            message: "Consider removing console.log in production code".to_string(),
                            line: Some(line_num + 1),
                            column: line.find("console.log"),
                        });
                    }
                }
            }
            "python" => {
                // Check for print statements
                for (line_num, line) in content.lines().enumerate() {
                    if line.trim().starts_with("print(") {
                        issues.push(CodeIssue {
                            severity: "info".to_string(),
                            message: "Consider using logging instead of print".to_string(),
                            line: Some(line_num + 1),
                            column: line.find("print("),
                        });
                    }
                }
            }
            _ => {}
        }

        issues
    }

    fn calculate_metrics(&self, content: &str, node: &tree_sitter::Node, language: &str) -> CodeMetrics {
        let lines = content.lines().collect::<Vec<_>>();
        let total_lines = lines.len();
        
        let comment_lines = lines.iter()
            .filter(|line| {
                let trimmed = line.trim();
                match language {
                    "rust" => trimmed.starts_with("//") || trimmed.starts_with("///"),
                    "javascript" | "typescript" => trimmed.starts_with("//"),
                    "python" => trimmed.starts_with("#"),
                    _ => false,
                }
            })
            .count();

        let comment_ratio = if total_lines > 0 {
            comment_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        let cyclomatic_complexity = self.calculate_complexity(node, language) as usize;
        
        // Simplified maintainability index calculation
        let lines_of_code = total_lines - comment_lines;
        let maintainability_index = 171.0 
            - 5.2 * (lines_of_code as f64).ln() 
            - 0.23 * cyclomatic_complexity as f64 
            + 16.2 * (lines_of_code as f64).ln();

        CodeMetrics {
            cyclomatic_complexity,
            lines_of_code,
            comment_ratio,
            maintainability_index: maintainability_index.max(0.0).min(100.0),
        }
    }
}