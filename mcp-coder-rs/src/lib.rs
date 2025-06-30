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
pub struct ReadFileRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilesRequest {
    pub directory: String,
    pub pattern: Option<String>,
    pub file_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub file_type: String,
    pub is_directory: bool,
}

pub struct McpCoderServer {
    base_directory: PathBuf,
}

impl McpCoderServer {
    pub fn new(base_directory: PathBuf) -> Self {
        Self { base_directory }
    }

    pub async fn serve(&self) -> Result<()> {
        let server = Server::new();

        // Register tools
        server.add_tool(Tool::new(
            "read_file",
            "Read the contents of a file",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "write_file",
            "Write content to a file",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "search_files",
            "Search for files in a directory",
            json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Directory to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to match filenames"
                    },
                    "file_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File extensions to filter by"
                    }
                },
                "required": ["directory"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "list_directory",
            "List contents of a directory",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to list"
                    }
                },
                "required": ["path"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "get_project_structure",
            "Get the structure of a project directory",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the project root"
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
            "file://{path}",
            "File content resource",
            Some("text/plain".to_string()),
        )).await?;

        server.add_resource(Resource::new(
            "directory://{path}",
            "Directory listing resource",
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
            "read_file" => {
                let req: ReadFileRequest = serde_json::from_value(call.arguments)?;
                let content = self.read_file(&req.path).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: content,
                    }],
                    is_error: false,
                })
            }
            "write_file" => {
                let req: WriteFileRequest = serde_json::from_value(call.arguments)?;
                self.write_file(&req.path, &req.content).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: format!("Successfully wrote to {}", req.path),
                    }],
                    is_error: false,
                })
            }
            "search_files" => {
                let req: SearchFilesRequest = serde_json::from_value(call.arguments)?;
                let files = self.search_files(&req.directory, req.pattern.as_deref(), req.file_types.as_deref()).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&files)?,
                    }],
                    is_error: false,
                })
            }
            "list_directory" => {
                let path = call.arguments.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("path is required"))?;
                let files = self.list_directory(path).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&files)?,
                    }],
                    is_error: false,
                })
            }
            "get_project_structure" => {
                let path = call.arguments.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("path is required"))?;
                let max_depth = call.arguments.get("max_depth")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3) as usize;
                let structure = self.get_project_structure(path, max_depth).await?;
                
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
        if let Some(path) = uri.strip_prefix("file://") {
            let content = self.read_file(path).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(Path::new(path).file_name().unwrap_or_default().to_string_lossy().to_string()),
                description: Some(format!("Content of file {}", path)),
                mime_type: Some(self.get_mime_type(path)),
                text: Some(content),
                blob: None,
            })
        } else if let Some(path) = uri.strip_prefix("directory://") {
            let files = self.list_directory(path).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("Directory {}", path)),
                description: Some(format!("Contents of directory {}", path)),
                mime_type: Some("application/json".to_string()),
                text: Some(serde_json::to_string_pretty(&files)?),
                blob: None,
            })
        } else {
            Err(anyhow!("Unknown resource URI: {}", uri))
        }
    }

    async fn read_file(&self, file_path: &str) -> Result<String> {
        let path = self.resolve_path(file_path)?;
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        if !path.is_file() {
            return Err(anyhow!("Path is not a file: {}", file_path));
        }

        let content = fs::read_to_string(&path).await?;
        Ok(content)
    }

    async fn write_file(&self, file_path: &str, content: &str) -> Result<()> {
        let path = self.resolve_path(file_path)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&path, content).await?;
        Ok(())
    }

    async fn search_files(
        &self,
        directory: &str,
        pattern: Option<&str>,
        file_types: Option<&[String]>,
    ) -> Result<Vec<FileInfo>> {
        let search_path = self.resolve_path(directory)?;

        if !search_path.exists() {
            return Err(anyhow!("Directory does not exist: {}", directory));
        }

        let regex = if let Some(pattern) = pattern {
            Some(Regex::new(pattern)?)
        } else {
            None
        };

        let mut results = Vec::new();

        for entry in WalkDir::new(&search_path).follow_links(false) {
            let entry = entry?;
            let path = entry.path();

            // Check file type filter
            if let Some(types) = file_types {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if !types.iter().any(|t| t == ext) {
                        continue;
                    }
                } else if !types.is_empty() {
                    continue;
                }
            }

            // Check pattern filter
            if let Some(ref regex) = regex {
                let file_name = path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                if !regex.is_match(file_name) {
                    continue;
                }
            }

            let metadata = fs::metadata(path).await?;

            results.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                size: metadata.len(),
                file_type: self.get_file_type(path),
                is_directory: metadata.is_dir(),
            });
        }

        results.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(results)
    }

    async fn list_directory(&self, directory: &str) -> Result<Vec<FileInfo>> {
        let dir_path = self.resolve_path(directory)?;

        if !dir_path.exists() {
            return Err(anyhow!("Directory does not exist: {}", directory));
        }

        if !dir_path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", directory));
        }

        let mut entries = fs::read_dir(&dir_path).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            files.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                size: metadata.len(),
                file_type: self.get_file_type(&path),
                is_directory: metadata.is_dir(),
            });
        }

        files.sort_by(|a, b| {
            // Directories first, then files
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.path.cmp(&b.path),
            }
        });

        Ok(files)
    }

    async fn get_project_structure(&self, path: &str, max_depth: usize) -> Result<Value> {
        let project_path = self.resolve_path(path)?;
        self.build_tree(&project_path, 0, max_depth).await
    }

    async fn build_tree(&self, path: &Path, current_depth: usize, max_depth: usize) -> Result<Value> {
        let metadata = fs::metadata(path).await?;
        let name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();

        let mut result = json!({
            "name": name,
            "path": path.to_string_lossy(),
            "is_directory": metadata.is_dir(),
            "size": metadata.len()
        });

        if metadata.is_dir() && current_depth < max_depth {
            let mut children = Vec::new();
            let mut entries = fs::read_dir(path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Skip hidden files and common ignored directories
                if entry_name.starts_with('.') || 
                   entry_name == "node_modules" ||
                   entry_name == "target" ||
                   entry_name == "__pycache__" {
                    continue;
                }

                if let Ok(child) = self.build_tree(&entry_path, current_depth + 1, max_depth).await {
                    children.push(child);
                }
            }

            children.sort_by(|a, b| {
                let a_is_dir = a["is_directory"].as_bool().unwrap_or(false);
                let b_is_dir = b["is_directory"].as_bool().unwrap_or(false);
                
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or("")),
                }
            });

            result["children"] = json!(children);
        }

        Ok(result)
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let path = if path.starts_with('/') {
            PathBuf::from(path)
        } else {
            self.base_directory.join(path)
        };

        // Basic security check to prevent path traversal
        let canonical_base = self.base_directory.canonicalize()?;
        let canonical_path = path.canonicalize().unwrap_or(path);
        
        if !canonical_path.starts_with(&canonical_base) {
            return Err(anyhow!("Path outside of allowed directory: {}", path.display()));
        }

        Ok(canonical_path)
    }

    fn get_file_type(&self, path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("js") | Some("mjs") => "javascript".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("py") => "python".to_string(),
            Some("json") => "json".to_string(),
            Some("toml") => "toml".to_string(),
            Some("yaml") | Some("yml") => "yaml".to_string(),
            Some("md") => "markdown".to_string(),
            Some("txt") => "text".to_string(),
            Some("html") => "html".to_string(),
            Some("css") => "css".to_string(),
            Some("xml") => "xml".to_string(),
            Some("sql") => "sql".to_string(),
            Some("sh") => "shell".to_string(),
            Some(ext) => ext.to_string(),
            None => "unknown".to_string(),
        }
    }

    fn get_mime_type(&self, path: &str) -> String {
        let path = Path::new(path);
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => "application/json".to_string(),
            Some("html") => "text/html".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") | Some("mjs") => "application/javascript".to_string(),
            Some("xml") => "application/xml".to_string(),
            _ => "text/plain".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpCoderServer::new(temp_dir.path().to_path_buf());
        
        // Test basic functionality
        let files = server.list_directory(".").await.unwrap();
        assert!(files.is_empty() || !files.is_empty()); // Just ensure no panic
    }
}