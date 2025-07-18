# MCP Spider

A Model Context Protocol (MCP) server for web crawling and scraping using [spider-rs](https://github.com/spider-rs/spider).

## Overview

This MCP server provides simple and efficient web crawling capabilities using the powerful spider-rs library. It focuses on basic, reliable functionality that works well as a foundation.

## Features

- **Basic Web Crawling**: Crawl websites with configurable limits
- **Single Page Scraping**: Extract content from individual web pages  
- **Link Extraction**: Get all links from a website with depth control
- **Proper spider-rs Integration**: Uses the latest spider-rs APIs correctly
- **Async/Concurrent**: Built on tokio for high performance

## Tools

### `crawl`
Crawl a website and return all discovered pages.

**Parameters:**
- `url` (required): The URL to crawl
- `max_pages` (optional): Maximum number of pages to crawl (default: 10)
- `respect_robots` (optional): Whether to respect robots.txt (default: true)

### `scrape_page`
Scrape a single web page and return its content.

**Parameters:**
- `url` (required): The URL to scrape

### `get_links`
Extract all links from a website.

**Parameters:**
- `url` (required): The URL to extract links from
- `max_depth` (optional): Maximum depth to crawl for links (default: 1)

## Installation

### Prerequisites
- Rust 1.70+ 
- Cargo

### Building from Source

```bash
git clone <repository-url>
cd mcp-spider
cargo build --release
```

### Running the Server

```bash
cargo run
```

## Usage

The server communicates over stdio using the Model Context Protocol. It can be integrated with any MCP-compatible client.

### Example Tool Calls

#### Crawl a website:
```json
{
  "name": "crawl",
  "arguments": {
    "url": "https://example.com",
    "max_pages": 5,
    "respect_robots": true
  }
}
```

#### Scrape a single page:
```json
{
  "name": "scrape_page", 
  "arguments": {
    "url": "https://example.com/page"
  }
}
```

#### Extract links:
```json
{
  "name": "get_links",
  "arguments": {
    "url": "https://example.com",
    "max_depth": 2
  }
}
```

## Configuration

The server accepts these command-line options:

- `--debug`: Enable debug logging
- `--user-agent`: Set custom user agent string
- `--delay`: Default delay between requests in seconds
- `--max-depth`: Maximum crawl depth
- `--subdomains`: Enable subdomain crawling
- `--respect-robots`: Respect robots.txt (default: true)

## Dependencies

- **spider**: Web crawling and scraping engine
- **async-mcp**: Model Context Protocol implementation
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **anyhow**: Error handling
- **tracing**: Logging

## Implementation Notes

This implementation focuses on:

1. **Simplicity**: Clean, straightforward spider-rs usage
2. **Reliability**: Proper error handling and async patterns
3. **Performance**: Leveraging spider-rs's concurrent architecture
4. **Standards Compliance**: Following spider-rs best practices

The server removes complex custom scraping logic and instead relies on spider-rs's proven, well-tested functionality.

## Future Enhancements

Potential additions (as requested):
- CSS selector-based element extraction
- Chrome/headless browser support
- Screenshot capabilities
- Form interaction
- Advanced filtering and processing

## License

MIT License