use anyhow::Result;
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
    let base = Url::parse("https://distribot.local/").unwrap();
    let resources = [
        "crawl",
        "scrape",
        "get_links",
    ]
    .iter()
    .map(|&name| Resource {
        uri: base.join(name).unwrap(),
        name: name.to_string(),
        description: Some(format!("Spider {}", name)),
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
    // Basic crawl tool
    server.register_tool(
        Tool {
            name: "crawl".to_string(),
            description: Some("Crawl a website and return all discovered pages".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to crawl"
                    },
                    "max_pages": {
                        "type": "integer",
                        "description": "Maximum number of pages to crawl",
                        "default": 10
                    },
                    "respect_robots": {
                        "type": "boolean",
                        "description": "Whether to respect robots.txt",
                        "default": true
                    }
                },
                "required": ["url"]
            }),
            output_schema: Some(json!({
                "type": "object"
            })),
        },
        |req: CallToolRequest| {
            Box::pin(async move { handle_crawl(req).await })
        },
    );

    // Simple page scraping tool
    server.register_tool(
        Tool {
            name: "scrape_page".to_string(),
            description: Some("Scrape a single web page and return its content".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to scrape"
                    }
                },
                "required": ["url"]
            }),
            output_schema: Some(json!({
                "type": "object"
            })),
        },
        |req: CallToolRequest| {
            Box::pin(async move { handle_scrape_page(req).await })
        },
    );

    // Get links from a website
    server.register_tool(
        Tool {
            name: "get_links".to_string(),
            description: Some("Extract all links from a website".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to extract links from"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth to crawl for links",
                        "default": 1
                    }
                },
                "required": ["url"]
            }),
            output_schema: Some(json!({
                "type": "object"
            })),
        },
        |req: CallToolRequest| {
            Box::pin(async move { handle_get_links(req).await })
        },
    );

    Ok(())
}

async fn handle_crawl(req: CallToolRequest) -> Result<CallToolResponse> {
    let args = req.arguments.unwrap_or_default();
    let url = args["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("URL is required"))?;
    
    let max_pages = args["max_pages"].as_u64().unwrap_or(10) as u32;
    let respect_robots = args["respect_robots"].as_bool().unwrap_or(true);

    info!("Crawling {} with max_pages: {}", url, max_pages);

    // Create spider website instance
    let mut website = Website::new(url);
    
    // Configure the website
    website
        .with_limit(max_pages)
        .with_respect_robots_txt(respect_robots)
        .with_subdomains(false)
        .with_tld(false);

    // Start crawling
    website.crawl().await;

    // Get the pages
    let pages = website.get_pages();
    let links = website.get_links();

    let mut results = Vec::new();
    
    if let Some(pages) = pages {
        for page in pages.iter() {
            results.push(json!({
                "url": page.get_url(),
                "html": page.get_html(),
                "status_code": page.status_code.as_u16(),
            }));
        }
    }

    let response_data = json!({
        "pages": results,
        "total_pages": results.len(),
        "links_found": links.len(),
        "links": links.iter().map(|link| link.as_ref()).collect::<Vec<_>>()
    });

    Ok(CallToolResponse {
        content: vec![ToolResponseContent::Text {
            text: serde_json::to_string_pretty(&response_data)?,
        }],
        is_error: None,
        meta: None,
    })
}

async fn handle_scrape_page(req: CallToolRequest) -> Result<CallToolResponse> {
    let args = req.arguments.unwrap_or_default();
    let url = args["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("URL is required"))?;

    info!("Scraping single page: {}", url);

    // Create spider website instance for single page
    let mut website = Website::new(url);
    
    // Configure for single page scraping
    website
        .with_limit(1)
        .with_subdomains(false)
        .with_tld(false)
        .with_respect_robots_txt(false);

    // Start crawling
    website.crawl().await;

    // Get the page
    let pages = website.get_pages();
    
    if let Some(pages) = pages {
        if let Some(page) = pages.first() {
            let response_data = json!({
                "url": page.get_url(),
                "html": page.get_html(),
                "status_code": page.status_code.as_u16(),
                "content_length": page.get_html().len(),
            });

            return Ok(CallToolResponse {
                content: vec![ToolResponseContent::Text {
                    text: serde_json::to_string_pretty(&response_data)?,
                }],
                is_error: None,
                meta: None,
            });
        }
    }

    // If no page was scraped
    Ok(CallToolResponse {
        content: vec![ToolResponseContent::Text {
            text: json!({
                "error": "Failed to scrape page",
                "url": url
            }).to_string(),
        }],
        is_error: Some(true),
        meta: None,
    })
}

async fn handle_get_links(req: CallToolRequest) -> Result<CallToolResponse> {
    let args = req.arguments.unwrap_or_default();
    let url = args["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("URL is required"))?;
    
    let max_depth = args["max_depth"].as_u64().unwrap_or(1) as u16;

    info!("Extracting links from {} with max_depth: {}", url, max_depth);

    // Create spider website instance
    let mut website = Website::new(url);
    
    // Configure for link extraction
    website
        .with_limit(if max_depth > 1 { 50 } else { 1 })
        .with_subdomains(false)
        .with_tld(false)
        .with_respect_robots_txt(true);

    // Start crawling
    website.crawl().await;

    // Get the links
    let links = website.get_links();
    let pages = website.get_pages();

    let link_list: Vec<String> = links.iter().map(|link| link.as_ref().to_string()).collect();
    
    let response_data = json!({
        "url": url,
        "links": link_list,
        "total_links": link_list.len(),
        "pages_crawled": pages.as_ref().map_or(0, |p| p.len()),
    });

    Ok(CallToolResponse {
        content: vec![ToolResponseContent::Text {
            text: serde_json::to_string_pretty(&response_data)?,
        }],
        is_error: None,
        meta: None,
    })
}
