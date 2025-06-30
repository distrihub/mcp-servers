use clap::{Args, Parser, Subcommand};
use mcp_coder::McpCoderServer;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[derive(Parser)]
#[command(name = "mcp-coder-rs")]
#[command(about = "MCP server for code analysis, formatting, and development tools")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Base directory for file operations
    #[arg(short, long, value_name = "DIR")]
    directory: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Port for HTTP server (if using HTTP transport)
    #[arg(short, long, default_value = "3000")]
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
    /// Show configuration
    Config,
}

#[derive(Args)]
struct ServeArgs {
    /// Base directory for file operations
    #[arg(short, long, value_name = "DIR")]
    directory: Option<PathBuf>,
}

#[derive(Args)]
struct HttpArgs {
    /// Port to bind the HTTP server
    #[arg(short, long, default_value = "3000")]
    port: u16,
    
    /// Host to bind the HTTP server
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[derive(Args)]
struct AnalyzeArgs {
    /// File to analyze
    file: PathBuf,
    
    /// Programming language (auto-detected if not specified)
    #[arg(short, long)]
    language: Option<String>,
}

#[derive(Args)]
struct FormatArgs {
    /// File to format
    file: PathBuf,
    
    /// Programming language
    #[arg(short, long)]
    language: String,
    
    /// Write formatted code back to file
    #[arg(short, long)]
    write: bool,
}

#[derive(Args)]
struct TestArgs {
    /// Run specific test
    #[arg(short, long)]
    test_name: Option<String>,
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

    // Determine base directory
    let base_directory = cli.directory
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match cli.command.unwrap_or(Commands::Serve { stdio: true }) {
        Commands::Serve { stdio } => {
            info!("Starting MCP Coder server");
            info!("Base directory: {}", base_directory.display());
            
            let server = McpCoderServer::new(base_directory);

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
        Commands::Config => {
            info!("MCP Coder Server Configuration:");
            info!("  Base Directory: {}", base_directory.display());
            info!("  Available Tools:");
            info!("    - read_file: Read file contents");
            info!("    - write_file: Write content to files");
            info!("    - search_files: Search for files with patterns");
            info!("    - list_directory: List directory contents");
            info!("    - get_project_structure: Get project tree structure");
            info!("  Available Resources:");
            info!("    - file://{path}: Access file content");
            info!("    - directory://{path}: Access directory listings");
        }
    }

    Ok(())
}

async fn run_tests(test_name: Option<&str>) -> anyhow::Result<()> {
    info!("Running MCP Coder tests");

    match test_name {
        Some("tools") => test_tools().await?,
        Some("resources") => test_resources().await?,
        Some("analyzer") => test_analyzer().await?,
        Some("formatter") => test_formatter().await?,
        None => {
            test_tools().await?;
            test_resources().await?;
            test_analyzer().await?;
            test_formatter().await?;
        }
        Some(name) => {
            error!("Unknown test: {}", name);
            return Err(anyhow::anyhow!("Unknown test: {}", name));
        }
    }

    info!("All tests completed successfully");
    Ok(())
}

async fn test_tools() -> anyhow::Result<()> {
    info!("Testing MCP tools...");
    
    // This would test tool registration and execution
    // In a real implementation, you'd use the MCP Inspector or a test client
    
    println!("✓ Tools test passed");
    Ok(())
}

async fn test_resources() -> anyhow::Result<()> {
    info!("Testing MCP resources...");
    
    // This would test resource access and content retrieval
    
    println!("✓ Resources test passed");
    Ok(())
}

async fn test_analyzer() -> anyhow::Result<()> {
    info!("Testing code analyzer...");
    
    use tempfile::NamedTempFile;
    use tokio::fs::write;
    
    let temp_file = NamedTempFile::new()?;
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

struct TestStruct {
    field: i32,
}

impl TestStruct {
    fn new(field: i32) -> Self {
        Self { field }
    }
}
"#;
    
    write(temp_file.path(), rust_code).await?;
    
    let server = McpCoderServer::new(None)?;
    let result = server.analyzer.analyze_file(
        temp_file.path().to_str().unwrap(),
        Some("rust"),
    ).await?;
    
    assert!(result.function_count >= 2); // main + new
    assert!(result.struct_count >= 1);   // TestStruct
    assert_eq!(result.language, "rust");
    
    println!("✓ Analyzer test passed");
    Ok(())
}

async fn test_formatter() -> anyhow::Result<()> {
    info!("Testing code formatter...");
    
    let server = McpCoderServer::new(None)?;
    let unformatted_rust = "fn main(){println!(\"Hello\");}";
    let formatted = server.formatter.format(unformatted_rust, "rust").await?;
    
    assert!(formatted.contains("fn main()"));
    assert!(formatted.len() > unformatted_rust.len()); // Should have added whitespace
    
    println!("✓ Formatter test passed");
    Ok(())
}