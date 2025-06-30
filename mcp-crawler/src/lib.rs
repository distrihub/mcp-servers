use anyhow::Result;
use rpc_router::{Router, Request, Error, CallResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, error};
use std::collections::HashMap;
use url::Url;

mod mcp;
use mcp::{types::*, utilities::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlSiteRequest {
    pub url: String,
    pub max_depth: Option<u32>,
    pub max_pages: Option<u32>,
    pub respect_robots: Option<bool>,
    pub follow_external: Option<bool>,
    pub delay_ms: Option<u64>,
    pub user_agent: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageContentRequest {
    pub url: String,
    pub extract_links: Option<bool>,
    pub extract_text: Option<bool>,
    pub extract_metadata: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RobotsRequest {
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlResult {
    pub urls: Vec<String>,
    pub pages_crawled: u32,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: Option<String>,
    pub text_content: Option<String>,
    pub links: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub status_code: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RobotsInfo {
    pub domain: String,
    pub allowed: bool,
    pub crawl_delay: Option<u32>,
    pub sitemap_urls: Vec<String>,
}

pub struct McpCrawlerServer;

impl McpCrawlerServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn serve(&self) -> Result<()> {
        let mut router = Router::new();

        // Standard MCP methods
        router.insert("initialize", initialize);
        router.insert("ping", ping);
        router.insert("logging/setLevel", logging_set_level);
        router.insert("roots/list", roots_list);

        // Tools
        router.insert("tools/list", list_tools);
        router.insert("crawl_site", crawl_site);
        router.insert("get_page_content", get_page_content);
        router.insert("check_robots", check_robots);

        // Resources
        router.insert("resources/list", list_resources);
        router.insert("resources/read", read_resource);

        serve_stdio(router).await
    }
}

async fn list_tools(_: Option<Value>) -> Result<Value, Error> {
    Ok(json!({
        "tools": [
            {
                "name": "crawl_site",
                "description": "Crawl a website and discover URLs with configurable depth and filtering",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to start crawling from"
                        },
                        "max_depth": {
                            "type": "integer",
                            "description": "Maximum crawl depth (default: 2)",
                            "default": 2
                        },
                        "max_pages": {
                            "type": "integer",
                            "description": "Maximum number of pages to crawl (default: 50)",
                            "default": 50
                        },
                        "respect_robots": {
                            "type": "boolean",
                            "description": "Whether to respect robots.txt (default: true)",
                            "default": true
                        },
                        "follow_external": {
                            "type": "boolean",
                            "description": "Whether to follow external links (default: false)",
                            "default": false
                        },
                        "delay_ms": {
                            "type": "integer",
                            "description": "Delay between requests in milliseconds (default: 1000)",
                            "default": 1000
                        }
                    },
                    "required": ["url"]
                }
            },
            {
                "name": "get_page_content",
                "description": "Fetch and parse content from a specific web page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch content from"
                        },
                        "extract_links": {
                            "type": "boolean",
                            "description": "Whether to extract links (default: true)",
                            "default": true
                        },
                        "extract_text": {
                            "type": "boolean",
                            "description": "Whether to extract text content (default: true)",
                            "default": true
                        },
                        "extract_metadata": {
                            "type": "boolean",
                            "description": "Whether to extract metadata (default: true)",
                            "default": true
                        }
                    },
                    "required": ["url"]
                }
            },
            {
                "name": "check_robots",
                "description": "Check robots.txt for a domain and return crawling permissions",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "The domain to check robots.txt for"
                        }
                    },
                    "required": ["domain"]
                }
            }
        ]
    }))
}

async fn crawl_site(request: Request) -> Result<CallResponse, Error> {
    let params: CrawlSiteRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Crawling site: {}", params.url);

    // Mock implementation - replace with actual crawler logic
    let result = CrawlResult {
        urls: vec![params.url.clone()],
        pages_crawled: 1,
        duration_ms: 1000,
        errors: vec![],
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn get_page_content(request: Request) -> Result<CallResponse, Error> {
    let params: PageContentRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Fetching page content: {}", params.url);

    // Mock implementation - replace with actual page fetching logic
    let result = PageContent {
        url: params.url.clone(),
        title: Some("Example Page".to_string()),
        text_content: Some("This is example page content.".to_string()),
        links: Some(vec!["https://example.com/link1".to_string()]),
        metadata: Some(HashMap::new()),
        status_code: 200,
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn check_robots(request: Request) -> Result<CallResponse, Error> {
    let params: RobotsRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Checking robots.txt for: {}", params.domain);

    // Mock implementation - replace with actual robots.txt checking logic
    let result = RobotsInfo {
        domain: params.domain.clone(),
        allowed: true,
        crawl_delay: Some(1),
        sitemap_urls: vec![format!("{}/sitemap.xml", params.domain)],
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn list_resources(_: Option<Value>) -> Result<Value, Error> {
    Ok(json!({
        "resources": [
            {
                "uri": "crawler://site/{url}",
                "name": "Crawled website data",
                "description": "Access crawled data for a specific website",
                "mimeType": "application/json"
            }
        ]
    }))
}

async fn read_resource(request: Request) -> Result<CallResponse, Error> {
    // Mock implementation - replace with actual resource reading logic
    Ok(CallResponse::from_value(json!({
        "contents": [{
            "uri": "crawler://site/example.com",
            "mimeType": "application/json",
            "text": "{\"status\": \"crawled\", \"pages\": 1}"
        }]
    })))
}