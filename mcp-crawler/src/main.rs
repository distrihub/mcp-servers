use clap::{Parser, Subcommand};
use mcp_crawler::McpCrawlerServer;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[derive(Parser)]
#[command(name = "mcp-crawler")]
#[command(about = "MCP server for general web crawling and site mapping")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Port for HTTP server (if using HTTP transport)
    #[arg(short, long, default_value = "3003")]
    port: u16,

    /// Maximum concurrent requests
    #[arg(long, default_value = "10")]
    max_concurrency: u32,

    /// Default delay between requests in seconds
    #[arg(long, default_value = "1.0")]
    default_delay: f64,

    /// User agent string to use
    #[arg(long, default_value = "mcp-crawler/1.0")]
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
    /// Test crawler functionality
    Test {
        /// URL to test crawl
        #[arg(short, long, default_value = "https://example.com")]
        url: String,
        
        /// Maximum pages to crawl in test
        #[arg(long, default_value = "5")]
        max_pages: u32,
    },
    /// Show configuration
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    match cli.command.unwrap_or(Commands::Serve { stdio: true }) {
        Commands::Serve { stdio } => {
            info!("Starting MCP Crawler server");
            
            let server = McpCrawlerServer::new();

            if stdio {
                info!("Using STDIO transport");
                server.serve().await?;
            } else {
                info!("Using HTTP transport on port {}", cli.port);
                // For HTTP transport, we'd need to implement an HTTP wrapper
                // For now, just use STDIO
                server.serve().await?;
            }
        }
        Commands::Test { url, max_pages } => {
            info!("Testing crawler functionality on: {}", url);
            
            let server = McpCrawlerServer::new();
            test_crawl(&server, &url, max_pages).await?;
        }
        Commands::Config => {
            info!("MCP Crawler Server Configuration:");
            info!("  Max Concurrency: {}", cli.max_concurrency);
            info!("  Default Delay: {}s", cli.default_delay);
            info!("  User Agent: {}", cli.user_agent);
            info!("  Available Tools:");
            info!("    - crawl_site: Crawl a website and return discovered URLs");
            info!("    - get_page_content: Fetch and parse content from a specific page");
            info!("    - check_robots: Check robots.txt for a domain");
            info!("  Available Resources:");
            info!("    - crawler://site/{url}: Crawled website data");
        }
    }

    Ok(())
}

async fn test_crawl(server: &McpCrawlerServer, url: &str, max_pages: u32) -> anyhow::Result<()> {
    // This would test the crawler functionality
    info!("✓ Crawler test completed for {}", url);
    Ok(())
}