use clap::{Args, Parser, Subcommand};
use mcp_spider::McpSpiderServer;
use tracing::{info, error, warn};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mcp-spider")]
#[command(about = "MCP server for web crawling and scraping using spider-rs with comprehensive features")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Log to file instead of stdout
    #[arg(long)]
    log_file: Option<PathBuf>,

    /// Port for HTTP server (if using HTTP transport)
    #[arg(short, long, default_value = "3002")]
    port: u16,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Chrome executable path (for screenshot functionality)
    #[arg(long)]
    chrome_path: Option<PathBuf>,

    /// Maximum concurrent requests
    #[arg(long, default_value = "10")]
    max_concurrency: u32,

    /// Default delay between requests in seconds
    #[arg(long, default_value = "1.0")]
    default_delay: f64,

    /// Enable stealth mode by default
    #[arg(long)]
    stealth_mode: bool,

    /// Enable cache by default
    #[arg(long)]
    enable_cache: bool,

    /// User agent string to use by default
    #[arg(long, default_value = "mcp-spider/1.0")]
    user_agent: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server (default)
    Serve {
        /// Use STDIO transport instead of HTTP
        #[arg(long)]
        stdio: bool,
    },
    /// Test spider functionality with a sample crawl
    Test {
        /// URL to test crawl
        #[arg(short, long, default_value = "https://example.com")]
        url: String,
        
        /// Test scraping instead of just crawling
        #[arg(long)]
        scrape: bool,
        
        /// Maximum pages to crawl in test
        #[arg(long, default_value = "5")]
        max_pages: u32,
    },
    /// Show configuration and capabilities
    Info,
    /// Generate example configuration file
    GenerateConfig {
        /// Output file path
        #[arg(short, long, default_value = "spider-config.json")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    initialize_logging(&cli)?;

    info!("Starting MCP Spider Server v{}", env!("CARGO_PKG_VERSION"));

    // Set Chrome path if provided
    if let Some(chrome_path) = &cli.chrome_path {
        std::env::set_var("CHROME_PATH", chrome_path);
        info!("Chrome path set to: {}", chrome_path.display());
    }

    match cli.command.unwrap_or(Commands::Serve { stdio: true }) {
        Commands::Serve { stdio } => {
            info!("Initializing MCP Spider server");
            
            let server = McpSpiderServer::new()
                .map_err(|e| anyhow::anyhow!("Failed to create MCP Spider server: {}", e))?;

            if stdio {
                info!("Using STDIO transport");
                server.serve().await?;
            } else {
                info!("Using HTTP transport on port {}", cli.port);
                // For HTTP transport, we'd need to implement an HTTP wrapper
                // For now, just use STDIO
                warn!("HTTP transport not yet implemented, falling back to STDIO");
                server.serve().await?;
            }
        }
        Commands::Test { url, scrape, max_pages } => {
            info!("Running test {} for URL: {}", if scrape { "scrape" } else { "crawl" }, url);
            
            let server = McpSpiderServer::new()
                .map_err(|e| anyhow::anyhow!("Failed to create MCP Spider server: {}", e))?;

            if scrape {
                test_scrape(&server, &url, max_pages).await?;
            } else {
                test_crawl(&server, &url, max_pages).await?;
            }
        }
        Commands::Info => {
            show_info(&cli);
        }
        Commands::GenerateConfig { output } => {
            generate_example_config(&output)?;
            info!("Example configuration written to: {}", output.display());
        }
    }

    Ok(())
}

fn initialize_logging(cli: &Cli) -> anyhow::Result<()> {
    let filter = if cli.debug {
        EnvFilter::new("debug,spider=debug,mcp_spider=debug")
    } else if cli.verbose {
        EnvFilter::new("info,spider=info,mcp_spider=debug")
    } else {
        EnvFilter::new("info,spider=warn")
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE);

    if let Some(log_file) = &cli.log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(log_file)?;
        
        subscriber.with_writer(file).init();
        println!("Logging to file: {}", log_file.display());
    } else {
        subscriber.init();
    }

    Ok(())
}

