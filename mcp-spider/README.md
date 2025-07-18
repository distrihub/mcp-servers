# MCP Spider Server

An advanced Model Context Protocol (MCP) server for web crawling and scraping using spider-rs with comprehensive Chrome automation, headless browsing, and web automation capabilities.

## Features

### Core Scraping Tools
- **Spider-powered crawling**: Built entirely with spider-rs for maximum performance
- **Chrome headless support**: Use real Chrome browser for JavaScript-heavy sites
- **Stealth mode**: Avoid bot detection with stealth browsing capabilities
- **CSS Selector Support**: Use powerful CSS selectors for element extraction
- **XPath Alternatives**: Convert common XPath expressions to CSS selectors
- **Element Extraction**: Extract elements, text, attributes, and metadata
- **Form Analysis**: Analyze and extract form fields and structures
- **Table Extraction**: Extract structured table data with headers
- **Link & Image Extraction**: Comprehensive link and image discovery
- **Pattern Matching**: Search content using regular expressions
- **Structured Data**: Extract JSON-LD and microdata

### Advanced Chrome Features
- **Headless & Headed modes**: Choose between invisible or visible browser operation
- **Screenshot capture**: Take full-page or element-specific screenshots
- **JavaScript execution**: Run custom JavaScript on pages
- **Element waiting**: Wait for dynamic content to load
- **Network interception**: Monitor and manipulate network requests
- **Cookie management**: Handle authentication and session state
- **User agent spoofing**: Customize browser identification

### Web Automation
- **Element clicking**: Click buttons, links, and interactive elements
- **Form filling**: Automatically fill form fields
- **Form submission**: Submit forms programmatically
- **Dynamic content handling**: Work with AJAX and SPA applications
- **Real browser simulation**: Full browser environment for complex sites

## Available Tools

### Basic Scraping Tools

#### 1. `scrape`
Basic webpage scraping using spider.

```json
{
  "url": "https://example.com"
}
```

#### 2. `chrome_scrape`
Advanced scraping using Chrome headless browser.

```json
{
  "url": "https://example.com",
  "stealth_mode": true,
  "take_screenshot": false,
  "wait_for_selector": ".dynamic-content",
  "timeout_seconds": 30
}
```

#### 3. `advanced_scrape`
Comprehensive scraping with all extraction options.

```json
{
  "url": "https://example.com",
  "use_chrome": true,
  "stealth_mode": true,
  "take_screenshot": true,
  "include_links": true,
  "include_images": true,
  "include_forms": true,
  "include_tables": true,
  "include_metadata": true,
  "include_structured_data": true
}
```

### Element Extraction Tools

#### 4. `select_elements`
Select elements using CSS selectors.

```json
{
  "url": "https://example.com",
  "selector": "div.content p",
  "use_chrome": false
}
```

#### 5. `extract_text`
Extract text content from elements.

```json
{
  "url": "https://example.com",
  "selector": "h1, h2, h3",
  "use_chrome": true
}
```

#### 6. `extract_attributes`
Extract specific attribute values.

```json
{
  "url": "https://example.com",
  "selector": "a",
  "attribute": "href",
  "use_chrome": false
}
```

#### 7. `extract_links`
Extract all links from a webpage.

```json
{
  "url": "https://example.com",
  "use_chrome": true
}
```

#### 8. `extract_images`
Extract all images with their attributes.

```json
{
  "url": "https://example.com",
  "use_chrome": false
}
```

#### 9. `extract_forms`
Extract form structures and field information.

```json
{
  "url": "https://example.com",
  "use_chrome": true
}
```

#### 10. `extract_tables`
Extract table data with headers and rows.

```json
{
  "url": "https://example.com",
  "use_chrome": false
}
```

#### 11. `extract_metadata`
Extract page metadata including Open Graph data.

```json
{
  "url": "https://example.com",
  "use_chrome": true
}
```

#### 12. `search_patterns`
Search for text patterns using regular expressions.

```json
{
  "url": "https://example.com",
  "pattern": "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}",
  "use_chrome": false
}
```

#### 13. `extract_structured_data`
Extract JSON-LD and microdata structured information.

```json
{
  "url": "https://example.com",
  "use_chrome": true
}
```

### XPath Tools

#### 14. `xpath_to_css`
Convert XPath expressions to CSS selectors.

```json
{
  "xpath": "//div[@class='content']//p[1]",
  "show_common_patterns": true
}
```

### Web Automation Tools

#### 15. `click_element`
Click an element using Chrome automation.

```json
{
  "url": "https://example.com",
  "selector": "button.submit"
}
```

#### 16. `fill_form`
Fill form fields with data.

```json
{
  "url": "https://example.com/form",
  "form_data": {
    "username": "john_doe",
    "email": "john@example.com",
    "message": "Hello world!"
  }
}
```

