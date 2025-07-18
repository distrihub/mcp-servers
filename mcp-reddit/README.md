# MCP Reddit Server

A Model Context Protocol (MCP) server implementation for Reddit API integration, providing comprehensive access to Reddit's data through various tools.

## Features

This MCP server provides the following tools for interacting with Reddit:

### Core Tools

- **`get_posts`** - Get posts from a subreddit with various sorting options
- **`search_posts`** - Search for posts across Reddit or within specific subreddits
- **`get_comments`** - Retrieve comments for a specific post
- **`get_subreddit_info`** - Get detailed information about a subreddit
- **`get_user_info`** - Get user profile information
- **`get_trending_subreddits`** - Get popular/trending subreddits
- **`get_user_posts`** - Get posts submitted by a specific user
- **`get_user_comments`** - Get comments made by a specific user

## Setup

### Prerequisites

1. **Reddit API Credentials**: You need to create a Reddit application to get API credentials
   - Go to https://www.reddit.com/prefs/apps
   - Click "Create App" or "Create Another App"
   - Choose "script" as the app type
   - Note down your Client ID and Client Secret

### Environment Variables

Set the following environment variables:

```bash
export REDDIT_CLIENT_ID="your_client_id_here"
export REDDIT_CLIENT_SECRET="your_client_secret_here"
export REDDIT_USER_AGENT="YourAppName/1.0 (by /u/YourUsername)"  # Optional, defaults to "MCP-Reddit-Server/1.0"
```

### Building

```bash
# Build the Reddit MCP server
cargo build --release --bin mcp-reddit

# Or build all servers including Reddit
cargo build --release
```

## Usage

### Claude Desktop Integration

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "reddit": {
      "command": "/path/to/mcp-servers/target/release/mcp-reddit",
      "env": {
        "REDDIT_CLIENT_ID": "your_client_id",
        "REDDIT_CLIENT_SECRET": "your_client_secret",
        "REDDIT_USER_AGENT": "YourAppName/1.0 (by /u/YourUsername)"
      }
    }
  }
}
```

### distri Framework Integration

```yaml
# distri-config.yaml
mcp_servers:
  - name: reddit
    command: mcp-reddit
    env:
      REDDIT_CLIENT_ID: "{{secrets.reddit_client_id}}"
      REDDIT_CLIENT_SECRET: "{{secrets.reddit_client_secret}}"
      REDDIT_USER_AGENT: "MCP-Reddit-Server/1.0"
```

## Tool Reference

### get_posts

Get posts from a subreddit.

**Parameters:**
- `subreddit` (required): The subreddit name (e.g., "programming", "rust")
- `sort` (optional): Sort order - "hot", "new", "top", "rising" (default: "hot")
- `limit` (optional): Number of posts to return (default: 25)

**Example:**
```json
{
  "name": "get_posts",
  "arguments": {
    "subreddit": "rust",
    "sort": "hot",
    "limit": 10
  }
}
```

### search_posts

Search for posts on Reddit.

**Parameters:**
- `query` (required): Search query
- `subreddit` (optional): Specific subreddit to search in
- `sort` (optional): Sort order - "relevance", "hot", "top", "new", "comments" (default: "relevance")
- `limit` (optional): Number of posts to return (default: 25)

**Example:**
```json
{
  "name": "search_posts",
  "arguments": {
    "query": "async rust",
    "subreddit": "rust",
    "sort": "relevance",
    "limit": 15
  }
}
```

### get_comments

Get comments for a specific post.

**Parameters:**
- `post_id` (required): The Reddit post ID
- `limit` (optional): Number of comments to return (default: 25)

**Example:**
```json
{
  "name": "get_comments",
  "arguments": {
    "post_id": "abc123",
    "limit": 50
  }
}
```

### get_subreddit_info

Get detailed information about a subreddit.

**Parameters:**
- `subreddit` (required): The subreddit name

**Example:**
```json
{
  "name": "get_subreddit_info",
  "arguments": {
    "subreddit": "rust"
  }
}
```

### get_user_info

Get information about a Reddit user.

**Parameters:**
- `username` (required): The Reddit username

**Example:**
```json
{
  "name": "get_user_info",
  "arguments": {
    "username": "some_user"
  }
}
```

### get_trending_subreddits

Get popular/trending subreddits.

**Parameters:**
- `limit` (optional): Number of subreddits to return (default: 25)

**Example:**
```json
{
  "name": "get_trending_subreddits",
  "arguments": {
    "limit": 20
  }
}
```

### get_user_posts

Get posts submitted by a specific user.

**Parameters:**
- `username` (required): The Reddit username
- `limit` (optional): Number of posts to return (default: 25)

**Example:**
```json
{
  "name": "get_user_posts",
  "arguments": {
    "username": "some_user",
    "limit": 10
  }
}
```

### get_user_comments

Get comments made by a specific user.

**Parameters:**
- `username` (required): The Reddit username
- `limit` (optional): Number of comments to return (default: 25)

**Example:**
```json
{
  "name": "get_user_comments",
  "arguments": {
    "username": "some_user",
    "limit": 15
  }
}
```

## API Rate Limits

The Reddit API has rate limits that this server respects:
- 60 requests per minute for authenticated requests
- The server uses client credentials flow for authentication
- Rate limiting is handled automatically by the underlying HTTP client

## Error Handling

The server provides detailed error messages for common issues:
- Missing API credentials
- Invalid subreddit names
- Network connectivity issues
- API rate limit exceeded

## Testing

Run the tests to verify the server is working correctly:

```bash
cargo test --bin mcp-reddit
```

## Resources

The server provides the following resources:
- `reddit://posts` - Reddit posts
- `reddit://comments` - Reddit comments  
- `reddit://subreddits` - Subreddit information
- `reddit://users` - User information

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo clippy`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.