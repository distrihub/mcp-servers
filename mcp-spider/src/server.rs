use anyhow::{Context, Result};
use async_mcp::server::{Server, ServerBuilder};
use async_mcp::transport::Transport;
use async_mcp::types::{
    CallToolRequest, CallToolResponse, ListRequest, PromptsListResponse, Resource,
    ResourcesListResponse, ServerCapabilities, Tool, ToolResponseContent,
};
use readability::extractor::extract;
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
                "content": {"type": "string"},
                "text": {"type": "string"}
            },
            "additionalProperties": false
        })),
    };

    // Register scrape tool
    server.register_tool(scrape_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;

                // Simple crawl using spider
                let mut website = Website::new(url)
                    .with_user_agent(Some("DistriBot"))
                    .with_subdomains(false)
                    .with_limit(1)
                    .with_tld(false)
                    .with_redirect_limit(3)
                    .with_respect_robots_txt(false)
                    .build()?;

                website.scrape().await;

                let page = website
                    .get_pages()
                    .map(|pages| pages.iter().next())
                    .flatten();

                let mut is_error = None;
                let content = if let Some(page) = page {
                    let input = page.get_bytes().unwrap();
                    let url = Url::parse(url)?;
                    let mut reader = std::io::Cursor::new(input);
                    let product = extract(&mut reader, &url)?;
                    let result = json!({
                        "content": product.content,
                        "text": product.text,
                    });
                    result
                } else {
                    is_error = Some(true);
                    json!({
                        "error": "No page found",
                    })
                };

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&content)?,
                    }],
                    is_error: is_error,
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

    Ok(())
}
