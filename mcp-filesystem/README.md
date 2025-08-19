# MCP Filesystem Server

A Model Context Protocol (MCP) server implementation for filesystem operations, providing comprehensive file and directory management capabilities for AI agents.

## Features

This MCP server provides the following tools for filesystem operations:

### File Operations
- **`read_file`** - Read the complete contents of a file
- **`write_file`** - Write content to a file (creates directories as needed)
- **`get_file_info`** - Get detailed file/directory metadata
- **`delete_file`** - Delete files or directories (recursive for directories)
- **`move_file`** - Move or rename files and directories

### Directory Operations  
- **`list_directory`** - List directory contents with [FILE]/[DIR] prefixes
- **`create_directory`** - Create directories (including parent directories)
- **`search_files`** - Recursively search for files matching a pattern

## Setup

### Building

```bash
# Build the filesystem MCP server
cargo build --release --bin mcp-filesystem

# Or build all servers including filesystem
cargo build --release
```

## Usage

### Claude Desktop Integration

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "/path/to/mcp-servers/target/release/mcp-filesystem"
    }
  }
}
```

### distri Framework Integration

```yaml
# distri-config.yaml
mcp_servers:
  - name: filesystem
    command: mcp-filesystem
    args: []
```

## Tools

### read_file
Read the complete contents of a text file.

```json
{
  "path": "/path/to/file.txt"
}
```

### write_file
Write content to a file, creating directories as needed.

```json
{
  "path": "/path/to/file.txt",
  "content": "Hello, World!"
}
```

### list_directory
List all files and directories in a path.

```json
{
  "path": "/path/to/directory"
}
```

### create_directory
Create a new directory (and parent directories if needed).

```json
{
  "path": "/path/to/new/directory"
}
```

### delete_file
Delete a file or directory (recursive for directories).

```json
{
  "path": "/path/to/file/or/directory"
}
```

### move_file
Move or rename a file or directory.

```json
{
  "from": "/path/to/source",
  "to": "/path/to/destination"
}
```

### search_files
Recursively search for files and directories matching a pattern.

```json
{
  "path": "/path/to/search/root",
  "pattern": "*.txt"
}
```

### get_file_info
Get detailed metadata about a file or directory.

```json
{
  "path": "/path/to/file/or/directory"
}
```

## Security Considerations

This server provides full filesystem access. When using it:

- Be cautious with delete operations as they cannot be undone
- The server supports `~` home directory expansion for convenience
- All operations respect filesystem permissions
- Consider restricting the server's access to specific directories if needed

## Testing

Run the test suite:

```bash
cargo test --package mcp-filesystem
```

## Debugging

Use the MCP Inspector to debug the server:

```bash
npx @modelcontextprotocol/inspector ./target/release/mcp-filesystem
```