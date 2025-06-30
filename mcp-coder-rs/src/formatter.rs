use anyhow::{anyhow, Result};
use std::process::Command;
use tokio::fs;
use tempfile::NamedTempFile;

pub struct CodeFormatter;

impl CodeFormatter {
    pub fn new() -> Self {
        Self
    }

    pub async fn format(&self, code: &str, language: &str) -> Result<String> {
        match language {
            "rust" => self.format_rust(code).await,
            "javascript" | "typescript" => self.format_javascript(code).await,
            "python" => self.format_python(code).await,
            _ => Err(anyhow!("Unsupported language for formatting: {}", language)),
        }
    }

    async fn format_rust(&self, code: &str) -> Result<String> {
        // Try to use rustfmt if available
        if self.is_command_available("rustfmt") {
            let mut temp_file = NamedTempFile::new()?;
            fs::write(temp_file.path(), code).await?;

            let output = Command::new("rustfmt")
                .arg("--emit=stdout")
                .arg(temp_file.path())
                .output()?;

            if output.status.success() {
                return Ok(String::from_utf8(output.stdout)?);
            }
        }

        // Fallback to basic formatting
        self.basic_rust_format(code)
    }

    async fn format_javascript(&self, code: &str) -> Result<String> {
        // Try to use prettier if available
        if self.is_command_available("prettier") {
            let mut temp_file = NamedTempFile::new()?;
            fs::write(temp_file.path(), code).await?;

            let output = Command::new("prettier")
                .arg("--parser")
                .arg("babel")
                .arg(temp_file.path())
                .output()?;

            if output.status.success() {
                return Ok(String::from_utf8(output.stdout)?);
            }
        }

        // Fallback to basic formatting
        self.basic_js_format(code)
    }

    async fn format_python(&self, code: &str) -> Result<String> {
        // Try to use black if available
        if self.is_command_available("black") {
            let mut temp_file = NamedTempFile::new()?;
            fs::write(temp_file.path(), code).await?;

            let output = Command::new("black")
                .arg("--code")
                .arg(code)
                .output()?;

            if output.status.success() {
                return Ok(String::from_utf8(output.stdout)?);
            }
        }

        // Try autopep8 as fallback
        if self.is_command_available("autopep8") {
            let output = Command::new("autopep8")
                .arg("--")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?
                .stdin
                .take()
                .unwrap()
                .write_all(code.as_bytes());

            // Note: This is a simplified version - in real implementation
            // you'd properly handle the stdin/stdout communication
        }

        // Fallback to basic formatting
        self.basic_python_format(code)
    }

    fn is_command_available(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn basic_rust_format(&self, code: &str) -> Result<String> {
        let mut formatted = String::new();
        let mut indent_level = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for line in code.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            // Simple indentation logic
            if trimmed.ends_with('{') && !in_string {
                formatted.push_str(&"    ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
                indent_level += 1;
            } else if trimmed.starts_with('}') && !in_string {
                if indent_level > 0 {
                    indent_level -= 1;
                }
                formatted.push_str(&"    ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
            } else {
                formatted.push_str(&"    ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
            }

            // Track string state for better formatting
            for ch in trimmed.chars() {
                if escape_next {
                    escape_next = false;
                    continue;
                }
                match ch {
                    '\\' => escape_next = true,
                    '"' => in_string = !in_string,
                    _ => {}
                }
            }
        }

        Ok(formatted)
    }

    fn basic_js_format(&self, code: &str) -> Result<String> {
        let mut formatted = String::new();
        let mut indent_level = 0;

        for line in code.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            if trimmed.ends_with('{') {
                formatted.push_str(&"  ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
                indent_level += 1;
            } else if trimmed.starts_with('}') {
                if indent_level > 0 {
                    indent_level -= 1;
                }
                formatted.push_str(&"  ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
            } else {
                formatted.push_str(&"  ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
            }
        }

        Ok(formatted)
    }

    fn basic_python_format(&self, code: &str) -> Result<String> {
        let mut formatted = String::new();
        let mut indent_level = 0;

        for line in code.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            // Handle Python indentation
            if trimmed.ends_with(':') {
                formatted.push_str(&"    ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
                indent_level += 1;
            } else if trimmed.starts_with("else:") || trimmed.starts_with("elif ") || 
                      trimmed.starts_with("except") || trimmed.starts_with("finally:") {
                if indent_level > 0 {
                    formatted.push_str(&"    ".repeat(indent_level - 1));
                }
                formatted.push_str(trimmed);
                formatted.push('\n');
            } else {
                // Check if we need to dedent
                let current_indent = line.len() - line.trim_start().len();
                if current_indent < indent_level * 4 {
                    indent_level = current_indent / 4;
                }
                
                formatted.push_str(&"    ".repeat(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
            }
        }

        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_rust_format() {
        let formatter = CodeFormatter::new();
        let code = r#"fn main(){println!("Hello");}"#;
        let result = formatter.basic_rust_format(code).unwrap();
        assert!(result.contains("fn main() {"));
    }

    #[tokio::test]
    async fn test_basic_js_format() {
        let formatter = CodeFormatter::new();
        let code = r#"function test(){console.log("hello");}"#;
        let result = formatter.basic_js_format(code).unwrap();
        assert!(result.contains("function test() {"));
    }

    #[tokio::test]
    async fn test_basic_python_format() {
        let formatter = CodeFormatter::new();
        let code = r#"def test():print("hello")"#;
        let result = formatter.basic_python_format(code).unwrap();
        assert!(result.contains("def test():"));
    }
}