#### 17. `submit_form`
Submit a form on the page.

```json
{
  "url": "https://example.com/form",
  "form_selector": "form#contact-form"
}
```

#### 18. `take_screenshot`
Take a screenshot of a page or element.

```json
{
  "url": "https://example.com",
  "selector": ".main-content"
}
```

#### 19. `wait_for_element`
Wait for an element to appear on the page.

```json
{
  "url": "https://example.com",
  "selector": ".loading-complete",
  "timeout_ms": 30000
}
```

#### 20. `execute_javascript`
Execute custom JavaScript on the page.

```json
{
  "url": "https://example.com",
  "script": "return document.title"
}
```

## Chrome Features

### Headless vs Headed Mode
- **Headless** (default): Browser runs invisibly in the background
- **Headed**: Browser window is visible for debugging
- Set via feature flags in `Cargo.toml`

### Stealth Mode
Avoids detection by:
- Spoofing user agent strings
- Hiding automation indicators
- Simulating human-like behavior
- Randomizing request timings

### Screenshot Capabilities
- Full page screenshots
- Element-specific screenshots
- Base64 encoded image data
- Support for both PNG and JPEG formats

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

| XPath | CSS Selector |
|-------|-------------|
| `//div` | `div` |
| `//a[@href]` | `a[href]` |
| `//input[@type='text']` | `input[type='text']` |
| `//div[@id='content']` | `div#content` |
| `//span[contains(@class, 'highlight')]` | `span[class*='highlight']` |
| `//p[1]` | `p:first-child` |
| `//li[last()]` | `li:last-child` |
| `/html/body/div` | `html > body > div` |

## Usage Examples

### Scrape JavaScript-Heavy Site
```json
{
  "url": "https://spa-app.example.com",
  "use_chrome": true,
  "stealth_mode": true,
  "wait_for_selector": ".content-loaded"
}
```

### Automate Login Form
```json
{
  "url": "https://example.com/login",
  "form_data": {
    "username": "user@example.com",
    "password": "secret123"
  }
}
```

### Take Screenshot After Interaction
```json
{
  "url": "https://example.com",
  "selector": "button.open-modal"
}
```
Then:
```json
{
  "url": "https://example.com",
  "selector": ".modal-content"
}
```

### Extract Dynamic Content
```json
{
  "url": "https://dynamic-site.example.com",
  "use_chrome": true,
  "wait_for_selector": ".ajax-loaded-content",
  "timeout_seconds": 60
}
```

### Execute Custom JavaScript
```json
{
  "url": "https://example.com",
  "script": "return Array.from(document.querySelectorAll('.product')).map(el => el.textContent)"
}
```

## Installation

1. Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "spider": {
      "command": "mcp-spider",
      "args": ["--debug"]
    }
  }
}
```

2. Build the server:

```bash
cargo build --release --features chrome
```

## Feature Flags

Enable specific features in `Cargo.toml`:

```toml
default = ["chrome"]
headless = ["spider/chrome"]
headed = ["spider/chrome_headed"]
stealth = ["spider/chrome_stealth"]
screenshots = ["spider/chrome_screenshot"]
```

## Command Line Options

- `--debug`: Enable debug logging
- `--user-agent <string>`: Set user agent string
- `--delay <seconds>`: Default delay between requests
- `--max-depth <number>`: Maximum crawl depth
- `--subdomains`: Enable subdomain crawling
- `--respect-robots <bool>`: Respect robots.txt

## Environment Variables

- `CHROME_URL`: Connect to remote Chrome instance
- `SCREENSHOT_DIRECTORY`: Directory for screenshot storage

## Error Handling

All tools include comprehensive error handling:

- Invalid CSS selectors return detailed error messages
- Network failures are gracefully handled
- Chrome connection issues are detected and reported
- JavaScript execution errors are captured
- Timeout and rate limiting protection

## Performance Considerations

- Chrome instances are reused efficiently
- Network request interception for faster loading
- Memory-efficient HTML parsing with spider-rs
- Configurable delays to respect server limits
- Smart mode: HTTP first, Chrome only when needed

## Architecture

The server is built entirely with spider-rs:
- **No scraper crate dependency**: Pure spider implementation
- **Chrome integration**: Native chrome headless support
- **Async/await**: Full async operation with tokio
- **Memory efficient**: Optimized for large-scale scraping
- **Type safe**: Rust's safety guarantees throughout

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

- E-commerce product scraping with Chrome
- News article extraction from SPAs
- Social media data collection with stealth mode
- Form automation workflows
- Table data extraction from dynamic tables
- SEO metadata analysis with JavaScript support
- Screenshot-based monitoring
- Complex multi-step automation scenarios