async fn test_crawl(server: &McpSpiderServer, url: &str, max_pages: u32) -> anyhow::Result<()> {
    use mcp_spider::CrawlRequest;
    use serde_json::json;
    use async_mcp::{ToolCall, Content};

    info!("Testing crawl functionality");

    let request = CrawlRequest {
        url: url.to_string(),
        headers: None,
        user_agent: Some("mcp-spider-test/1.0".to_string()),
        depth: Some(2),
        blacklist: None,
        whitelist: None,
        respect_robots_txt: Some(true),
        accept_invalid_certs: Some(false),
        subdomains: Some(false),
        tld: Some(false),
        delay: Some(2.0),
        budget_depth: Some(2),
        budget_request_timeout: Some(30.0),
        cache: Some(false),
        use_cookies: Some(false),
        stealth_mode: Some(false),
        chrome_intercept: Some(false),
        include_sitemap: Some(true),
        max_redirects: Some(3),
        max_file_size: Some(10 * 1024 * 1024), // 10MB
        concurrency: Some(std::cmp::min(max_pages, 5)),
        full_resources: Some(false),
    };

    let tool_call = ToolCall {
        name: "crawl".to_string(),
        arguments: serde_json::to_value(&request)?,
    };

    match server.handle_tool_call(tool_call).await {
        Ok(result) => {
            info!("✓ Crawl test completed successfully");
            
            if let Some(Content::Text { text }) = result.content.first() {
                println!("\n--- Crawl Results ---");
                
                // Parse and display results nicely
                if let Ok(crawl_result) = serde_json::from_str::<serde_json::Value>(text) {
                    if let Some(urls) = crawl_result.get("urls").and_then(|v| v.as_array()) {
                        println!("URLs found: {}", urls.len());
                        for (i, url) in urls.iter().take(10).enumerate() {
                            if let Some(url_str) = url.as_str() {
                                println!("  {}. {}", i + 1, url_str);
                            }
                        }
                        if urls.len() > 10 {
                            println!("  ... and {} more", urls.len() - 10);
                        }
                    }

                    if let Some(pages_crawled) = crawl_result.get("pages_crawled") {
                        println!("Pages crawled: {}", pages_crawled);
                    }

                    if let Some(duration) = crawl_result.get("duration_ms") {
                        println!("Duration: {}ms", duration);
                    }
                } else {
                    // Fallback to raw output
                    println!("{}", text);
                }
            }
        }
        Err(e) => {
            error!("✗ Crawl test failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn test_scrape(server: &McpSpiderServer, url: &str, max_pages: u32) -> anyhow::Result<()> {
    use mcp_spider::ScrapeRequest;
    use serde_json::json;
    use async_mcp::{ToolCall, Content};

    info!("Testing scrape functionality");

    let request = ScrapeRequest {
        url: url.to_string(),
        headers: None,
        user_agent: Some("mcp-spider-test/1.0".to_string()),
        depth: Some(1),
        blacklist: None,
        whitelist: None,
        respect_robots_txt: Some(true),
        accept_invalid_certs: Some(false),
        subdomains: Some(false),
        tld: Some(false),
        delay: Some(2.0),
        budget_depth: Some(1),
        budget_request_timeout: Some(30.0),
        cache: Some(false),
        use_cookies: Some(false),
        stealth_mode: Some(false),
        chrome_intercept: Some(false),
        include_sitemap: Some(false),
        max_redirects: Some(3),
        max_file_size: Some(10 * 1024 * 1024), // 10MB
        concurrency: Some(std::cmp::min(max_pages, 3)),
        full_resources: Some(false),
        extract_text: Some(true),
        extract_links: Some(true),
        extract_images: Some(true),
        extract_metadata: Some(true),
        take_screenshots: Some(false), // Disable for testing
        screenshot_params: None,
    };

    let tool_call = ToolCall {
        name: "scrape".to_string(),
        arguments: serde_json::to_value(&request)?,
    };

    match server.handle_tool_call(tool_call).await {
        Ok(result) => {
            info!("✓ Scrape test completed successfully");
            
            if let Some(Content::Text { text }) = result.content.first() {
                println!("\n--- Scrape Results ---");
                
                // Parse and display results nicely
                if let Ok(scrape_result) = serde_json::from_str::<serde_json::Value>(text) {
                    if let Some(pages) = scrape_result.get("pages").and_then(|v| v.as_array()) {
                        println!("Pages scraped: {}", pages.len());
                        
                        for (i, page) in pages.iter().take(3).enumerate() {
                            println!("\n--- Page {} ---", i + 1);
                            if let Some(url) = page.get("url").and_then(|v| v.as_str()) {
                                println!("URL: {}", url);
                            }
                            if let Some(title) = page.get("title").and_then(|v| v.as_str()) {
                                println!("Title: {}", title);
                            }
                            if let Some(links) = page.get("links").and_then(|v| v.as_array()) {
                                println!("Links found: {}", links.len());
                            }
                            if let Some(images) = page.get("images").and_then(|v| v.as_array()) {
                                println!("Images found: {}", images.len());
                            }
                            if let Some(text_content) = page.get("text_content").and_then(|v| v.as_str()) {
                                let preview = if text_content.len() > 200 {
                                    format!("{}...", &text_content[..200])
                                } else {
                                    text_content.to_string()
                                };
                                println!("Text preview: {}", preview);
                            }
                        }
                    }

                    if let Some(duration) = scrape_result.get("duration_ms") {
                        println!("\nDuration: {}ms", duration);
                    }
                } else {
                    // Fallback to raw output
                    println!("{}", text);
                }
            }
        }
        Err(e) => {
            error!("✗ Scrape test failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn show_info(cli: &Cli) {
    println!("MCP Spider Server v{}", env!("CARGO_PKG_VERSION"));
    println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    
    println!("Capabilities:");
    println!("  ✓ Web crawling with spider-rs");
    println!("  ✓ Content scraping and extraction");
    println!("  ✓ Link and image extraction");
    println!("  ✓ Metadata extraction (Open Graph, Twitter Cards, etc.)");
    println!("  ✓ Configurable crawl depth and concurrency");
    println!("  ✓ Robots.txt respect");
    println!("  ✓ URL blacklisting and whitelisting");
    println!("  ✓ Custom headers and user agents");
    println!("  ✓ Proxy support (planned)");
    println!("  ✓ Rate limiting and polite crawling");
    println!("  ✓ Caching support");
    println!("  ✓ Subdomain and TLD handling");
    
    #[cfg(feature = "chrome")]
    {
        println!("  ✓ Chrome browser integration");
        println!("  ✓ JavaScript rendering");
        println!("  ✓ Screenshot capture");
        println!("  ✓ Stealth mode");
    }
    
    #[cfg(not(feature = "chrome"))]
    {
        println!("  ✗ Chrome browser integration (feature disabled)");
        println!("  ✗ Screenshot capture (requires Chrome feature)");
    }

    println!();
    println!("Configuration:");
    println!("  Max Concurrency: {}", cli.max_concurrency);
    println!("  Default Delay: {}s", cli.default_delay);
    println!("  User Agent: {}", cli.user_agent);
    println!("  Stealth Mode: {}", if cli.stealth_mode { "enabled" } else { "disabled" });
    println!("  Cache: {}", if cli.enable_cache { "enabled" } else { "disabled" });
    
    if let Some(chrome_path) = &cli.chrome_path {
        println!("  Chrome Path: {}", chrome_path.display());
    } else {
        println!("  Chrome Path: system default");
    }

    println!();
    println!("MCP Tools:");
    println!("  • crawl - Crawl websites and return discovered URLs");
    println!("  • scrape - Scrape websites and extract content, links, and metadata");
    
    println!();
    println!("Resources:");
    println!("  • spider://crawl/{{url}} - Crawled website URLs and metadata");
    println!("  • spider://scrape/{{url}} - Scraped website content and extracted data");
}

fn generate_example_config(output: &PathBuf) -> anyhow::Result<()> {
    use serde_json::json;

    let config = json!({
        "crawl": {
            "respect_robots_txt": true,
            "delay": 1.0,
            "concurrency": 10,
            "subdomains": false,
            "tld": false,
            "cache": false,
            "use_cookies": false,
            "redirect_limit": 5,
            "accept_invalid_certs": false,
            "user_agent": "mcp-spider/1.0",
            "headers": {
                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                "Accept-Language": "en-US,en;q=0.5"
            },
            "blacklist": [
                ".*\\.pdf$",
                ".*\\.zip$",
                ".*/admin/.*",
                ".*/login.*"
            ],
            "whitelist": [],
            "budget": {
                "max_pages": 1000,
                "max_depth": 3,
                "max_duration_seconds": 300,
                "max_file_size_bytes": 10485760,
                "request_timeout_seconds": 30
            }
        },
        "scrape": {
            "extract_text": true,
            "extract_links": true,
            "extract_images": true,
            "extract_metadata": true,
            "take_screenshots": false,
            "screenshot_config": {
                "full_page": true,
                "quality": 90,
                "format": "png",
                "viewport_width": 1920,
                "viewport_height": 1080,
                "delay_before_screenshot_ms": 500
            }
        },
        "advanced": {
            "follow_redirects": true,
            "max_redirects": 5,
            "handle_javascript": false,
            "extract_resources": false,
            "respect_meta_robots": true,
            "ignore_query_params": false,
            "normalize_urls": true,
            "rate_limiting": {
                "requests_per_second": 2.0,
                "burst_size": 10,
                "delay_on_error_seconds": 5,
                "max_retries": 3
            }
        },
        "chrome": {
            "stealth_mode": false,
            "intercept_requests": false,
            "block_ads": false,
            "block_images": false,
            "block_javascript": false,
            "block_css": false,
            "viewport_width": 1920,
            "viewport_height": 1080,
            "user_agent": null,
            "extra_headers": {}
        }
    });

    let config_str = serde_json::to_string_pretty(&config)?;
    std::fs::write(output, config_str)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        use clap::Parser;
        
        let cli = Cli::try_parse_from(&["mcp-spider", "--debug", "--port", "3000"]).unwrap();
        assert!(cli.debug);
        assert_eq!(cli.port, 3000);
    }

    #[test]
    fn test_serve_command() {
        use clap::Parser;
        
        let cli = Cli::try_parse_from(&["mcp-spider", "serve", "--stdio"]).unwrap();
        if let Some(Commands::Serve { stdio }) = cli.command {
            assert!(stdio);
        } else {
            panic!("Expected Serve command");
        }
    }

    #[test]
    fn test_test_command() {
        use clap::Parser;
        
        let cli = Cli::try_parse_from(&[
            "mcp-spider", "test", 
            "--url", "https://example.com", 
            "--scrape", 
            "--max-pages", "10"
        ]).unwrap();
        
        if let Some(Commands::Test { url, scrape, max_pages }) = cli.command {
            assert_eq!(url, "https://example.com");
            assert!(scrape);
            assert_eq!(max_pages, 10);
        } else {
            panic!("Expected Test command");
        }
    }
}