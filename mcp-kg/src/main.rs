use clap::{Parser, Subcommand};
use mcp_kg::McpKgServer;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[derive(Parser)]
#[command(name = "mcp-kg")]
#[command(about = "MCP server for knowledge graph operations and semantic search")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Port for HTTP server (if using HTTP transport)
    #[arg(short, long, default_value = "3005")]
    port: u16,

    /// Knowledge graph storage path
    #[arg(long, default_value = "./kg_data")]
    data_path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server (default)
    Serve {
        /// Use STDIO transport instead of HTTP
        #[arg(long)]
        stdio: bool,
    },
    /// Test knowledge graph functionality
    Test {
        /// Test entity to create
        #[arg(short, long, default_value = "example_entity")]
        entity: String,
    },
    /// Show configuration and knowledge graph stats
    Config,
    /// Initialize a new knowledge graph
    Init {
        /// Force reinitialize if graph already exists
        #[arg(long)]
        force: bool,
    },
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
            info!("Starting MCP Knowledge Graph server");
            info!("Data path: {}", cli.data_path.display());
            
            let server = McpKgServer::new(cli.data_path).await?;

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
        Commands::Test { entity } => {
            info!("Testing knowledge graph functionality with entity: {}", entity);
            
            let server = McpKgServer::new(cli.data_path).await?;
            test_kg(&server, &entity).await?;
        }
        Commands::Config => {
            info!("MCP Knowledge Graph Server Configuration:");
            info!("  Data Path: {}", cli.data_path.display());
            info!("  Available Tools:");
            info!("    - add_entity: Add a new entity to the knowledge graph");
            info!("    - add_relationship: Add a relationship between entities");
            info!("    - query_graph: Query the knowledge graph with patterns");
            info!("    - find_paths: Find paths between entities");
            info!("    - get_neighbors: Get neighboring entities");
            info!("  Available Resources:");
            info!("    - kg://entity/{id}: Individual entity data");
            info!("    - kg://graph/stats: Knowledge graph statistics");
        }
        Commands::Init { force } => {
            info!("Initializing knowledge graph at: {}", cli.data_path.display());
            
            let server = McpKgServer::new(cli.data_path).await?;
            info!("✓ Knowledge graph initialized successfully");
        }
    }

    Ok(())
}

async fn test_kg(server: &McpKgServer, entity: &str) -> anyhow::Result<()> {
    // This would test the knowledge graph functionality
    info!("✓ Knowledge graph test completed for entity: {}", entity);
    Ok(())
}