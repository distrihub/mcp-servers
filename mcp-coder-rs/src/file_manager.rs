use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub path: String,
    pub size: u64,
    pub modified: Option<String>,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub children: Option<Vec<ProjectStructure>>,
    pub file_type: Option<String>,
}

pub struct FileManager {
    base_directory: PathBuf,
}

impl FileManager {
    pub fn new(base_directory: PathBuf) -> Self {
        Self { base_directory }
    }

    pub async fn search_files(
        &self,
        directory: &str,
        pattern: Option<&str>,
        file_types: Option<&[String]>,
    ) -> Result<Vec<FileSearchResult>> {
        let search_path = if directory.starts_with('/') {
            PathBuf::from(directory)
        } else {
            self.base_directory.join(directory)
        };

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

            if !path.is_file() {
                continue;
            }

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
            let modified = metadata.modified()
                .ok()
                .and_then(|time| {
                    use std::time::UNIX_EPOCH;
                    time.duration_since(UNIX_EPOCH)
                        .ok()
                        .map(|d| {
                            use chrono::{DateTime, Utc};
                            let datetime = DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0);
                            datetime.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        })
                        .flatten()
                });

            results.push(FileSearchResult {
                path: path.to_string_lossy().to_string(),
                size: metadata.len(),
                modified,
                file_type: self.get_file_type(path),
            });
        }

        results.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(results)
    }

    pub async fn get_project_structure(&self, path: &str, max_depth: usize) -> Result<ProjectStructure> {
        let target_path = if path.starts_with('/') {
            PathBuf::from(path)
        } else {
            self.base_directory.join(path)
        };

        if !target_path.exists() {
            return Err(anyhow!("Path does not exist: {}", path));
        }

        self.build_structure(&target_path, 0, max_depth).await
    }

    async fn build_structure(&self, path: &Path, current_depth: usize, max_depth: usize) -> Result<ProjectStructure> {
        let metadata = fs::metadata(path).await?;
        let name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();

        if metadata.is_dir() {
            let mut children = Vec::new();

            if current_depth < max_depth {
                let mut entries = fs::read_dir(path).await?;
                let mut dir_entries = Vec::new();

                while let Some(entry) = entries.next_entry().await? {
                    dir_entries.push(entry);
                }

                // Sort entries: directories first, then files
                dir_entries.sort_by(|a, b| {
                    let a_is_dir = a.path().is_dir();
                    let b_is_dir = b.path().is_dir();
                    
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                for entry in dir_entries {
                    // Skip hidden files and common ignored directories
                    let entry_name = entry.file_name();
                    let entry_name_str = entry_name.to_string_lossy();
                    
                    if entry_name_str.starts_with('.') || 
                       entry_name_str == "node_modules" ||
                       entry_name_str == "target" ||
                       entry_name_str == "__pycache__" ||
                       entry_name_str == "dist" ||
                       entry_name_str == "build" {
                        continue;
                    }

                    match self.build_structure(&entry.path(), current_depth + 1, max_depth).await {
                        Ok(child) => children.push(child),
                        Err(_) => continue, // Skip entries we can't read
                    }
                }
            }

            Ok(ProjectStructure {
                name,
                path: path.to_string_lossy().to_string(),
                is_directory: true,
                size: None,
                children: if children.is_empty() { None } else { Some(children) },
                file_type: None,
            })
        } else {
            Ok(ProjectStructure {
                name,
                path: path.to_string_lossy().to_string(),
                is_directory: false,
                size: Some(metadata.len()),
                children: None,
                file_type: Some(self.get_file_type(path)),
            })
        }
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
            Some("dockerfile") => "dockerfile".to_string(),
            Some(ext) => ext.to_string(),
            None => "unknown".to_string(),
        }
    }

    pub async fn read_file_content(&self, file_path: &str) -> Result<String> {
        let path = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            self.base_directory.join(file_path)
        };

        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        if !path.is_file() {
            return Err(anyhow!("Path is not a file: {}", file_path));
        }

        let content = fs::read_to_string(&path).await?;
        Ok(content)
    }

    pub async fn get_file_info(&self, file_path: &str) -> Result<FileSearchResult> {
        let path = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            self.base_directory.join(file_path)
        };

        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let metadata = fs::metadata(&path).await?;
        let modified = metadata.modified()
            .ok()
            .and_then(|time| {
                use std::time::UNIX_EPOCH;
                time.duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|d| {
                        use chrono::{DateTime, Utc};
                        let datetime = DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0);
                        datetime.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    })
                    .flatten()
            });

        Ok(FileSearchResult {
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified,
            file_type: self.get_file_type(&path),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::{create_dir, write};

    #[tokio::test]
    async fn test_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new(temp_dir.path().to_path_buf());

        // Create a simple project structure
        let src_dir = temp_dir.path().join("src");
        create_dir(&src_dir).await.unwrap();
        write(src_dir.join("main.rs"), "fn main() {}").await.unwrap();
        write(temp_dir.path().join("Cargo.toml"), "[package]").await.unwrap();

        let structure = manager.get_project_structure(".", 2).await.unwrap();
        assert!(structure.is_directory);
        assert!(structure.children.is_some());
    }

    #[tokio::test]
    async fn test_file_search() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new(temp_dir.path().to_path_buf());

        // Create test files
        write(temp_dir.path().join("test.rs"), "fn test() {}").await.unwrap();
        write(temp_dir.path().join("test.js"), "function test() {}").await.unwrap();

        let results = manager.search_files(".", None, Some(&["rs".to_string()])).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("test.rs"));
    }

    #[tokio::test]
    async fn test_read_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new(temp_dir.path().to_path_buf());

        let content = "Hello, world!";
        let file_path = temp_dir.path().join("test.txt");
        write(&file_path, content).await.unwrap();

        let read_content = manager.read_file_content("test.txt").await.unwrap();
        assert_eq!(read_content, content);
    }
}