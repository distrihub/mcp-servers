#![recursion_limit = "256"]

use anyhow::{anyhow, Result};
use async_mcp::{
    server::Server,
    types::{Tool, Resource, ClientCapabilities, ToolResponseContent, Content},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use spider::website::Website;
use spider::configuration::Configuration;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error, debug};
use url::Url;
use hashbrown::HashSet;

pub mod crawler;
pub mod scraper;
pub mod config;
pub mod utils;

use crawler::SpiderCrawler;
use scraper::SpiderScraper;
use config::{CrawlConfig, ScrapeConfig, SpiderConfiguration};

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlRequest {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub user_agent: Option<String>,
    pub depth: Option<u32>,
    pub blacklist: Option<Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    pub respect_robots_txt: Option<bool>,
    pub accept_invalid_certs: Option<bool>,
    pub subdomains: Option<bool>,
    pub tld: Option<bool>,
    pub delay: Option<f64>,
    pub budget_depth: Option<u32>,
    pub budget_request_timeout: Option<f64>,
    pub cache: Option<bool>,
    pub use_cookies: Option<bool>,
    pub stealth_mode: Option<bool>,
    pub chrome_intercept: Option<bool>,
    pub include_sitemap: Option<bool>,
    pub max_redirects: Option<u32>,
    pub max_file_size: Option<u64>,
    pub concurrency: Option<u32>,
    pub full_resources: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeRequest {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub user_agent: Option<String>,
    pub depth: Option<u32>,
    pub blacklist: Option<Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    pub respect_robots_txt: Option<bool>,
    pub accept_invalid_certs: Option<bool>,
    pub subdomains: Option<bool>,
    pub tld: Option<bool>,
    pub delay: Option<f64>,
    pub budget_depth: Option<u32>,
    pub budget_request_timeout: Option<f64>,
    pub cache: Option<bool>,
    pub use_cookies: Option<bool>,
    pub stealth_mode: Option<bool>,
    pub chrome_intercept: Option<bool>,
    pub include_sitemap: Option<bool>,
    pub max_redirects: Option<u32>,
    pub max_file_size: Option<u64>,
    pub concurrency: Option<u32>,
    pub full_resources: Option<bool>,
    pub extract_text: Option<bool>,
    pub extract_links: Option<bool>,
    pub extract_images: Option<bool>,
    pub extract_metadata: Option<bool>,
    pub take_screenshots: Option<bool>,
    pub screenshot_params: Option<ScreenshotParams>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotParams {
    pub full_page: Option<bool>,
    pub quality: Option<u8>,
    pub format: Option<String>, // "png" or "jpeg"
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlResult {
    pub urls: Vec<String>,
    pub pages_crawled: u32,
    pub pages_failed: u32,
    pub duration_ms: u64,
    pub sitemap_urls: Option<Vec<String>>,
    pub error_pages: Option<Vec<ErrorPage>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeResult {
    pub pages: Vec<ScrapedPage>,
    pub pages_crawled: u32,
    pub pages_failed: u32,
    pub duration_ms: u64,
    pub sitemap_urls: Option<Vec<String>>,
    pub error_pages: Option<Vec<ErrorPage>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapedPage {
    pub url: String,
    pub status_code: Option<u16>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub text_content: Option<String>,
    pub links: Option<Vec<LinkInfo>>,
    pub images: Option<Vec<ImageInfo>>,
    pub metadata: Option<PageMetadata>,
    pub headers: Option<HashMap<String, String>>,
    pub screenshot_path: Option<String>,
    pub screenshot_base64: Option<String>,
    pub bytes: Option<usize>,
    pub duration_ms: Option<u64>,
    pub redirect_count: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkInfo {
    pub url: String,
    pub text: Option<String>,
    pub title: Option<String>,
    pub rel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub url: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author: Option<String>,
    pub canonical_url: Option<String>,
    pub language: Option<String>,
    pub charset: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
    pub twitter_card: Option<String>,
    pub twitter_title: Option<String>,
    pub twitter_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPage {
    pub url: String,
    pub error: String,
    pub status_code: Option<u16>,
}

pub struct McpSpiderServer {
    crawler: SpiderCrawler,
    scraper: SpiderScraper,
}

impl McpSpiderServer {
    pub fn new() -> Result<Self> {
        let crawler = SpiderCrawler::new()?;
        let scraper = SpiderScraper::new()?;

        Ok(Self { crawler, scraper })
    }

    pub async fn serve(&self) -> Result<()> {
        let server = Server::new();

        // Register crawl tool
        server.add_tool(Tool::new(
            "crawl",
            "Crawl websites and return discovered URLs with comprehensive spider-rs features",
            json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to start crawling from",
                        "format": "uri"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Additional HTTP headers to include with requests",
                        "additionalProperties": {"type": "string"}
                    },
                    "user_agent": {
                        "type": "string",
                        "description": "Custom User-Agent string to use for requests"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Maximum crawl depth (0 = unlimited)",
                        "minimum": 0,
                        "default": 2
                    },
                    "blacklist": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of regex patterns to exclude URLs"
                    },
                    "whitelist": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of regex patterns to include URLs (overrides blacklist)"
                    },
                    "respect_robots_txt": {
                        "type": "boolean",
                        "description": "Whether to respect robots.txt rules",
                        "default": true
                    },
                    "accept_invalid_certs": {
                        "type": "boolean",
                        "description": "Whether to accept invalid SSL certificates",
                        "default": false
                    },
                    "subdomains": {
                        "type": "boolean",
                        "description": "Whether to crawl subdomains",
                        "default": false
                    },
                    "tld": {
                        "type": "boolean",
                        "description": "Whether to allow all TLDs for the domain",
                        "default": false
                    },
                    "delay": {
                        "type": "number",
                        "description": "Delay between requests in seconds",
                        "minimum": 0,
                        "default": 1.0
                    },
                    "budget_depth": {
                        "type": "integer",
                        "description": "Maximum depth for crawl budget",
                        "minimum": 0
                    },
                    "budget_request_timeout": {
                        "type": "number",
                        "description": "Request timeout in seconds",
                        "minimum": 0,
                        "default": 30.0
                    },
                    "cache": {
                        "type": "boolean",
                        "description": "Whether to cache HTTP responses",
                        "default": false
                    },
                    "use_cookies": {
                        "type": "boolean",
                        "description": "Whether to use cookies",
                        "default": false
                    },
                    "stealth_mode": {
                        "type": "boolean",
                        "description": "Enable stealth mode for Chrome browser",
                        "default": false
                    },
                    "chrome_intercept": {
                        "type": "boolean",
                        "description": "Enable Chrome network request interception",
                        "default": false
                    },
                    "include_sitemap": {
                        "type": "boolean",
                        "description": "Whether to include URLs from sitemap.xml",
                        "default": true
                    },
                    "max_redirects": {
                        "type": "integer",
                        "description": "Maximum number of redirects to follow",
                        "minimum": 0,
                        "default": 5
                    },
                    "max_file_size": {
                        "type": "integer",
                        "description": "Maximum file size to download in bytes",
                        "minimum": 0
                    },
                    "concurrency": {
                        "type": "integer",
                        "description": "Number of concurrent requests",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "full_resources": {
                        "type": "boolean",
                        "description": "Whether to crawl all resources (CSS, JS, images, etc.)",
                        "default": false
                    }
                },
                "required": ["url"]
            }),
        )).await?;

        // Register scrape tool
        server.add_tool(Tool::new(
            "scrape",
            "Scrape websites and extract content, links, images, and metadata with spider-rs",
            json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to start scraping from",
                        "format": "uri"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Additional HTTP headers to include with requests",
                        "additionalProperties": {"type": "string"}
                    },
                    "user_agent": {
                        "type": "string",
                        "description": "Custom User-Agent string to use for requests"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Maximum crawl depth (0 = unlimited)",
                        "minimum": 0,
                        "default": 1
                    },
                    "blacklist": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of regex patterns to exclude URLs"
                    },
                    "whitelist": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of regex patterns to include URLs (overrides blacklist)"
                    },
                    "respect_robots_txt": {
                        "type": "boolean",
                        "description": "Whether to respect robots.txt rules",
                        "default": true
                    },
                    "accept_invalid_certs": {
                        "type": "boolean",
                        "description": "Whether to accept invalid SSL certificates",
                        "default": false
                    },
                    "subdomains": {
                        "type": "boolean",
                        "description": "Whether to crawl subdomains",
                        "default": false
                    },
                    "tld": {
                        "type": "boolean",
                        "description": "Whether to allow all TLDs for the domain",
                        "default": false
                    },
                    "delay": {
                        "type": "number",
                        "description": "Delay between requests in seconds",
                        "minimum": 0,
                        "default": 1.0
                    },
                    "budget_depth": {
                        "type": "integer",
                        "description": "Maximum depth for crawl budget",
                        "minimum": 0
                    },
                    "budget_request_timeout": {
                        "type": "number",
                        "description": "Request timeout in seconds",
                        "minimum": 0,
                        "default": 30.0
                    },
                    "cache": {
                        "type": "boolean",
                        "description": "Whether to cache HTTP responses",
                        "default": false
                    },
                    "use_cookies": {
                        "type": "boolean",
                        "description": "Whether to use cookies",
                        "default": false
                    },
                    "stealth_mode": {
                        "type": "boolean",
                        "description": "Enable stealth mode for Chrome browser",
                        "default": false
                    },
                    "chrome_intercept": {
                        "type": "boolean",
                        "description": "Enable Chrome network request interception",
                        "default": false
                    },
                    "include_sitemap": {
                        "type": "boolean",
                        "description": "Whether to include URLs from sitemap.xml",
                        "default": true
                    },
                    "max_redirects": {
                        "type": "integer",
                        "description": "Maximum number of redirects to follow",
                        "minimum": 0,
                        "default": 5
                    },
                    "max_file_size": {
                        "type": "integer",
                        "description": "Maximum file size to download in bytes",
                        "minimum": 0
                    },
                    "concurrency": {
                        "type": "integer",
                        "description": "Number of concurrent requests",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "full_resources": {
                        "type": "boolean",
                        "description": "Whether to crawl all resources (CSS, JS, images, etc.)",
                        "default": false
                    },
                    "extract_text": {
                        "type": "boolean",
                        "description": "Whether to extract text content from pages",
                        "default": true
                    },
                    "extract_links": {
                        "type": "boolean",
                        "description": "Whether to extract links from pages",
                        "default": true
                    },
                    "extract_images": {
                        "type": "boolean",
                        "description": "Whether to extract image information from pages",
                        "default": true
                    },
                    "extract_metadata": {
                        "type": "boolean",
                        "description": "Whether to extract page metadata (title, description, etc.)",
                        "default": true
                    },
                    "take_screenshots": {
                        "type": "boolean",
                        "description": "Whether to take screenshots of pages (requires Chrome)",
                        "default": false
                    },
                    "screenshot_params": {
                        "type": "object",
                        "description": "Screenshot configuration parameters",
                        "properties": {
                            "full_page": {
                                "type": "boolean",
                                "description": "Take full page screenshot",
                                "default": true
                            },
                            "quality": {
                                "type": "integer",
                                "description": "Screenshot quality (1-100)",
                                "minimum": 1,
                                "maximum": 100,
                                "default": 90
                            },
                            "format": {
                                "type": "string",
                                "description": "Screenshot format",
                                "enum": ["png", "jpeg"],
                                "default": "png"
                            },
                            "viewport_width": {
                                "type": "integer",
                                "description": "Viewport width for screenshot",
                                "minimum": 100,
                                "default": 1920
                            },
                            "viewport_height": {
                                "type": "integer",
                                "description": "Viewport height for screenshot",
                                "minimum": 100,
                                "default": 1080
                            }
                        }
                    }
                },
                "required": ["url"]
            }),
        )).await?;

        // Register resources
        server.add_resource(Resource::new(
            "spider://crawl/{url}",
            "Crawled website URLs and metadata",
            Some("application/json".to_string()),
        )).await?;

        server.add_resource(Resource::new(
            "spider://scrape/{url}",
            "Scraped website content and extracted data",
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

    pub async fn handle_tool_call(&self, call: ToolCall) -> Result<ToolResult> {
        match call.name.as_str() {
            "crawl" => {
                let req: CrawlRequest = serde_json::from_value(call.arguments)?;
                info!("Starting crawl for URL: {}", req.url);
                
                let start_time = std::time::Instant::now();
                let result = self.crawler.crawl(req).await?;
                let duration = start_time.elapsed();
                
                info!("Crawl completed in {:?}: {} URLs found", duration, result.urls.len());
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: false,
                })
            }
            "scrape" => {
                let req: ScrapeRequest = serde_json::from_value(call.arguments)?;
                info!("Starting scrape for URL: {}", req.url);
                
                let start_time = std::time::Instant::now();
                let result = self.scraper.scrape(req).await?;
                let duration = start_time.elapsed();
                
                info!("Scrape completed in {:?}: {} pages scraped", duration, result.pages.len());
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: false,
                })
            }
            _ => Err(anyhow!("Unknown tool: {}", call.name)),
        }
    }

    async fn handle_resource_request(&self, uri: &str) -> Result<Resource> {
        if let Some(url) = uri.strip_prefix("spider://crawl/") {
            // Return cached crawl results if available
            let result = format!("Crawl results for: {}", url);
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("Crawl: {}", url)),
                description: Some("Crawled website URLs and metadata".to_string()),
                mime_type: Some("application/json".to_string()),
                text: Some(result),
                blob: None,
            })
        } else if let Some(url) = uri.strip_prefix("spider://scrape/") {
            // Return cached scrape results if available
            let result = format!("Scrape results for: {}", url);
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("Scrape: {}", url)),
                description: Some("Scraped website content and extracted data".to_string()),
                mime_type: Some("application/json".to_string()),
                text: Some(result),
                blob: None,
            })
        } else {
            Err(anyhow!("Unknown resource URI: {}", uri))
        }
    }
}

