use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_mcp::server::{Server, ServerBuilder};
use async_mcp::transport::Transport;
use async_mcp::types::{
    CallToolRequest, CallToolResponse, ListRequest, ResourcesListResponse, ServerCapabilities,
    Tool, ToolResponseContent,
};
use serde_json::json;
use tracing::info;

pub fn build<T: Transport>(transport: T) -> Result<Server<T>> {
    let mut server = Server::builder(transport)
        .capabilities(ServerCapabilities {
            tools: Some(json!({})),
            ..Default::default()
        })
        .request_handler("resources/list", |_req: ListRequest| {
            Box::pin(async move {
                Ok(ResourcesListResponse {
                    resources: vec![],
                    next_cursor: None,
                    meta: None,
                })
            })
        });

    register_tools(&mut server)?;
    
    let server = server.build();
    info!("MCP Filesystem server initialized");
    Ok(server)
}

fn register_tools<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    // Read File Tool
    let read_file_tool = Tool {
        name: "read_file".to_string(),
        description: Some("Read the complete contents of a file from the file system. \
            Handles various text encodings and provides detailed error messages \
            if the file cannot be read. Use this tool when you need to examine \
            the contents of a single file.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "content": {"type": "string"}
            }
        })),
    };

    server.register_tool(read_file_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                info!("Reading file: {:?}", path);
                let content = std::fs::read_to_string(path)?;
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: content }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error reading file: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error reading file: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Write File Tool
    let write_file_tool = Tool {
        name: "write_file".to_string(),
        description: Some("Write content to a file, creating the file if it doesn't exist \
            and creating parent directories as needed. This will overwrite existing files.".to_string()),
        input_schema: json!({
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
            "required": ["path", "content"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(write_file_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                let content = args["content"]
                    .as_str()
                    .ok_or(anyhow::anyhow!("Missing content parameter"))?;
                info!("Writing file: {:?}", path);
                
                // Create parent directories if they don't exist
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                std::fs::write(path, content)?;
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: "File written successfully".to_string() 
                    }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error writing file: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error writing file: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // List Directory Tool
    let list_directory_tool = Tool {
        name: "list_directory".to_string(),
        description: Some("Get a detailed listing of all files and directories in a specified path. \
            Results clearly distinguish between files and directories with [FILE] and [DIR] \
            prefixes. This tool is essential for understanding directory structure and \
            finding specific files within a directory.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "entries": {"type": "string"}
            }
        })),
    };

    server.register_tool(list_directory_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                info!("Listing directory: {:?}", path);
                let entries = std::fs::read_dir(path)?;
                let mut text = String::new();
                for entry in entries {
                    let entry = entry?;
                    let prefix = if entry.file_type()?.is_dir() {
                        "[DIR]"
                    } else {
                        "[FILE]"
                    };
                    text.push_str(&format!(
                        "{} {}\n",
                        prefix,
                        entry.file_name().to_string_lossy()
                    ));
                }
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error listing directory: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error listing directory: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Create Directory Tool
    let create_directory_tool = Tool {
        name: "create_directory".to_string(),
        description: Some("Create a new directory, including any necessary parent directories. \
            If the directory already exists, this operation will succeed without error.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to create"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(create_directory_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                info!("Creating directory: {:?}", path);
                std::fs::create_dir_all(path)?;
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: "Directory created successfully".to_string() 
                    }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error creating directory: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error creating directory: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Delete File Tool
    let delete_file_tool = Tool {
        name: "delete_file".to_string(),
        description: Some("Delete a file or directory. For directories, this will recursively \
            delete all contents. Use with caution as this operation cannot be undone.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory to delete"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(delete_file_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                info!("Deleting file: {:?}", path);
                if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                } else {
                    std::fs::remove_file(path)?;
                }
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: "File/directory deleted successfully".to_string() 
                    }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error deleting file: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error deleting file: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Move File Tool
    let move_file_tool = Tool {
        name: "move_file".to_string(),
        description: Some("Move or rename a file or directory from one location to another. \
            This can be used for both moving files between directories and renaming files \
            in the same directory.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "from": {
                    "type": "string",
                    "description": "Source path of the file or directory to move"
                },
                "to": {
                    "type": "string", 
                    "description": "Destination path for the file or directory"
                }
            },
            "required": ["from", "to"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(move_file_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let from_path = get_path_from_key(&args, "from")?;
                let to_path = get_path_from_key(&args, "to")?;
                info!("Moving file from {:?} to {:?}", from_path, to_path);
                
                // Create parent directories if they don't exist
                if let Some(parent) = to_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                std::fs::rename(from_path, to_path)?;
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: "File/directory moved successfully".to_string() 
                    }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error moving file: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error moving file: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Search Files Tool
    let search_files_tool = Tool {
        name: "search_files".to_string(),
        description: Some("Recursively search for files and directories matching a pattern. \
            Searches through all subdirectories from the starting path. The search \
            is case-insensitive and matches partial names. Returns full paths to all \
            matching items. Great for finding files when you don't know their exact location.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Starting directory path for the search"
                },
                "pattern": {
                    "type": "string",
                    "description": "Search pattern to match against file and directory names"
                }
            },
            "required": ["path", "pattern"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "matches": {"type": "array", "items": {"type": "string"}}
            }
        })),
    };

    server.register_tool(search_files_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                let pattern = args["pattern"]
                    .as_str()
                    .ok_or(anyhow::anyhow!("Missing pattern parameter"))?;
                info!("Searching files in {:?} with pattern: {}", path, pattern);
                let mut matches = Vec::new();
                search_directory(&path, pattern, &mut matches)?;
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: matches.join("\n"),
                    }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error searching files: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error searching files: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Get File Info Tool
    let get_file_info_tool = Tool {
        name: "get_file_info".to_string(),
        description: Some("Retrieve detailed metadata about a file or directory. Returns comprehensive \
            information including size, creation time, last modified time, permissions, \
            and type. This tool is perfect for understanding file characteristics \
            without reading the actual content.".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory to get info about"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "info": {"type": "object"}
            }
        })),
    };

    server.register_tool(get_file_info_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let path = get_path(&args)?;
                info!("Getting file info for: {:?}", path);
                let metadata = std::fs::metadata(&path)?;
                let file_type = if metadata.is_file() {
                    "file"
                } else if metadata.is_dir() {
                    "directory"
                } else {
                    "other"
                };
                
                let info = json!({
                    "path": path.to_string_lossy(),
                    "type": file_type,
                    "size": metadata.len(),
                    "modified": metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())),
                    "created": metadata.created().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())),
                    "readonly": metadata.permissions().readonly(),
                });
                
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&info)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }.await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error getting file info: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error getting file info: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    Ok(())
}

fn search_directory(dir: &Path, pattern: &str, matches: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        // Check if the current file/directory matches the pattern
        if name.contains(&pattern.to_lowercase()) {
            matches.push(path.to_string_lossy().to_string());
        }

        // Recursively search subdirectories
        if path.is_dir() {
            search_directory(&path, pattern, matches)?;
        }
    }
    Ok(())
}

fn get_path(args: &HashMap<String, serde_json::Value>) -> Result<PathBuf> {
    let path = args["path"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing path parameter"))?;
    resolve_path(path)
}

fn get_path_from_key(args: &HashMap<String, serde_json::Value>, key: &str) -> Result<PathBuf> {
    let path = args[key]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing {} parameter", key))?;
    resolve_path(path)
}

fn resolve_path(path: &str) -> Result<PathBuf> {
    if path.starts_with('~') {
        let home = home::home_dir().ok_or(anyhow::anyhow!("Could not determine home directory"))?;
        // Strip the ~ and join with home path
        let path = home.join(path.strip_prefix("~/").unwrap_or_default());
        Ok(path)
    } else {
        Ok(PathBuf::from(path))
    }
}