use anyhow::Result;
use async_mcp::transport::ServerStdioTransport;
use clap::Parser;
use mcp_spider::build;

#[derive(Parser)]
#[command(name = "mcp-spider")]
#[command(about = "MCP server for web crawling and scraping using spider-rs")]
#[command(version)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// User agent string
    #[arg(long, default_value = "mcp-spider/0.1.0")]
    user_agent: String,

    /// Default delay between requests in seconds
    #[arg(long, default_value = "1.0")]
    delay: f64,

    /// Maximum crawl depth
    #[arg(long, default_value = "2")]
    max_depth: u16,

    /// Enable subdomain crawling
    #[arg(long)]
    subdomains: bool,

    /// Respect robots.txt
    #[arg(long, default_value = "true")]
    respect_robots: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up tracing
    let max_level = if cli.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(max_level)
        // needs to be stderr due to stdio transport
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting MCP Spider server");
    tracing::info!("Configuration:");
    tracing::info!("  User Agent: {}", cli.user_agent);
    tracing::info!("  Default Delay: {}s", cli.delay);
    tracing::info!("  Max Depth: {}", cli.max_depth);
    tracing::info!("  Subdomains: {}", cli.subdomains);
    tracing::info!("  Respect Robots: {}", cli.respect_robots);

    let server = build(ServerStdioTransport)?;
    let server_handle = tokio::spawn(async move { server.listen().await });

    server_handle
        .await?
        .map_err(|e| anyhow::anyhow!("Server error: {:#?}", e))?;
    Ok(())
}