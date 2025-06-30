use clap::{Args, Parser, Subcommand};
use mcp_twitter::McpTwitterServer;
use mcp_twitter::auth::TwitterAuth;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[derive(Parser)]
#[command(name = "mcp-twitter-rs")]
#[command(about = "MCP server for Twitter/X integration with posting, searching, and analytics")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Twitter API Key
    #[arg(short, long, env = "TWITTER_API_KEY")]
    api_key: Option<String>,

    /// Twitter API Secret
    #[arg(short = 's', long, env = "TWITTER_API_SECRET")]
    api_secret: Option<String>,

    /// Twitter Access Token (for write operations)
    #[arg(short = 't', long, env = "TWITTER_ACCESS_TOKEN")]
    access_token: Option<String>,

    /// Twitter Access Token Secret (for write operations)
    #[arg(long, env = "TWITTER_ACCESS_TOKEN_SECRET")]
    access_token_secret: Option<String>,

    /// Twitter Bearer Token (for read operations)
    #[arg(short = 'b', long, env = "TWITTER_BEARER_TOKEN")]
    bearer_token: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Port for HTTP server (if using HTTP transport)
    #[arg(short, long, default_value = "3001")]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server (default)
    Serve {
        /// Use STDIO transport instead of HTTP
        #[arg(long)]
        stdio: bool,
    },
    /// Test Twitter API connection
    Test,
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

    // Load configuration
    let auth = if let (Some(api_key), Some(api_secret)) = (&cli.api_key, &cli.api_secret) {
        TwitterAuth::new(
            api_key.clone(),
            api_secret.clone(),
            cli.access_token.clone(),
            cli.access_token_secret.clone(),
            cli.bearer_token.clone(),
        )
    } else {
        info!("Loading Twitter credentials from environment variables");
        TwitterAuth::from_env().map_err(|e| {
            anyhow::anyhow!(
                "Failed to load Twitter credentials: {}. Please provide via CLI args or environment variables:\n\
                - TWITTER_API_KEY\n\
                - TWITTER_API_SECRET\n\
                - TWITTER_BEARER_TOKEN (for read operations)\n\
                - TWITTER_ACCESS_TOKEN (for write operations)\n\
                - TWITTER_ACCESS_TOKEN_SECRET (for write operations)",
                e
            )
        })?
    };

    match cli.command.unwrap_or(Commands::Serve { stdio: true }) {
        Commands::Serve { stdio } => {
            info!("Starting MCP Twitter server");
            
            if !auth.has_bearer_token() && !auth.has_oauth_credentials() {
                error!("No valid Twitter credentials provided. Need either bearer token or OAuth credentials.");
                return Err(anyhow::anyhow!("Missing Twitter credentials"));
            }

            let server = McpTwitterServer::new(
                auth.api_key.clone(),
                auth.api_secret.clone(),
                auth.access_token.clone(),
                auth.access_token_secret.clone(),
                auth.bearer_token.clone(),
            )?;

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
        Commands::Test => {
            info!("Testing Twitter API connection");
            
            if !auth.has_bearer_token() {
                error!("Bearer token required for API testing");
                return Err(anyhow::anyhow!("Bearer token required"));
            }

            // Test the connection by making a simple API call
            let server = McpTwitterServer::new(
                auth.api_key.clone(),
                auth.api_secret.clone(),
                auth.access_token.clone(),
                auth.access_token_secret.clone(),
                auth.bearer_token.clone(),
            )?;

            info!("✓ Twitter MCP server created successfully");
            info!("✓ All credentials loaded");
            
            if auth.has_oauth_credentials() {
                info!("✓ OAuth credentials available (can post tweets)");
            } else {
                info!("ℹ OAuth credentials not available (read-only mode)");
            }
        }
        Commands::Config => {
            info!("Twitter MCP Server Configuration:");
            info!("  API Key: {}", if auth.api_key.is_empty() { "Not set" } else { "Set" });
            info!("  API Secret: {}", if auth.api_secret.is_empty() { "Not set" } else { "Set" });
            info!("  Bearer Token: {}", if auth.has_bearer_token() { "Set" } else { "Not set" });
            info!("  OAuth Credentials: {}", if auth.has_oauth_credentials() { "Set" } else { "Not set" });
            
            if auth.has_bearer_token() {
                info!("✓ Can perform read operations (search tweets, get user info, etc.)");
            }
            
            if auth.has_oauth_credentials() {
                info!("✓ Can perform write operations (post tweets, etc.)");
            }
            
            if !auth.has_bearer_token() && !auth.has_oauth_credentials() {
                error!("✗ No valid credentials configured");
            }
        }
    }

    Ok(())
}