impl Default for McpSpiderServer {
    fn default() -> Self {
        Self::new().expect("Failed to create MCP Spider server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = McpSpiderServer::new();
        assert!(server.is_ok());
    }

    #[test]
    fn test_crawl_request_serialization() {
        let request = CrawlRequest {
            url: "https://example.com".to_string(),
            headers: None,
            user_agent: Some("test-agent".to_string()),
            depth: Some(2),
            blacklist: None,
            whitelist: None,
            respect_robots_txt: Some(true),
            accept_invalid_certs: Some(false),
            subdomains: Some(false),
            tld: Some(false),
            delay: Some(1.0),
            budget_depth: None,
            budget_request_timeout: Some(30.0),
            cache: Some(false),
            use_cookies: Some(false),
            stealth_mode: Some(false),
            chrome_intercept: Some(false),
            include_sitemap: Some(true),
            max_redirects: Some(5),
            max_file_size: None,
            concurrency: Some(10),
            full_resources: Some(false),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_scrape_request_serialization() {
        let request = ScrapeRequest {
            url: "https://example.com".to_string(),
            headers: None,
            user_agent: Some("test-agent".to_string()),
            depth: Some(1),
            blacklist: None,
            whitelist: None,
            respect_robots_txt: Some(true),
            accept_invalid_certs: Some(false),
            subdomains: Some(false),
            tld: Some(false),
            delay: Some(1.0),
            budget_depth: None,
            budget_request_timeout: Some(30.0),
            cache: Some(false),
            use_cookies: Some(false),
            stealth_mode: Some(false),
            chrome_intercept: Some(false),
            include_sitemap: Some(true),
            max_redirects: Some(5),
            max_file_size: None,
            concurrency: Some(10),
            full_resources: Some(false),
            extract_text: Some(true),
            extract_links: Some(true),
            extract_images: Some(true),
            extract_metadata: Some(true),
            take_screenshots: Some(false),
            screenshot_params: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://example.com"));
    }
}