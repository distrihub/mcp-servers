# MCP Spider Server

An advanced Model Context Protocol (MCP) server for web crawling and scraping using spider-rs with comprehensive programmatic scraping capabilities.

## Features

### Core Scraping Tools
- **CSS Selector Support**: Use powerful CSS selectors for element extraction
- **XPath Alternatives**: Convert common XPath expressions to CSS selectors
- **Element Extraction**: Extract elements, text, attributes, and metadata
- **Form Analysis**: Analyze and extract form fields and structures
- **Table Extraction**: Extract structured table data with headers
- **Link & Image Extraction**: Comprehensive link and image discovery
- **Pattern Matching**: Search content using regular expressions
- **Structured Data**: Extract JSON-LD and microdata

### Advanced Features
- **Session Management**: Maintain cookies and session state
- **Metadata Extraction**: Extract page titles, descriptions, and Open Graph data
- **Comprehensive Scraping**: One-stop tool for complete page analysis
- **Error Handling**: Robust error handling and reporting

## Available Tools

### 1. `scrape`
Basic webpage scraping with content extraction using readability.

```json
{
  "url": "https://example.com"
}
```

### 2. `select_elements`
Select elements using CSS selectors.

```json
{
  "url": "https://example.com",
  "selector": "div.content p"
}
```

### 3. `extract_text`
Extract text content from elements.

```json
{
  "url": "https://example.com",
  "selector": "h1, h2, h3"
}
```

### 4. `extract_attributes`
Extract specific attribute values.

```json
{
  "url": "https://example.com",
  "selector": "a",
  "attribute": "href"
}
```

### 5. `extract_links`
Extract all links from a webpage.

```json
{
  "url": "https://example.com"
}
```

### 6. `extract_images`
Extract all images with their attributes.

```json
{
  "url": "https://example.com"
}
```

### 7. `extract_forms`
Extract form structures and field information.

```json
{
  "url": "https://example.com"
}
```

### 8. `extract_tables`
Extract table data with headers and rows.

```json
{
  "url": "https://example.com"
}
```

### 9. `extract_metadata`
Extract page metadata including Open Graph data.

```json
{
  "url": "https://example.com"
}
```

### 10. `search_patterns`
Search for text patterns using regular expressions.

```json
{
  "url": "https://example.com",
  "pattern": "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"
}
```

### 11. `extract_structured_data`
Extract JSON-LD and microdata structured information.

```json
{
  "url": "https://example.com"
}
```

### 12. `xpath_to_css`
Convert XPath expressions to CSS selectors.

```json
{
  "xpath": "//div[@class='content']//p[1]",
  "show_common_patterns": true
}
```

### 13. `advanced_scrape`
Comprehensive scraping with customizable data extraction.

```json
{
  "url": "https://example.com",
  "include_links": true,
  "include_images": true,
  "include_forms": true,
  "include_tables": true,
  "include_metadata": true,
  "include_structured_data": true
}
```

## CSS Selector Examples

The server supports full CSS selector syntax:

- **Element selectors**: `div`, `p`, `span`
- **Class selectors**: `.classname`, `div.content`
- **ID selectors**: `#id`, `div#header`
- **Attribute selectors**: `[href]`, `[type="text"]`, `[class*="partial"]`
- **Pseudo-selectors**: `:first-child`, `:last-child`, `:nth-child(n)`
- **Descendant selectors**: `div p`, `nav > ul > li`
- **Multiple selectors**: `h1, h2, h3`

## XPath to CSS Conversion

Common XPath patterns and their CSS equivalents:

| XPath                                   | CSS Selector               |
| --------------------------------------- | -------------------------- |
| `//div`                                 | `div`                      |
| `//a[@href]`                            | `a[href]`                  |
| `//input[@type='text']`                 | `input[type='text']`       |
| `//div[@id='content']`                  | `div#content`              |
| `//span[contains(@class, 'highlight')]` | `span[class*='highlight']` |
| `//p[1]`                                | `p:first-child`            |
| `//li[last()]`                          | `li:last-child`            |
| `/html/body/div`                        | `html > body > div`        |

## Usage Examples

### Extract Article Headlines
```json
{
  "url": "https://news.ycombinator.com",
  "selector": ".titleline > a"
}
```

### Find Email Addresses
```json
{
  "url": "https://example.com/contact",
  "pattern": "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"
}
```

### Extract Product Information
```json
{
  "url": "https://shop.example.com/product/123",
  "selector": ".product-title, .price, .description"
}
```

### Analyze Form Fields
```json
{
  "url": "https://example.com/signup"
}
```

## Installation

1. Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "spider": {
      "command": "mcp-crawl",
      "args": ["--debug"]
    }
  }
}
```

2. Build the server:

```bash
cargo build --release
```

## Command Line Options

- `--debug`: Enable debug logging
- `--user-agent <string>`: Set user agent string
- `--delay <seconds>`: Default delay between requests
- `--max-depth <number>`: Maximum crawl depth
- `--subdomains`: Enable subdomain crawling
- `--respect-robots <bool>`: Respect robots.txt

## Error Handling

All tools include comprehensive error handling:

- Invalid CSS selectors return detailed error messages
- Network failures are gracefully handled
- Malformed HTML is parsed with error recovery
- Timeout and rate limiting protection

## Performance Considerations

- Session reuse for efficient multiple requests
- Cookie management for authenticated scraping
- Configurable delays to respect server limits
- Memory-efficient HTML parsing

## License

This project is licensed under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Examples Repository

For more examples and use cases, see the `examples/` directory:

- E-commerce product scraping
- News article extraction
- Social media data collection
- Form automation scripts
- Table data extraction
- SEO metadata analysis