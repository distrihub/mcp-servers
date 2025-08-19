use crate::scraper_tools::{ElementExtractor, ScrapingSession, XPathAlternative};
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
    let resources = if let Ok(base) = Url::parse("https://distribot.local/") {
        [
            "crawl", "scrape", "extract", "select", "forms", "tables", "metadata",
        ]
        .iter()
        .filter_map(|r| {
            base.join(r).ok().map(|uri| Resource {
                uri,
                name: r.to_string(),
                description: Some(format!("DistriBot {} results", r)),
                mime_type: Some("application/json".to_string()),
            })
        })
        .collect()
    } else {
        vec![]
    };
    ResourcesListResponse {
        resources,
        next_cursor: None,
        meta: None,
    }
}

fn register_tools<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    register_scrape_tool(server)?;
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
    register_xpath_to_css_tool(server)?;
    register_advanced_scrape_tool(server)?;

    Ok(())
}

fn register_scrape_tool<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    let scrape_tool = Tool {
        name: "scrape".to_string(),
        description: Some(
            "Scrape a single webpage and extract basic content using readability".to_string(),
        ),
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

    server.register_tool(scrape_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;

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
                    if let Some(input) = page.get_bytes() {
                        let url = Url::parse(url)?;
                        let mut reader = std::io::Cursor::new(input);
                        let product = extract(&mut reader, &url)?;
                        json!({
                            "content": product.content,
                            "text": product.text,
                        })
                    } else {
                        is_error = Some(true);
                        json!({
                            "error": "No page content available",
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
                let metadata = extractor.extract_metadata();

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
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("url is missing")?;
                let pattern = args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .context("pattern is missing")?;

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);
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
                let xpath = args
                    .get("xpath")
                    .and_then(|v| v.as_str())
                    .context("xpath is missing")?;
                let show_common_patterns = args
                    .get("show_common_patterns")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

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

                let mut session = ScrapingSession::new()?;
                let html = session.fetch_page(url).await?;
                let extractor = ElementExtractor::new(&html);

                let mut response = json!({
                    "url": url
                });

                if include_metadata {
                    response["metadata"] = extractor.extract_metadata();
                }

                if include_links {
                    response["links"] = json!(extractor.extract_links()?);
                }

                if include_images {
                    response["images"] = json!(extractor.extract_images()?);
                }

                if include_forms {
                    response["forms"] = json!(extractor.extract_forms()?);
                }

                if include_tables {
                    response["tables"] = json!(extractor.extract_tables()?);
                }

                if include_structured_data {
                    response["structured_data"] = json!(extractor.extract_structured_data()?);
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
