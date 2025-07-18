use crate::scraper_tools::{
    create_mock_page, ElementExtractor, ScrapingOptions, SpiderSession, WebAutomation,
    XPathAlternative,
};
use anyhow::{Context, Result};
use async_mcp::server::{Server, ServerBuilder};
use async_mcp::transport::Transport;
use async_mcp::types::{
    CallToolRequest, CallToolResponse, ListRequest, PromptsListResponse, Resource,
    ResourcesListResponse, ServerCapabilities, Tool, ToolResponseContent,
};
use base64::Engine;
use serde_json::json;
use spider::website::Website;
use std::collections::HashMap;
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
        "extract",
        "select",
        "forms",
        "tables",
        "metadata",
        "chrome_scrape",
        "automation",
        "screenshot",
        "wait_element",
        "execute_js",
    ]
    .iter()
    .map(|r| Resource {
        uri: base.join(r).unwrap(),
        name: r.to_string(),
        description: Some(format!("DistriBot {} results", r)),
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
    // Basic scraping tools
    register_scrape_tool(server)?;
    register_chrome_scrape_tool(server)?;
    register_advanced_scrape_tool(server)?;

    // Element extraction tools
    register_select_elements_tool(server)?;
    register_extract_text_tool(server)?;
    register_extract_attributes_tool(server)?;
    register_extract_links_tool(server)?;
    register_extract_images_tool(server)?;
    register_extract_forms_tool(server)?;
    register_extract_tables_tool(server)?;
    register_extract_metadata_tool(server)?;
    register_search_patterns_tool(server)?;
    register_extract_structured_data_tool(server)?;

    // XPath tools
    register_xpath_to_css_tool(server)?;

    // Web automation tools
    register_click_element_tool(server)?;
    register_fill_form_tool(server)?;
    register_submit_form_tool(server)?;
    register_take_screenshot_tool(server)?;
    register_wait_for_element_tool(server)?;
    register_execute_javascript_tool(server)?;

    Ok(())
}

fn register_scrape_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let scrape_tool = Tool {
        name: "scrape".to_string(),
        description: Some("Scrape a single webpage using spider".to_string()),
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
                "html": {"type": "string"},
                "url": {"type": "string"}
            },
            "additionalProperties": false
        })),
    };

    server.register_tool(scrape_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;

                let mut website = Website::new(url);
                website
                    .with_user_agent(Some("mcp-spider/1.0"))
                    .with_subdomains(false)
                    .with_limit(1)
                    .with_tld(false)
                    .with_redirect_limit(3)
                    .with_respect_robots_txt(false);

                website.crawl().await;

                let mut is_error = None;
                let content = if let Some(pages) = website.get_pages() {
                    if let Some(page) = pages.first() {
                        json!({
                            "html": page.get_html(),
                            "url": page.get_url(),
                        })
                    } else {
                        is_error = Some(true);
                        json!({
                            "error": "No pages found",
                        })
                    }
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

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_chrome_scrape_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "chrome_scrape".to_string(),
        description: Some(
            "Scrape a webpage using Chrome headless browser with JavaScript support (simulated)"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "stealth_mode": {
                    "type": "boolean",
                    "description": "Enable stealth mode to avoid detection",
                    "default": false
                },
                "take_screenshot": {
                    "type": "boolean",
                    "description": "Take a screenshot of the page",
                    "default": false
                },
                "wait_for_selector": {
                    "type": "string",
                    "description": "CSS selector to wait for before extracting content"
                },
                "timeout_seconds": {
                    "type": "number",
                    "description": "Timeout in seconds",
                    "default": 30
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "html": {"type": "string"},
                "url": {"type": "string"},
                "screenshot": {"type": "string"}
            }
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let stealth_mode = args
                    .get("stealth_mode")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let take_screenshot = args
                    .get("take_screenshot")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let _wait_for_selector = args.get("wait_for_selector").and_then(|v| v.as_str());
                let _timeout_seconds = args
                    .get("timeout_seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(30);

                let options = ScrapingOptions {
                    use_chrome: true,
                    stealth_mode,
                    take_screenshot,
                    ..Default::default()
                };

                let session = SpiderSession::new().with_chrome().with_stealth();
                let scraping_result = session.scrape_with_options(url, options).await?;

                let content = json!({
                    "html": scraping_result.html,
                    "url": scraping_result.url,
                    "screenshot": scraping_result.screenshot,
                    "note": "Chrome features are simulated in this version"
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&content)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_select_elements_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "select_elements".to_string(),
        description: Some("Select elements from HTML using CSS selectors".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector to match elements"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url", "selector"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "object"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let selector = args
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .context("selector is missing")?;
                let _use_chrome = args
                    .get("use_chrome")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let elements = extractor.select_elements(selector)?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&elements)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_text_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_text".to_string(),
        description: Some("Extract text content from elements matching a CSS selector".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector to match elements"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url", "selector"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "string"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let selector = args
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .context("selector is missing")?;
                let _use_chrome = args
                    .get("use_chrome")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let texts = extractor.extract_text(selector)?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&texts)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_attributes_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_attributes".to_string(),
        description: Some(
            "Extract specific attribute values from elements matching a CSS selector".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector to match elements"
                },
                "attribute": {
                    "type": "string",
                    "description": "Attribute name to extract"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url", "selector", "attribute"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "string"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let selector = args
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .context("selector is missing")?;
                let attribute = args
                    .get("attribute")
                    .and_then(|v| v.as_str())
                    .context("attribute is missing")?;
                let _use_chrome = args
                    .get("use_chrome")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let attributes = extractor.extract_attributes(selector, attribute)?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&attributes)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_links_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_links".to_string(),
        description: Some("Extract all links from a webpage".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "object"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let _use_chrome = args["use_chrome"].as_bool().unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let links = extractor.extract_links()?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&links)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_images_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_images".to_string(),
        description: Some("Extract all images from a webpage".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "object"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let _use_chrome = args
                    .get("use_chrome")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let images = extractor.extract_images()?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&images)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_forms_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_forms".to_string(),
        description: Some("Extract all forms and their fields from a webpage".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "object"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let _use_chrome = args["use_chrome"].as_bool().unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let forms = extractor.extract_forms()?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&forms)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_tables_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_tables".to_string(),
        description: Some("Extract all tables with headers and data from a webpage".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "object"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let _use_chrome = args["use_chrome"].as_bool().unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let tables = extractor.extract_tables()?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&tables)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_metadata_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "extract_metadata".to_string(),
        description: Some(
            "Extract page metadata including title, description, and Open Graph data".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object"
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let _use_chrome = args["use_chrome"].as_bool().unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let metadata = extractor.extract_metadata()?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&metadata)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_search_patterns_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "search_patterns".to_string(),
        description: Some("Search for text patterns using regular expressions".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url", "pattern"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "string"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let pattern = args["pattern"].as_str().context("pattern is missing")?;
                let _use_chrome = args["use_chrome"].as_bool().unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let matches = extractor.search_patterns(pattern)?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&matches)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_extract_structured_data_tool<T: Transport>(
    server: &mut ServerBuilder<T>,
) -> Result<()> {
    let tool = Tool {
        name: "extract_structured_data".to_string(),
        description: Some(
            "Extract structured data (JSON-LD, microdata) from a webpage".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "array",
            "items": {"type": "object"}
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let _use_chrome = args["use_chrome"].as_bool().unwrap_or(false);

                let session = SpiderSession::new();
                let html = session.fetch_page(url).await?;

                let page = create_mock_page(url, &html);
                let extractor = ElementExtractor::new(&html, page);
                let structured_data = extractor.extract_structured_data()?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&structured_data)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_xpath_to_css_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "xpath_to_css".to_string(),
        description: Some(
            "Convert XPath expressions to CSS selectors and show common XPath alternatives"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "xpath": {
                    "type": "string",
                    "description": "XPath expression to convert"
                },
                "show_common_patterns": {
                    "type": "boolean",
                    "description": "Whether to show common XPath to CSS pattern mappings",
                    "default": false
                }
            },
            "required": ["xpath"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object"
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let xpath = args["xpath"].as_str().context("xpath is missing")?;
                let show_common_patterns = args["show_common_patterns"].as_bool().unwrap_or(false);

                let css = XPathAlternative::xpath_to_css(xpath)?;
                let mut response = json!({
                    "xpath": xpath,
                    "css": css
                });

                if show_common_patterns {
                    response["common_patterns"] = json!(XPathAlternative::common_patterns());
                }

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_advanced_scrape_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "advanced_scrape".to_string(),
        description: Some(
            "Perform comprehensive scraping of a webpage extracting all available data".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to scrape",
                    "format": "uri"
                },
                "use_chrome": {
                    "type": "boolean",
                    "description": "Use Chrome for JavaScript-heavy sites",
                    "default": false
                },
                "stealth_mode": {
                    "type": "boolean",
                    "description": "Enable stealth mode",
                    "default": false
                },
                "take_screenshot": {
                    "type": "boolean",
                    "description": "Take a screenshot",
                    "default": false
                },
                "include_links": {
                    "type": "boolean",
                    "description": "Include links extraction",
                    "default": true
                },
                "include_images": {
                    "type": "boolean",
                    "description": "Include images extraction",
                    "default": true
                },
                "include_forms": {
                    "type": "boolean",
                    "description": "Include forms extraction",
                    "default": true
                },
                "include_tables": {
                    "type": "boolean",
                    "description": "Include tables extraction",
                    "default": true
                },
                "include_metadata": {
                    "type": "boolean",
                    "description": "Include metadata extraction",
                    "default": true
                },
                "include_structured_data": {
                    "type": "boolean",
                    "description": "Include structured data extraction",
                    "default": true
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object"
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let use_chrome = args
                    .get("use_chrome")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let stealth_mode = args
                    .get("stealth_mode")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let take_screenshot = args
                    .get("take_screenshot")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let include_links = args
                    .get("include_links")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let include_images = args
                    .get("include_images")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let include_forms = args
                    .get("include_forms")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let include_tables = args
                    .get("include_tables")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let include_metadata = args
                    .get("include_metadata")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let include_structured_data = args
                    .get("include_structured_data")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let options = ScrapingOptions {
                    use_chrome,
                    stealth_mode,
                    take_screenshot,
                    extract_metadata: include_metadata,
                    extract_links: include_links,
                    extract_images: include_images,
                    extract_forms: include_forms,
                    extract_tables: include_tables,
                    extract_structured_data: include_structured_data,
                    ..Default::default()
                };

                let session = if use_chrome {
                    SpiderSession::new().with_chrome().with_stealth()
                } else {
                    SpiderSession::new()
                };

                let scraping_result = session.scrape_with_options(url, options).await?;

                let mut response = json!({
                    "url": scraping_result.url,
                    "html": scraping_result.html
                });

                if let Some(metadata) = scraping_result.metadata {
                    response["metadata"] = metadata;
                }

                if let Some(links) = scraping_result.links {
                    response["links"] = json!(links);
                }

                if let Some(images) = scraping_result.images {
                    response["images"] = json!(images);
                }

                if let Some(forms) = scraping_result.forms {
                    response["forms"] = json!(forms);
                }

                if let Some(tables) = scraping_result.tables {
                    response["tables"] = json!(tables);
                }

                if let Some(structured_data) = scraping_result.structured_data {
                    response["structured_data"] = json!(structured_data);
                }

                if let Some(screenshot) = scraping_result.screenshot {
                    response["screenshot"] = json!(screenshot);
                }

                if use_chrome {
                    response["note"] = json!("Chrome features are simulated in this version");
                }

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

// Web automation tools
fn register_click_element_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "click_element".to_string(),
        description: Some(
            "Click an element on a webpage using Chrome automation (simulated)".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector of the element to click"
                }
            },
            "required": ["url", "selector"],
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

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let selector = args["selector"].as_str().context("selector is missing")?;

                let automation = WebAutomation::new();
                let result_message = automation.click_element(url, selector).await?;

                let response = json!({
                    "success": true,
                    "message": result_message
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_fill_form_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "fill_form".to_string(),
        description: Some(
            "Fill form fields on a webpage using Chrome automation (simulated)".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to",
                    "format": "uri"
                },
                "form_data": {
                    "type": "object",
                    "description": "Form field names and values to fill",
                    "additionalProperties": {"type": "string"}
                }
            },
            "required": ["url", "form_data"],
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

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let form_data_json = args["form_data"]
                    .as_object()
                    .context("form_data is missing")?;

                let mut form_data = HashMap::new();
                for (key, value) in form_data_json {
                    if let Some(val_str) = value.as_str() {
                        form_data.insert(key.clone(), val_str.to_string());
                    }
                }

                let automation = WebAutomation::new();
                let result_message = automation.fill_form(url, form_data).await?;

                let response = json!({
                    "success": true,
                    "message": result_message
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_submit_form_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "submit_form".to_string(),
        description: Some(
            "Submit a form on a webpage using Chrome automation (simulated)".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to",
                    "format": "uri"
                },
                "form_selector": {
                    "type": "string",
                    "description": "CSS selector of the form to submit"
                }
            },
            "required": ["url", "form_selector"],
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

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let form_selector = args["form_selector"]
                    .as_str()
                    .context("form_selector is missing")?;

                let automation = WebAutomation::new();
                let result_message = automation.submit_form(url, form_selector).await?;

                let response = json!({
                    "success": true,
                    "message": result_message
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_take_screenshot_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "take_screenshot".to_string(),
        description: Some(
            "Take a screenshot of a webpage or specific element using Chrome (simulated)"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector of element to screenshot (optional, full page if not provided)"
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "screenshot_data": {"type": "string"},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let selector = args["selector"].as_str();

                let automation = WebAutomation::new();
                let screenshot_data = automation.take_screenshot(url, selector).await?;

                let response = json!({
                    "success": true,
                    "screenshot_data": base64::engine::general_purpose::STANDARD.encode(&screenshot_data),
                    "message": "Screenshot taken successfully (simulated)"
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_wait_for_element_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "wait_for_element".to_string(),
        description: Some(
            "Wait for an element to appear on a webpage using Chrome automation (simulated)"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to",
                    "format": "uri"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector of the element to wait for"
                },
                "timeout_ms": {
                    "type": "number",
                    "description": "Timeout in milliseconds",
                    "default": 30000
                }
            },
            "required": ["url", "selector"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "found": {"type": "boolean"},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let selector = args["selector"].as_str().context("selector is missing")?;
                let timeout_ms = args["timeout_ms"].as_u64().unwrap_or(30000);

                let automation = WebAutomation::new();
                let found = automation.wait_for_element(url, selector, timeout_ms).await?;

                let response = json!({
                    "success": true,
                    "found": found,
                    "message": if found { "Element found (simulated)" } else { "Element not found within timeout (simulated)" }
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn register_execute_javascript_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let tool = Tool {
        name: "execute_javascript".to_string(),
        description: Some(
            "Execute JavaScript code on a webpage using Chrome automation (simulated)".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to",
                    "format": "uri"
                },
                "script": {
                    "type": "string",
                    "description": "JavaScript code to execute"
                }
            },
            "required": ["url", "script"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {},
                "message": {"type": "string"}
            }
        })),
    };

    server.register_tool(tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args["url"].as_str().context("url is missing")?;
                let script = args["script"].as_str().context("script is missing")?;

                let automation = WebAutomation::new();
                let execution_result = automation.execute_javascript(url, script).await?;

                let response = json!({
                    "success": true,
                    "result": execution_result,
                    "message": "JavaScript executed successfully (simulated)"
                });

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            handle_tool_result(result)
        })
    });

    Ok(())
}

fn handle_tool_result(
    result: Result<CallToolResponse, anyhow::Error>,
) -> Result<CallToolResponse, anyhow::Error> {
    match result {
        Ok(response) => Ok(response),
        Err(e) => {
            info!("Error handling tool request: {:#?}", e);
            Ok(CallToolResponse {
                content: vec![ToolResponseContent::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
                meta: None,
            })
        }
    }
}
