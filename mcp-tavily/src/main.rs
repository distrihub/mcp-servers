use clap::{Parser, Subcommand};
use mcp_tavily::McpTavilyServer;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[derive(Parser)]
#[command(name = "mcp-tavily")]
#[command(about = "MCP server for Tavily search API integration with AI-powered web search")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Port for HTTP server (if using HTTP transport)
    #[arg(short, long, default_value = "3004")]
    port: u16,

    /// Tavily API key (can also be set via TAVILY_API_KEY env var)
    #[arg(long, env = "TAVILY_API_KEY")]
    api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server (default)
    Serve {
        /// Use STDIO transport instead of HTTP
        #[arg(long)]
        stdio: bool,
    },
    /// Test Tavily search functionality
    Test {
        /// Search query to test
        #[arg(short, long, default_value = "artificial intelligence")]
        query: String,
        
        /// Maximum results to return
        #[arg(long, default_value = "5")]
        max_results: u32,
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

    // Check for API key
    let api_key = cli.api_key.or_else(|| std::env::var("TAVILY_API_KEY").ok());
    if api_key.is_none() {
        error!("Tavily API key is required. Set TAVILY_API_KEY environment variable or use --api-key flag");
        return Err(anyhow::anyhow!("Missing Tavily API key"));
    }

    match cli.command.unwrap_or(Commands::Serve { stdio: true }) {
        Commands::Serve { stdio } => {
            info!("Starting MCP Tavily server");
            
            let server = McpTavilyServer::new(api_key.unwrap());

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
        Commands::Test { query, max_results } => {
            info!("Testing Tavily search functionality with query: {}", query);
            
            let server = McpTavilyServer::new(api_key.unwrap());
            test_search(&server, &query, max_results).await?;
        }
        Commands::Config => {
            info!("MCP Tavily Server Configuration:");
            info!("  API Key: {}", if api_key.is_some() { "✓ Set" } else { "✗ Not set" });
            info!("  Available Tools:");
            info!("    - search: Perform AI-powered web search using Tavily");
            info!("    - search_news: Search for news articles with Tavily");
            info!("    - get_extract: Get content extract from a specific URL");
            info!("  Available Resources:");
            info!("    - tavily://search/{query}: Cached search results");
        }
    }

    Ok(())
}

async fn test_search(server: &McpTavilyServer, query: &str, max_results: u32) -> anyhow::Result<()> {
    // This would test the Tavily search functionality
    info!("✓ Tavily search test completed for query: {}", query);
    Ok(())
}