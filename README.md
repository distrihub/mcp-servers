# MCP Servers - Rust Implementation

A collection of MCP (Model Context Protocol) server implementations in Rust, designed for integration with AI agents and the [distri framework](https://github.com/distrihub/distri).

## Overview

This workspace contains multiple MCP server implementations, each providing specialized functionality for different use cases:

## Available MCP Servers

| Server          | Status             | Description                                        | Tools                                                                                      | Resources                                                                    | Use Cases                                           |
| --------------- | ------------------ | -------------------------------------------------- | ------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- | --------------------------------------------------- |
| **mcp-tavily**  | 🔧 Work in Progress | Tavily search API integration                      | `search`, `search_news`, `get_extract`                                                     | `tavily://search/{query}`                                                    | AI-powered web search, news aggregation             |
| **mcp-coder**   | 🔧 Work in Progress | File system operations, code analysis & formatting | `read_file`, `write_file`, `search_files`, `list_directory`, `get_project_structure`       | `file://{path}`, `directory://{path}`                                        | Code development, file management, project analysis |
| **mcp-twitter** | 🔧 Work in Progress | Twitter/X API integration                          | `post_tweet`, `search_tweets`, `get_user_info`, `get_user_timeline`, `get_tweet_analytics` | `twitter://user/{id}`, `twitter://tweet/{id}`, `twitter://trends/{location}` | Social media automation, content posting, analytics |
| **mcp-spider**  | 🔧 Work in Progress | Advanced web crawling with spider-rs               | `scrape`                                                                                   | `spider://scrape/{url}`                                                      | Web data extraction, site mapping, content analysis |






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
      "args": ["--stealth-mode"]
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
       args: ["--stealth-mode"]
     - name: tavily
       command: mcp-tavily
       env:
         TAVILY_API_KEY: "{{secrets.tavily_key}}"
     - name: kg
       command: mcp-kg
       args: ["--data-path", "{{workspace}}/knowledge_graph"]
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

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects
- [distri framework](https://github.com/distrihub/distri) - AI Agent Framework
- [Model Context Protocol](https://modelcontextprotocol.io/) - Official MCP documentation
- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk) - TypeScript implementation