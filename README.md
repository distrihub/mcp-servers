# MCP Servers - Rust Implementation

A collection of MCP (Model Context Protocol) server implementations in Rust, designed for integration with AI agents and the [distri framework](https://github.com/distrihub/distri).

## Overview

This workspace contains multiple MCP server implementations, each providing specialized functionality for different use cases:

## Available MCP Servers

| Server          | Status             | Description                                        | Tools                                                                                      | Resources                                                                    | Use Cases                                           |
| --------------- | ------------------ | -------------------------------------------------- | ------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- | --------------------------------------------------- |
| **mcp-coder**   | ✅ Active           | File system operations, code analysis & formatting | `read_file`, `write_file`, `search_files`, `list_directory`, `get_project_structure`       | `file://{path}`, `directory://{path}`                                        | Code development, file management, project analysis |
| **mcp-twitter** | ✅ Active           | Twitter/X API integration                          | `post_tweet`, `search_tweets`, `get_user_info`, `get_user_timeline`, `get_tweet_analytics` | `twitter://user/{id}`, `twitter://tweet/{id}`, `twitter://trends/{location}` | Social media automation, content posting, analytics |
| **mcp-spider**  | ✅ Working          | Advanced web crawling with spider-rs               | `crawl`, `scrape`                                                                          | `spider://crawl/{url}`, `spider://scrape/{url}`                              | Web data extraction, site mapping, content analysis |

| **mcp-tavily** | 🔧 Work in Progress | Tavily search API integration | `search`, `search_news`, `get_extract` | `tavily://search/{query}` | AI-powered web search, news aggregation |


### Server Details

#### 🔹 mcp-coder
**Purpose**: File system operations and code development tools
- **Port**: 3000 (HTTP mode)
- **Base directory**: Configurable via `--directory` flag
- **Key Features**: Safe file operations, project structure analysis, code formatting
- **Dependencies**: Standard file system libraries

#### 🔹 mcp-twitter  
**Purpose**: Twitter/X platform integration
- **Port**: 3001 (HTTP mode)
- **Authentication**: Twitter API v2 credentials required
- **Key Features**: Tweet posting, user analytics, search functionality
- **Rate Limits**: Respects Twitter API rate limits

#### 🔹 mcp-spider
**Purpose**: Comprehensive web crawling capabilities
- **Port**: 3002 (HTTP mode)  
- **Key Features**: Website crawling, content extraction, link discovery, robots.txt respect
- **Spider Integration**: Fast and reliable web crawling using spider-rs engine
- **Scalability**: High-performance concurrent crawling

#### 🔹 mcp-crawler
**Purpose**: Simple and reliable web crawling
- **Port**: 3003 (HTTP mode)
- **Key Features**: robots.txt compliance, content extraction, link discovery
- **Focus**: Simplicity and reliability over advanced features

#### 🔹 mcp-tavily
**Purpose**: AI-powered search through Tavily API
- **Port**: 3004 (HTTP mode)
- **Authentication**: Tavily API key required
- **Key Features**: Semantic search, news search, content extraction
- **Intelligence**: AI-generated summaries and answers

#### 🔹 mcp-kg
**Purpose**: Knowledge graph operations and management
- **Port**: 3005 (HTTP mode)
- **Storage**: Local graph database
- **Key Features**: Entity management, relationship mapping, path finding
- **Query Language**: Pattern-based graph queries

## Prerequisites

- Rust 1.70 or later
- OpenSSL development libraries (`libssl-dev` on Ubuntu/Debian)
- For mcp-twitter: Twitter API credentials
- For mcp-tavily: Tavily API key
- All MCP servers use async-mcp for protocol communication

## Installation

### Building from source

```bash
# Clone the repository
git clone https://github.com/your-org/mcp-servers
cd mcp-servers

# Build all servers
cargo build --release

# Or build individual servers
cargo build --release --bin mcp-coder
cargo build --release --bin mcp-twitter
cargo build --release --bin mcp-spider
cargo build --release --bin mcp-tavily
```

## Usage with STDIO

All servers are designed to work with the Model Context Protocol over STDIO, making them compatible with MCP clients like Claude Desktop, VS Code extensions, and the distri framework.

### mcp-coder

The coder server provides file system operations and basic code analysis.

```bash
# Start the server (default: stdio mode)
./target/release/mcp-coder

# Specify a base directory
./target/release/mcp-coder --directory /path/to/project

# Show configuration
./target/release/mcp-coder config

# Enable debug logging
./target/release/mcp-coder --debug serve
```

### mcp-twitter

The Twitter server provides integration with Twitter/X API v2.

```bash
# Set up environment variables
export TWITTER_API_KEY="your_api_key"
export TWITTER_API_SECRET="your_api_secret"
export TWITTER_BEARER_TOKEN="your_bearer_token"
export TWITTER_ACCESS_TOKEN="your_access_token"          # Optional, for posting
export TWITTER_ACCESS_TOKEN_SECRET="your_token_secret"   # Optional, for posting

# Start the server
./target/release/mcp-twitter

# Test the connection
./target/release/mcp-twitter test

# Show configuration
./target/release/mcp-twitter config
```

### mcp-spider

The spider server provides advanced web crawling capabilities.

```bash
# Start the server
./target/release/mcp-spider

# Start with debug logging
./target/release/mcp-spider --debug

# With custom options
./target/release/mcp-spider --user-agent "MyBot/1.0" --delay 2.0 --max-depth 3
```

### mcp-crawler

Simple web crawler for basic crawling needs.

```bash
# Start the server
./target/release/mcp-crawler

# Test crawling
./target/release/mcp-crawler test --url https://example.com

# Show configuration
./target/release/mcp-crawler config
```

### mcp-tavily

AI-powered search using Tavily API.

```bash
# Set up API key
export TAVILY_API_KEY="your_api_key"

# Start the server
./target/release/mcp-tavily

# Test search
./target/release/mcp-tavily test --query "artificial intelligence"
```

### mcp-kg

Knowledge graph operations and management.

```bash
# Start the server with custom data path
./target/release/mcp-kg --data-path ./my_knowledge_graph

# Initialize a new knowledge graph
./target/release/mcp-kg init

# Show configuration and stats
./target/release/mcp-kg config
```

## Integration with MCP Clients

### Claude Desktop

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "coder": {
      "command": "/path/to/mcp-servers/target/release/mcp-coder",
      "args": ["--directory", "/path/to/your/project"]
    },
    "twitter": {
      "command": "/path/to/mcp-servers/target/release/mcp-twitter",
      "env": {
        "TWITTER_API_KEY": "your_key",
        "TWITTER_API_SECRET": "your_secret",
        "TWITTER_BEARER_TOKEN": "your_bearer_token"
      }
    },
    "spider": {
      "command": "/path/to/mcp-servers/target/release/mcp-spider",
              "args": ["--debug"]
    },
    "crawler": {
      "command": "/path/to/mcp-servers/target/release/mcp-crawler"
    },
    "tavily": {
      "command": "/path/to/mcp-servers/target/release/mcp-tavily",
      "env": {
        "TAVILY_API_KEY": "your_api_key"
      }
    },
    "kg": {
      "command": "/path/to/mcp-servers/target/release/mcp-kg",
      "args": ["--data-path", "/path/to/kg/data"]
    }
  }
}
```

### distri Framework Integration

These servers are specifically designed to work with the [distri framework](https://github.com/distrihub/distri). The distri framework provides a distributed computing platform where these MCP servers can be used as tools for various tasks.

#### Using with distri

1. **Install distri framework**:
   ```bash
   # Follow installation instructions from distri repository
   git clone https://github.com/distrihub/distri
   ```

2. **Configure MCP servers in distri**:
   ```yaml
   # distri-config.yaml
   mcp_servers:
     - name: coder
       command: mcp-coder
       args: ["--directory", "{{workspace}}"]
     - name: twitter
       command: mcp-twitter
       env:
         TWITTER_BEARER_TOKEN: "{{secrets.twitter_bearer}}"
     - name: spider
       command: mcp-spider
       args: ["--debug"]
     - name: tavily
       command: mcp-tavily
       env:
         TAVILY_API_KEY: "{{secrets.tavily_key}}"
     - name: kg
       command: mcp-kg
       args: ["--data-path", "{{workspace}}/knowledge_graph"]
   ```

3. **Use in distri tasks**:
   ```python
   # Example distri task using MCP servers
   import distri
   
   @distri.task
   async def analyze_and_share(project_path: str, query: str):
       # Use coder server to analyze project
       structure = await distri.mcp.call("coder", "get_project_structure", {
           "path": project_path,
           "max_depth": 2
       })
       
       # Use tavily to research the topic
       research = await distri.mcp.call("tavily", "search", {
           "query": query,
           "max_results": 5
       })
       
       # Use knowledge graph to store relationships
       await distri.mcp.call("kg", "add_entity", {
           "id": f"project_{project_path.replace('/', '_')}",
           "label": f"Project: {project_path}",
           "entity_type": "software_project",
           "properties": {"file_count": len(structure.get("children", []))}
       })
       
       # Use twitter to share insights
       await distri.mcp.call("twitter", "post_tweet", {
           "text": f"Analyzed project with {len(structure['children'])} components. Research shows: {research['answer'][:100]}..."
       })
   ```

## Development

### Project Structure

```
mcp-servers/
├── Cargo.toml                 # Workspace configuration
├── .gitignore                 # Git ignore rules
├── README.md                  # This file
├── mcp-coder/                # File operations MCP server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # CLI entry point
│       └── lib.rs            # Core implementation
├── mcp-twitter/              # Twitter integration MCP server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # CLI entry point
│       ├── lib.rs            # Core implementation
│       ├── auth.rs           # Twitter authentication
│       ├── twitter_client.rs # Twitter API client
│       └── models.rs         # Data models
├── mcp-spider/               # Advanced web crawling server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # CLI entry point
│       ├── lib.rs            # Core implementation
│       ├── crawler.rs        # Spider crawler
│       ├── scraper.rs        # Content scraper
│       ├── config.rs         # Configuration
│       └── utils.rs          # Utilities
├── mcp-crawler/              # Simple web crawler server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # CLI entry point
│       └── lib.rs            # Core implementation
├── mcp-tavily/               # Tavily search integration
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # CLI entry point
│       └── lib.rs            # Core implementation
└── mcp-kg/                   # Knowledge graph server
    ├── Cargo.toml
    └── src/
        ├── main.rs           # CLI entry point
        └── lib.rs            # Core implementation
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific server
cargo test --package mcp-coder
cargo test --package mcp-twitter
cargo test --package mcp-spider
cargo test --package mcp-crawler
cargo test --package mcp-tavily
cargo test --package mcp-kg

