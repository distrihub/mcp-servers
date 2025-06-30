# MCP Servers - Rust Implementation

A collection of MCP (Model Context Protocol) server implementations in Rust, specifically designed for integration with the [distri framework](https://github.com/distrihub/distri).

## Overview

This workspace contains two MCP server implementations:

- **mcp-coder**: File system operations, code reading/writing, and project structure analysis
- **mcp-twitter**: Twitter/X API integration for posting tweets, searching, and user analytics

## Prerequisites

- Rust 1.70 or later
- For mcp-twitter-rs: Twitter API credentials

## Installation

### Building from source

```bash
# Clone the repository
git clone https://github.com/distrihub/mcp-servers
cd mcp-servers

# Build all servers
cargo build --release

# Or build individual servers
cargo build --release --bin mcp-coder
cargo build --release --bin mcp-twitter
```

## Usage with STDIO

Both servers are designed to work with the Model Context Protocol over STDIO, making them compatible with MCP clients like Claude Desktop, VS Code extensions, and the distri framework.

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

#### Available Tools

- `read_file`: Read the contents of a file
- `write_file`: Write content to a file
- `search_files`: Search for files with regex patterns and file type filters
- `list_directory`: List contents of a directory
- `get_project_structure`: Get hierarchical project structure

#### Available Resources

- `file://{path}`: Access file content directly
- `directory://{path}`: Access directory listings

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

# Or provide credentials via CLI
./target/release/mcp-twitter \
  --api-key "your_key" \
  --api-secret "your_secret" \
  --bearer-token "your_bearer_token"

# Test the connection
./target/release/mcp-twitter test

# Show configuration
./target/release/mcp-twitter config
```

#### Available Tools

- `post_tweet`: Post a tweet (requires OAuth credentials)
- `search_tweets`: Search for tweets using Twitter's search API
- `get_user_info`: Get information about a Twitter user
- `get_user_timeline`: Get recent tweets from a user's timeline
- `get_tweet_analytics`: Get analytics data (requires special API access)

#### Available Resources

- `twitter://user/{user_id}`: Twitter user profile information
- `twitter://tweet/{tweet_id}`: Individual tweet content and metadata
- `twitter://trends/{location}`: Trending topics (placeholder for v1.1 compatibility)

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
   ```

3. **Use in distri tasks**:
   ```python
   # Example distri task using MCP servers
   import distri
   
   @distri.task
   async def analyze_and_tweet(project_path: str):
       # Use coder server to analyze project
       structure = await distri.mcp.call("coder", "get_project_structure", {
           "path": project_path,
           "max_depth": 2
       })
       
       # Use twitter server to share results
       await distri.mcp.call("twitter", "post_tweet", {
           "text": f"Just analyzed a project with {len(structure['children'])} top-level items!"
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
└── mcp-twitter/              # Twitter integration MCP server
    ├── Cargo.toml
    └── src/
        ├── main.rs           # CLI entry point
        ├── lib.rs            # Core implementation
        ├── auth.rs           # Twitter authentication
        ├── twitter_client.rs # Twitter API client
        └── models.rs         # Data models
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific server
cargo test --package mcp-coder
cargo test --package mcp-twitter

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

Both servers support the MCP debugging tools recommended by the [Model Context Protocol documentation](https://modelcontextprotocol.io/docs/tools/debugging).

### Using MCP Inspector

```bash
# Install MCP Inspector (if not already installed)
npm install -g @modelcontextprotocol/inspector

# Debug the coder server
npx @modelcontextprotocol/inspector ./target/release/mcp-coder

# Debug the twitter server
npx @modelcontextprotocol/inspector ./target/release/mcp-twitter
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

## API Credentials

### Twitter API Setup

1. Create a Twitter Developer account at https://developer.twitter.com
2. Create a new app and generate API keys
3. For read-only operations, you only need the Bearer Token
4. For posting tweets, you need OAuth 1.0a credentials (API Key, API Secret, Access Token, Access Token Secret)

### Environment Variables

The servers support loading credentials from environment variables:

```bash
# Twitter credentials
export TWITTER_API_KEY="your_api_key"
export TWITTER_API_SECRET="your_api_secret"
export TWITTER_BEARER_TOKEN="your_bearer_token"
export TWITTER_ACCESS_TOKEN="your_access_token"
export TWITTER_ACCESS_TOKEN_SECRET="your_access_token_secret"
```

## Troubleshooting

### Common Issues

1. **Permission denied errors**: Ensure the servers have read/write permissions for the specified directories
2. **Twitter API errors**: Check your API credentials and rate limits
3. **STDIO communication issues**: Verify JSON-RPC message formatting

### Debug Mode

Both servers support debug logging:

```bash
./target/release/mcp-coder --debug
./target/release/mcp-twitter --debug
```

### Logging

The servers use the `tracing` crate for logging. Set the `RUST_LOG` environment variable for more detailed logs:

```bash
RUST_LOG=debug ./target/release/mcp-coder
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects

- [distri framework](https://github.com/distrihub/distri) - Distributed computing platform
- [Model Context Protocol](https://modelcontextprotocol.io/) - Official MCP documentation
- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk) - TypeScript implementation 