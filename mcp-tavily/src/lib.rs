use anyhow::Result;
use rpc_router::{Router, Request, Error, CallResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, error};
use std::collections::HashMap;

mod mcp;
use mcp::{types::*, utilities::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub search_depth: Option<String>, // "basic" or "advanced"
    pub max_results: Option<u32>,
    pub include_domains: Option<Vec<String>>,
    pub exclude_domains: Option<Vec<String>>,
    pub include_answer: Option<bool>,
    pub include_raw_content: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsSearchRequest {
    pub query: String,
    pub days: Option<u32>, // How many days back to search
    pub max_results: Option<u32>,
    pub include_answer: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub answer: Option<String>,
    pub results: Vec<SearchResultItem>,
    pub response_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub content: String,
    pub score: f64,
    pub published_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResult {
    pub url: String,
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
}

pub struct McpTavilyServer {
    api_key: String,
    client: reqwest::Client,
}

impl McpTavilyServer {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { api_key, client }
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
        router.insert("search", search);
        router.insert("search_news", search_news);
        router.insert("get_extract", get_extract);

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
                "name": "search",
                "description": "Perform AI-powered web search using Tavily API",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        },
                        "search_depth": {
                            "type": "string",
                            "enum": ["basic", "advanced"],
                            "description": "Search depth - basic for quick results, advanced for comprehensive search",
                            "default": "basic"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 5)",
                            "default": 5,
                            "minimum": 1,
                            "maximum": 20
                        },
                        "include_domains": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Domains to include in search"
                        },
                        "exclude_domains": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Domains to exclude from search"
                        },
                        "include_answer": {
                            "type": "boolean",
                            "description": "Whether to include an AI-generated answer",
                            "default": true
                        },
                        "include_raw_content": {
                            "type": "boolean",
                            "description": "Whether to include raw content from pages",
                            "default": false
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "search_news",
                "description": "Search for recent news articles using Tavily",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The news search query"
                        },
                        "days": {
                            "type": "integer",
                            "description": "How many days back to search (default: 7)",
                            "default": 7,
                            "minimum": 1,
                            "maximum": 30
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 5)",
                            "default": 5,
                            "minimum": 1,
                            "maximum": 20
                        },
                        "include_answer": {
                            "type": "boolean",
                            "description": "Whether to include an AI-generated summary",
                            "default": true
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_extract",
                "description": "Extract content from a specific URL using Tavily",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to extract content from",
                            "format": "uri"
                        }
                    },
                    "required": ["url"]
                }
            }
        ]
    }))
}

async fn search(request: Request) -> Result<CallResponse, Error> {
    let params: SearchRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Performing Tavily search: {}", params.query);

    // Mock implementation - replace with actual Tavily API call
    let result = SearchResult {
        query: params.query.clone(),
        answer: Some("This is a mock AI-generated answer based on the search results.".to_string()),
        results: vec![
            SearchResultItem {
                title: "Example Result 1".to_string(),
                url: "https://example.com/1".to_string(),
                content: "This is example content from the first search result.".to_string(),
                score: 0.95,
                published_date: Some("2024-01-15".to_string()),
            },
            SearchResultItem {
                title: "Example Result 2".to_string(),
                url: "https://example.com/2".to_string(),
                content: "This is example content from the second search result.".to_string(),
                score: 0.87,
                published_date: Some("2024-01-10".to_string()),
            },
        ],
        response_time: 1.23,
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn search_news(request: Request) -> Result<CallResponse, Error> {
    let params: NewsSearchRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Performing Tavily news search: {}", params.query);

    // Mock implementation - replace with actual Tavily news search API call
    let result = SearchResult {
        query: params.query.clone(),
        answer: Some("This is a mock AI-generated news summary.".to_string()),
        results: vec![
            SearchResultItem {
                title: "Breaking News Example".to_string(),
                url: "https://news.example.com/breaking".to_string(),
                content: "This is example news content from a recent article.".to_string(),
                score: 0.98,
                published_date: Some("2024-01-20".to_string()),
            },
        ],
        response_time: 0.89,
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn get_extract(request: Request) -> Result<CallResponse, Error> {
    let params: ExtractRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Extracting content from URL: {}", params.url);

    // Mock implementation - replace with actual Tavily extract API call
    let result = ExtractResult {
        url: params.url.clone(),
        title: "Example Article Title".to_string(),
        content: "This is the extracted content from the article. It would normally contain the full text content of the web page.".to_string(),
        author: Some("John Doe".to_string()),
        published_date: Some("2024-01-15".to_string()),
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
                "uri": "tavily://search/{query}",
                "name": "Tavily search results",
                "description": "Cached search results for a specific query",
                "mimeType": "application/json"
            }
        ]
    }))
}

async fn read_resource(request: Request) -> Result<CallResponse, Error> {
    // Mock implementation - replace with actual cached search results
    Ok(CallResponse::from_value(json!({
        "contents": [{
            "uri": "tavily://search/example",
            "mimeType": "application/json",
            "text": "{\"query\": \"example\", \"results\": []}"
        }]
    })))
}