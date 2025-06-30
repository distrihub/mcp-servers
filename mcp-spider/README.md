# MCP Spider - Advanced Web Crawling & Scraping MCP Server

A comprehensive Model Context Protocol (MCP) server for web crawling and scraping built with Rust and the powerful [spider-rs](https://github.com/spider-rs/spider) library.

## 🚀 Features

### Core Capabilities
- **Fast Web Crawling**: Multi-threaded crawling with configurable concurrency
- **Content Extraction**: Extract text, links, images, and metadata
- **Comprehensive Scraping**: Full page content with structured data extraction
- **Robots.txt Respect**: Ethical crawling with robots.txt compliance
- **Rate Limiting**: Polite crawling with configurable delays
- **URL Filtering**: Blacklist/whitelist support with regex patterns

### Advanced Features
- **Chrome Integration**: JavaScript rendering and screenshot capture
- **Stealth Mode**: Anti-detection crawling capabilities
- **Caching**: HTTP response caching for efficiency
- **Proxy Support**: HTTP/HTTPS/SOCKS5 proxy configuration
- **Subdomain Handling**: Configurable subdomain and TLD crawling
- **Custom Headers**: Full control over HTTP headers and user agents
- **Sitemap Integration**: Automatic sitemap.xml parsing and inclusion

### Content Extraction
- **Text Content**: Clean text extraction from HTML
- **Links**: All links with metadata (title, rel, etc.)
- **Images**: Image URLs with alt text and dimensions
- **Metadata**: Open Graph, Twitter Cards, schema.org data
- **Social Links**: Automatic social media link detection
- **Email/Phone**: Extract contact information from pages

## 🛠 Installation

### From Source

```bash
git clone <repository-url>
cd mcp-servers/mcp-spider
cargo build --release
```

### Using Cargo

```bash
cargo install --path mcp-spider
```

## 📋 Usage

### Starting the MCP Server

```bash
# Start with default settings (STDIO transport)
./target/release/mcp-spider

# Start with debug logging
./target/release/mcp-spider --debug

# Generate example configuration
./target/release/mcp-spider generate-config
```

### Command Line Options

```bash
# Show all capabilities and configuration
./target/release/mcp-spider info

# Test crawling functionality
./target/release/mcp-spider test --url https://example.com

# Test scraping functionality  
./target/release/mcp-spider test --url https://example.com --scrape

# Custom configuration
./target/release/mcp-spider --max-concurrency 20 --default-delay 0.5 --stealth-mode
```

## 🔧 MCP Tools

### `crawl` - Website Crawling

Crawls websites and returns discovered URLs with comprehensive configuration options.

**Parameters:**
- `url` (required): Starting URL to crawl
- `depth`: Maximum crawl depth (default: 2)
- `concurrency`: Number of concurrent requests (default: 10)
- `delay`: Delay between requests in seconds (default: 1.0)
- `respect_robots_txt`: Follow robots.txt rules (default: true)
- `subdomains`: Allow subdomain crawling (default: false)
- `blacklist`: Array of regex patterns to exclude URLs
- `whitelist`: Array of regex patterns to include URLs
- `headers`: Custom HTTP headers object
- `user_agent`: Custom User-Agent string
- `cache`: Enable HTTP caching (default: false)
- `stealth_mode`: Enable stealth crawling (default: false)
- `max_redirects`: Maximum redirects to follow (default: 5)
- `max_file_size`: Maximum file size in bytes
- `include_sitemap`: Include sitemap.xml URLs (default: true)

**Example:**
```json
{
  "url": "https://example.com",
  "depth": 3,
  "concurrency": 15,
  "delay": 2.0,
  "blacklist": [".*\\.pdf$", ".*/admin/.*"],
  "headers": {
    "Accept-Language": "en-US,en;q=0.9"
  }
}
```

### `scrape` - Content Extraction

Scrapes websites and extracts structured content including text, links, images, and metadata.

**Parameters:**
All `crawl` parameters plus:
- `extract_text`: Extract text content (default: true)
- `extract_links`: Extract all links (default: true) 
- `extract_images`: Extract image information (default: true)
- `extract_metadata`: Extract page metadata (default: true)
- `take_screenshots`: Capture page screenshots (default: false)
- `screenshot_params`: Screenshot configuration object

**Screenshot Parameters:**
- `full_page`: Capture full page (default: true)
- `quality`: Image quality 1-100 (default: 90)
- `format`: Image format "png" or "jpeg" (default: "png")
- `viewport_width`: Viewport width (default: 1920)
- `viewport_height`: Viewport height (default: 1080)

**Example:**
```json
{
  "url": "https://news.example.com",
  "depth": 2,
  "extract_text": true,
  "extract_links": true,
  "extract_metadata": true,
  "take_screenshots": true,
  "screenshot_params": {
    "full_page": true,
    "quality": 95,
    "format": "png"
  }
}
```

## 📊 Output Formats

### Crawl Results
```json
{
  "urls": ["https://example.com", "https://example.com/page1"],
  "pages_crawled": 25,
  "pages_failed": 2,
  "duration_ms": 5420,
  "sitemap_urls": ["https://example.com/sitemap.xml"],
  "error_pages": [
    {
      "url": "https://example.com/broken",
      "error": "HTTP 404",
      "status_code": 404
    }
  ]
}
```

### Scrape Results
```json
{
  "pages": [
    {
      "url": "https://example.com",
      "status_code": 200,
      "title": "Example Website",
      "content": "<html>...</html>",
      "text_content": "Welcome to our website...",
      "links": [
        {
          "url": "https://example.com/about",
          "text": "About Us",
          "title": "Learn about our company",
          "rel": null
        }
      ],
      "images": [
        {
          "url": "https://example.com/logo.png",
          "alt": "Company Logo",
          "width": 200,
          "height": 100
        }
      ],
      "metadata": {
        "description": "Example website description",
        "keywords": ["example", "website"],
        "og_title": "Example Website",
        "og_image": "https://example.com/og-image.png"
      },
      "screenshot_base64": "iVBORw0KGgoAAAANSUhEUgAA...",
      "duration_ms": 1250
    }
  ],
  "pages_crawled": 15,
  "pages_failed": 1,
  "duration_ms": 12300
}
```

## ⚙️ Configuration

### Environment Variables

- `CHROME_PATH`: Path to Chrome/Chromium executable
- `SCREENSHOT_DIRECTORY`: Directory to save screenshots

### Configuration Presets

The server includes several built-in configuration presets:

- **Fast Crawl**: High concurrency, minimal delays
- **Polite Crawl**: Low concurrency, respectful delays
- **Comprehensive Scrape**: Full content extraction with screenshots
- **Stealth Crawl**: Anti-detection mode with randomization

## 🚀 Performance

The spider-rs library provides excellent performance characteristics:

- **Concurrent**: Multi-threaded crawling with configurable workers
- **Memory Efficient**: Streaming processing with minimal memory usage
- **Fast**: Can crawl millions of pages with proper configuration
- **Scalable**: Horizontal scaling support with distributed workers

## 🔒 Ethical Considerations

This tool is designed for ethical web crawling:

- **Robots.txt Compliance**: Automatic robots.txt parsing and respect
- **Rate Limiting**: Built-in delays to avoid overwhelming servers
- **User Agent**: Proper identification in requests
- **Resource Limits**: Configurable limits to prevent abuse

## 🐛 Troubleshooting

### Common Issues

1. **Chrome not found**: Set `CHROME_PATH` environment variable
2. **SSL errors**: Use `accept_invalid_certs: true` for testing
3. **Rate limiting**: Increase delay or reduce concurrency
4. **Memory usage**: Enable caching or reduce concurrent requests

### Debug Mode

Enable debug logging for detailed information:

```bash
./target/release/mcp-spider --debug test --url https://example.com
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🔗 Links

- [spider-rs GitHub](https://github.com/spider-rs/spider)
- [Model Context Protocol](https://github.com/modelcontextprotocol)
- [MCP Specification](https://spec.modelcontextprotocol.io/)

## 📝 Changelog

### v1.0.0
- Initial release with spider-rs integration
- Full MCP compatibility
- Comprehensive crawling and scraping features
- Chrome integration for screenshots
- Advanced configuration options