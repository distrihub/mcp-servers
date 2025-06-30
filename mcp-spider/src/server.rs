use anyhow::{Context, Result};
use async_mcp::server::{Server, ServerBuilder};
use async_mcp::transport::Transport;
use async_mcp::types::{
    CallToolRequest, CallToolResponse, ListRequest, PromptsListResponse, Resource,
    ResourcesListResponse, ServerCapabilities, Tool, ToolResponseContent,
};
use serde_json::json;
use spider::website::Website;
use tracing::info;
use url::Url;

pub fn build<T: Transport>(t: T) -> Result<Server<T>> {
    let mut server = Server::builder(t)
        .capabilities(ServerCapabilities {
            tools: Some(json!({})),
            ..Default::default()
        })
        .request_handler("resources/list", |_req: ListRequest| {
            Box::pin(async move { Ok(list_resources()) })
        })
        .request_handler("prompts/list", |_req: ListRequest| {
            Box::pin(async move {
                Ok(PromptsListResponse {
                    prompts: vec![],
                    next_cursor: None,
                    meta: None,
                })
            })
        });

    register_tools(&mut server)?;

    let server = server.build();

    Ok(server)
}

fn list_resources() -> ResourcesListResponse {
    let base = Url::parse("https://spider.local/").unwrap();
    let resources = ["crawl", "scrape"]
        .iter()
        .map(|r| Resource {
            uri: base.join(r).unwrap(),
            name: r.to_string(),
            description: Some(format!("Spider {} results", r)),
            mime_type: Some("application/json".to_string()),
        })
        .collect();
    ResourcesListResponse {
        resources,
        next_cursor: None,
        meta: None,
    }
}

fn register_tools<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    // Crawl Tool
    let crawl_tool = Tool {
        name: "crawl".to_string(),
        description: Some("Crawl websites and return discovered URLs using spider-rs".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to start crawling from",
                    "format": "uri"
                },
                "depth": {
                    "type": "integer",
                    "description": "Maximum crawl depth (default: 2)",
                    "minimum": 0,
                    "default": 2
                },
                "subdomains": {
                    "type": "boolean",
                    "description": "Whether to crawl subdomains",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "urls": {"type": "array", "items": {"type": "string"}},
                "pages_crawled": {"type": "integer"},
                "duration_ms": {"type": "integer"}
            },
        })),
    };

    // Scrape Tool (simplified)
    let scrape_tool = Tool {
        name: "scrape".to_string(),
        description: Some("Scrape a single webpage and extract basic content".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "content": {"type": "string"},
                "links": {"type": "array", "items": {"type": "string"}},
                "duration_ms": {"type": "integer"}
            },
        })),
    };

    // Register crawl tool
    server.register_tool(crawl_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let _depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2);
                let _subdomains = args.get("subdomains").and_then(|v| v.as_bool()).unwrap_or(false);

                info!("Starting crawl of {}", url);

                let start_time = std::time::Instant::now();
                
                // Simple crawl using spider
                let mut website = Website::new(url);
                website.crawl().await;

                let urls: Vec<String> = website.get_links()
                    .iter()
                    .map(|link| link.as_ref().to_string())
                    .collect();

                let duration = start_time.elapsed();
                let pages_crawled = urls.len();

                let result = json!({
                    "urls": urls,
                    "pages_crawled": pages_crawled,
                    "duration_ms": duration.as_millis()
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling crawl request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error: {}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register scrape tool
    server.register_tool(scrape_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;

                info!("Starting scrape of {}", url);

                let start_time = std::time::Instant::now();
                
                // Simple HTTP request to get page content
                let client = reqwest::Client::new();
                let response = client.get(url).send().await?;
                let html = response.text().await?;

                // Basic link extraction
                let document = scraper::Html::parse_document(&html);
                let link_selector = scraper::Selector::parse("a[href]").unwrap();
                
                let links: Vec<String> = document
                    .select(&link_selector)
                    .filter_map(|el| el.value().attr("href"))
                    .map(|href| href.to_string())
                    .collect();

                let duration = start_time.elapsed();

                let result = json!({
                    "url": url,
                    "content": html.chars().take(5000).collect::<String>(), // Limit content size
                    "links": links,
                    "duration_ms": duration.as_millis()
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling scrape request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("Error: {}", e),
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