# Run with output
cargo test -- --nocapture
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo clippy`
6. Submit a pull request

## MCP Debugging

All servers support the MCP debugging tools recommended by the [Model Context Protocol documentation](https://modelcontextprotocol.io/docs/tools/debugging).

### Using MCP Inspector

```bash
# Install MCP Inspector (if not already installed)
npm install -g @modelcontextprotocol/inspector

# Debug any server
npx @modelcontextprotocol/inspector ./target/release/mcp-coder
npx @modelcontextprotocol/inspector ./target/release/mcp-twitter
npx @modelcontextprotocol/inspector ./target/release/mcp-spider
npx @modelcontextprotocol/inspector ./target/release/mcp-tavily

```

### Manual Testing with stdio

You can manually test the servers by sending JSON-RPC messages:

```bash
# Start server in debug mode
./target/release/mcp-coder --debug

# Send initialization message
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | ./target/release/mcp-coder

# List available tools
echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}' | ./target/release/mcp-coder
```


## Troubleshooting

### Common Issues

1. **Permission denied errors**: Ensure the servers have read/write permissions for specified directories
2. **Twitter API errors**: Check API credentials and rate limits
3. **Spider crawling errors**: Check robots.txt compliance and rate limiting
4. **STDIO communication issues**: Verify JSON-RPC message formatting
5. **OpenSSL errors**: Install development libraries (`sudo apt install libssl-dev pkg-config`)

### Debug Mode

All servers support debug logging:

```bash
./target/release/mcp-coder --debug
./target/release/mcp-twitter --debug
./target/release/mcp-spider --debug
./target/release/mcp-crawler --debug
./target/release/mcp-tavily --debug
./target/release/mcp-kg --debug
```

### Logging

The servers use the `tracing` crate for logging. Set the `RUST_LOG` environment variable for more detailed logs:

```bash
RUST_LOG=debug ./target/release/mcp-coder
```

### Server Status

Check server status and capabilities:

```bash
# Check configuration and available tools
./target/release/mcp-coder config
./target/release/mcp-twitter config
./target/release/mcp-spider --help
./target/release/mcp-crawler config
./target/release/mcp-tavily config
./target/release/mcp-kg config
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects

- [distri framework](https://github.com/distrihub/distri) - AI Agent Framework
- [Model Context Protocol](https://modelcontextprotocol.io/) - Official MCP documentation
- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk) - TypeScript implementation
- [spider-rs](https://github.com/spider-rs/spider) - High-performance web crawler
- [Tavily](https://tavily.com) - AI-powered search API 