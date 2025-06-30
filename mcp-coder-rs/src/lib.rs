use anyhow::{anyhow, Result};
use async_mcp::{
    Content, PromptMessage, Resource, Server, Tool, ToolCall, ToolResult, ClientCapabilities, McpError
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use regex::Regex;

pub mod code_analyzer;
pub mod formatter;
pub mod file_manager;

use code_analyzer::CodeAnalyzer;
use formatter::CodeFormatter;
use file_manager::FileManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysisRequest {
    pub file_path: String,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeFormatRequest {
    pub code: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchRequest {
    pub directory: String,
    pub pattern: Option<String>,
    pub file_types: Option<Vec<String>>,
}

pub struct McpCoderServer {
    analyzer: CodeAnalyzer,
    formatter: CodeFormatter,
    file_manager: FileManager,
    base_directory: PathBuf,
}

impl McpCoderServer {
    pub fn new(base_directory: Option<PathBuf>) -> Result<Self> {
        let base_dir = base_directory.unwrap_or_else(|| std::env::current_dir().unwrap());
        
        Ok(Self {
            analyzer: CodeAnalyzer::new()?,
            formatter: CodeFormatter::new(),
            file_manager: FileManager::new(base_dir.clone()),
            base_directory: base_dir,
        })
    }

    pub async fn serve(&self) -> Result<()> {
        let server = Server::new();
        
        // Register tools
        server.add_tool(Tool::new(
            "analyze_code",
            "Analyze code file for complexity, dependencies, and structure",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the code file to analyze"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (rust, javascript, python, etc.)",
                        "enum": ["rust", "javascript", "python", "typescript"]
                    }
                },
                "required": ["file_path"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "format_code",
            "Format code according to language standards",
            json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to format"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language",
                        "enum": ["rust", "javascript", "python", "typescript"]
                    }
                },
                "required": ["code", "language"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "search_files",
            "Search for files in the codebase with optional patterns",
            json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Directory to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex)"
                    },
                    "file_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File extensions to include (e.g., ['rs', 'js', 'py'])"
                    }
                },
                "required": ["directory"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "get_project_structure",
            "Get the directory structure of a project",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Root path to analyze"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth to traverse",
                        "default": 3
                    }
                },
                "required": ["path"]
            }),
        )).await?;

        // Register resources
        server.add_resource(Resource::new(
            "codebase://files/{path}",
            "File content from the codebase",
            Some("text/plain".to_string()),
        )).await?;

        server.add_resource(Resource::new(
            "codebase://structure/{path}",
            "Directory structure of the codebase",
            Some("application/json".to_string()),
        )).await?;

        // Set tool handlers
        server.set_tool_handler(|call: ToolCall| async move {
            self.handle_tool_call(call).await
        }).await?;

        // Set resource handler
        server.set_resource_handler(|uri: &str| async move {
            self.handle_resource_request(uri).await
        }).await?;

        server.start().await?;
        Ok(())
    }

    async fn handle_tool_call(&self, call: ToolCall) -> Result<ToolResult> {
        match call.name.as_str() {
            "analyze_code" => {
                let req: CodeAnalysisRequest = serde_json::from_value(call.arguments)?;
                let result = self.analyzer.analyze_file(&req.file_path, req.language.as_deref()).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: false,
                })
            }
            "format_code" => {
                let req: CodeFormatRequest = serde_json::from_value(call.arguments)?;
                let formatted = self.formatter.format(&req.code, &req.language).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: formatted,
                    }],
                    is_error: false,
                })
            }
            "search_files" => {
                let req: FileSearchRequest = serde_json::from_value(call.arguments)?;
                let results = self.file_manager.search_files(
                    &req.directory,
                    req.pattern.as_deref(),
                    req.file_types.as_deref(),
                ).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&results)?,
                    }],
                    is_error: false,
                })
            }
            "get_project_structure" => {
                let path = call.arguments.get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let max_depth = call.arguments.get("max_depth")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3) as usize;
                
                let structure = self.file_manager.get_project_structure(path, max_depth).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&structure)?,
                    }],
                    is_error: false,
                })
            }
            _ => Err(anyhow!("Unknown tool: {}", call.name)),
        }
    }

    async fn handle_resource_request(&self, uri: &str) -> Result<Resource> {
        if uri.starts_with("codebase://files/") {
            let path = &uri[17..]; // Remove "codebase://files/"
            let content = fs::read_to_string(path).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(path.to_string()),
                description: Some(format!("Content of file: {}", path)),
                mime_type: Some("text/plain".to_string()),
                text: Some(content),
                blob: None,
            })
        } else if uri.starts_with("codebase://structure/") {
            let path = &uri[21..]; // Remove "codebase://structure/"
            let structure = self.file_manager.get_project_structure(path, 3).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("Structure of {}", path)),
                description: Some(format!("Directory structure of: {}", path)),
                mime_type: Some("application/json".to_string()),
                text: Some(serde_json::to_string_pretty(&structure)?),
                blob: None,
            })
        } else {
            Err(anyhow!("Unknown resource URI: {}", uri))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpCoderServer::new(Some(temp_dir.path().to_path_buf()));
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_rust_code() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        
        write(&rust_file, r#"
fn main() {
    println!("Hello, world!");
}

struct TestStruct {
    field: i32,
}

impl TestStruct {
    fn new(field: i32) -> Self {
        Self { field }
    }
}
"#).await.unwrap();

        let server = McpCoderServer::new(Some(temp_dir.path().to_path_buf())).unwrap();
        let result = server.analyzer.analyze_file(rust_file.to_str().unwrap(), Some("rust")).await;
        assert!(result.is_ok());
    }
}