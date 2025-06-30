use anyhow::{anyhow, Result};
use spider::website::Website;
use spider::configuration::Configuration;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use url::Url;
use hashbrown::HashSet;
// use base64::{Engine as _, engine::general_purpose};

use crate::{
    ScrapeRequest, ScrapeResult, ScrapedPage, LinkInfo, ImageInfo, PageMetadata, 
    ErrorPage, ScreenshotParams
};
use crate::config::SpiderConfiguration;

pub struct SpiderScraper {
    default_config: Configuration,
}

impl SpiderScraper {
    pub fn new() -> Result<Self> {
        let mut config = Configuration::new();
        
        // Set reasonable defaults for scraping
        config.respect_robots_txt = true;
        config.delay = 1000; // 1 second delay
        config.concurrency = 5; // Lower concurrency for scraping
        config.subdomains = false;
        config.tld = false;
        config.cache = false;
        config.use_cookies = false;
        config.redirect_limit = 5;
        config.accept_invalid_certs = false;

        Ok(Self {
            default_config: config,
        })
    }

    pub async fn scrape(&self, request: ScrapeRequest) -> Result<ScrapeResult> {
        let start_time = Instant::now();
        
        // Validate URL
        let base_url = Url::parse(&request.url)
            .map_err(|e| anyhow!("Invalid URL '{}': {}", request.url, e))?;

        info!("Starting scrape of: {}", base_url);

        // Configure spider based on request
        let mut config = self.default_config.clone();
        self.apply_scrape_config(&mut config, &request)?;

        // Create and configure website
        let mut website = Website::new(&request.url);
        website.configure(config);

        // Add custom headers if provided
        if let Some(headers) = &request.headers {
            for (key, value) in headers {
                website.with_header(key, value);
            }
        }

        // Set user agent if provided
        if let Some(user_agent) = &request.user_agent {
            website.with_user_agent(user_agent);
        }

        // Set up blacklist patterns
        if let Some(blacklist) = &request.blacklist {
            for pattern in blacklist {
                website.with_blacklist_url(vec![pattern.clone()]);
            }
        }

        // Start crawling/scraping
        info!("Scraping with depth: {:?}, extract_text: {}, extract_links: {}, extract_images: {}", 
               request.depth.unwrap_or(1),
               request.extract_text.unwrap_or(true),
               request.extract_links.unwrap_or(true),
               request.extract_images.unwrap_or(true));

        website.crawl().await;

        // Process results
        let pages = website.get_pages();
        let mut scraped_pages = Vec::new();
        let mut pages_crawled = 0u32;
        let mut pages_failed = 0u32;
        let mut error_pages = Vec::new();

        for page in pages {
            let page_start = Instant::now();
            
            match self.process_page(page, &request).await {
                Ok(scraped_page) => {
                    if scraped_page.error.is_some() {
                        pages_failed += 1;
                        if let Some(status) = scraped_page.status_code {
                            error_pages.push(ErrorPage {
                                url: scraped_page.url.clone(),
                                error: scraped_page.error.clone().unwrap_or_default(),
                                status_code: Some(status),
                            });
                        }
                    } else {
                        pages_crawled += 1;
                    }
                    scraped_pages.push(scraped_page);
                }
                Err(e) => {
                    pages_failed += 1;
                    error_pages.push(ErrorPage {
                        url: page.get_url().to_string(),
                        error: e.to_string(),
                        status_code: page.get_status_code(),
                    });
                    warn!("Failed to process page {}: {}", page.get_url(), e);
                }
            }
        }

        // Get sitemap URLs if requested
        let sitemap_urls = if request.include_sitemap.unwrap_or(true) {
            Some(self.extract_sitemap_urls(&website).await?)
        } else {
            None
        };

        let duration = start_time.elapsed();
        
        info!("Scrape completed: {} pages processed, {} succeeded, {} failed in {:?}",
              scraped_pages.len(), pages_crawled, pages_failed, duration);

        Ok(ScrapeResult {
            pages: scraped_pages,
            pages_crawled,
            pages_failed,
            duration_ms: duration.as_millis() as u64,
            sitemap_urls,
            error_pages: if error_pages.is_empty() { None } else { Some(error_pages) },
        })
    }

    async fn process_page(&self, page: &spider::page::Page, request: &ScrapeRequest) -> Result<ScrapedPage> {
        let page_start = Instant::now();
        let url = page.get_url().to_string();
        let html_content = page.get_html();
        let status_code = page.get_status_code();
        
        debug!("Processing page: {}", url);

        // Check for error status
        let error = if let Some(code) = status_code {
            if code >= 400 {
                Some(format!("HTTP {}", code))
            } else {
                None
            }
        } else {
            None
        };

        // Parse HTML if available and no error
        let (title, text_content, links, images, metadata) = if !html_content.is_empty() && error.is_none() {
            let document = Html::parse_document(html_content);
            
            let title = if request.extract_metadata.unwrap_or(true) {
                self.extract_title(&document)
            } else {
                None
            };

            let text_content = if request.extract_text.unwrap_or(true) {
                Some(self.extract_text_content(&document))
            } else {
                None
            };

            let links = if request.extract_links.unwrap_or(true) {
                Some(self.extract_links(&document, &url)?)
            } else {
                None
            };

            let images = if request.extract_images.unwrap_or(true) {
                Some(self.extract_images(&document, &url)?)
            } else {
                None
            };

            let metadata = if request.extract_metadata.unwrap_or(true) {
                Some(self.extract_metadata(&document))
            } else {
                None
            };

            (title, text_content, links, images, metadata)
        } else {
            (None, None, None, None, None)
        };

        // Extract headers if available
        let headers = self.extract_headers(page);

        // Handle screenshots if requested
        let (screenshot_path, screenshot_base64) = if request.take_screenshots.unwrap_or(false) {
            self.take_screenshot(&url, request.screenshot_params.as_ref()).await?
        } else {
            (None, None)
        };

        let duration = page_start.elapsed();

        Ok(ScrapedPage {
            url,
            status_code,
            title,
            content: if html_content.is_empty() { None } else { Some(html_content.to_string()) },
            text_content,
            links,
            images,
            metadata,
            headers,
            screenshot_path,
            screenshot_base64,
            bytes: Some(html_content.len()),
            duration_ms: Some(duration.as_millis() as u64),
            redirect_count: None, // spider-rs doesn't directly expose this
            error,
        })
    }

    fn extract_title(&self, document: &Html) -> Option<String> {
        let title_selector = Selector::parse("title").ok()?;
        document
            .select(&title_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_text_content(&self, document: &Html) -> String {
        // Remove script and style elements, then extract text
        let mut text_parts = Vec::new();
        
        // Extract text from common content elements
        let content_selectors = [
            "p", "h1", "h2", "h3", "h4", "h5", "h6", 
            "article", "section", "main", "div", "span",
            "li", "td", "th"
        ];

        for selector_str in &content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<String>();
                    let cleaned = text.trim();
                    if !cleaned.is_empty() && cleaned.len() > 10 {
                        text_parts.push(cleaned.to_string());
                    }
                }
            }
        }

        text_parts.join(" ").trim().to_string()
    }

    fn extract_links(&self, document: &Html, base_url: &str) -> Result<Vec<LinkInfo>> {
        let link_selector = Selector::parse("a[href]")
            .map_err(|e| anyhow!("Failed to parse link selector: {}", e))?;
        
        let base = Url::parse(base_url)?;
        let mut links = Vec::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base.join(href) {
                    let link_info = LinkInfo {
                        url: absolute_url.to_string(),
                        text: Some(element.text().collect::<String>().trim().to_string()),
                        title: element.value().attr("title").map(|s| s.to_string()),
                        rel: element.value().attr("rel").map(|s| s.to_string()),
                    };
                    links.push(link_info);
                }
            }
        }

        Ok(links)
    }

    fn extract_images(&self, document: &Html, base_url: &str) -> Result<Vec<ImageInfo>> {
        let img_selector = Selector::parse("img[src]")
            .map_err(|e| anyhow!("Failed to parse image selector: {}", e))?;
        
        let base = Url::parse(base_url)?;
        let mut images = Vec::new();

        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(absolute_url) = base.join(src) {
                    let image_info = ImageInfo {
                        url: absolute_url.to_string(),
                        alt: element.value().attr("alt").map(|s| s.to_string()),
                        title: element.value().attr("title").map(|s| s.to_string()),
                        width: element.value().attr("width")
                            .and_then(|w| w.parse().ok()),
                        height: element.value().attr("height")
                            .and_then(|h| h.parse().ok()),
                    };
                    images.push(image_info);
                }
            }
        }

        Ok(images)
    }

    fn extract_metadata(&self, document: &Html) -> PageMetadata {
        let meta_selector = Selector::parse("meta").unwrap();
        let mut metadata = PageMetadata {
            description: None,
            keywords: None,
            author: None,
            canonical_url: None,
            language: None,
            charset: None,
            og_title: None,
            og_description: None,
            og_image: None,
            twitter_card: None,
            twitter_title: None,
            twitter_description: None,
        };

        for element in document.select(&meta_selector) {
            let attrs = element.value();
            
            if let Some(name) = attrs.attr("name") {
                let content = attrs.attr("content").unwrap_or_default();
                match name.to_lowercase().as_str() {
                    "description" => metadata.description = Some(content.to_string()),
                    "keywords" => metadata.keywords = Some(
                        content.split(',').map(|s| s.trim().to_string()).collect()
                    ),
                    "author" => metadata.author = Some(content.to_string()),
                    _ => {}
                }
            }

            if let Some(property) = attrs.attr("property") {
                let content = attrs.attr("content").unwrap_or_default();
                match property.to_lowercase().as_str() {
                    "og:title" => metadata.og_title = Some(content.to_string()),
                    "og:description" => metadata.og_description = Some(content.to_string()),
                    "og:image" => metadata.og_image = Some(content.to_string()),
                    _ => {}
                }
            }

            if let Some(name) = attrs.attr("name") {
                let content = attrs.attr("content").unwrap_or_default();
                if name.starts_with("twitter:") {
                    match name.to_lowercase().as_str() {
                        "twitter:card" => metadata.twitter_card = Some(content.to_string()),
                        "twitter:title" => metadata.twitter_title = Some(content.to_string()),
                        "twitter:description" => metadata.twitter_description = Some(content.to_string()),
                        _ => {}
                    }
                }
            }
        }

        // Extract canonical URL
        if let Ok(link_selector) = Selector::parse("link[rel=\"canonical\"][href]") {
            if let Some(element) = document.select(&link_selector).next() {
                metadata.canonical_url = element.value().attr("href").map(|s| s.to_string());
            }
        }

        // Extract language from html tag
        if let Ok(html_selector) = Selector::parse("html[lang]") {
            if let Some(element) = document.select(&html_selector).next() {
                metadata.language = element.value().attr("lang").map(|s| s.to_string());
            }
        }

        metadata
    }

    fn extract_headers(&self, page: &spider::page::Page) -> Option<HashMap<String, String>> {
        // spider-rs doesn't directly expose headers in the Page struct
        // This is a placeholder for when/if that functionality is available
        None
    }

    async fn take_screenshot(&self, url: &str, params: Option<&ScreenshotParams>) -> Result<(Option<String>, Option<String>)> {
        // Screenshot functionality would require Chrome integration
        // This is a placeholder implementation
        debug!("Screenshot requested for: {}", url);
        
        if let Some(_params) = params {
            // In a real implementation, this would:
            // 1. Launch Chrome/Chromium with spider-rs chrome features
            // 2. Navigate to the URL
            // 3. Take the screenshot with specified parameters
            // 4. Return the path and/or base64 encoded image
            
            // For now, return None to indicate screenshots aren't implemented yet
            warn!("Screenshot functionality not yet implemented");
        }

        Ok((None, None))
    }

    fn apply_scrape_config(&self, config: &mut Configuration, request: &ScrapeRequest) -> Result<()> {
        if let Some(respect_robots) = request.respect_robots_txt {
            config.respect_robots_txt = respect_robots;
        }

        if let Some(accept_invalid_certs) = request.accept_invalid_certs {
            config.accept_invalid_certs = accept_invalid_certs;
        }

        if let Some(subdomains) = request.subdomains {
            config.subdomains = subdomains;
        }

        if let Some(tld) = request.tld {
            config.tld = tld;
        }

        if let Some(delay) = request.delay {
            config.delay = (delay * 1000.0) as u64; // Convert to milliseconds
        }

        if let Some(cache) = request.cache {
            config.cache = cache;
        }

        if let Some(cookies) = request.use_cookies {
            config.use_cookies = cookies;
        }

        if let Some(max_redirects) = request.max_redirects {
            config.redirect_limit = max_redirects;
        }

        if let Some(concurrency) = request.concurrency {
            config.concurrency = concurrency as usize;
        }

        Ok(())
    }

    async fn extract_sitemap_urls(&self, website: &Website) -> Result<Vec<String>> {
        let mut sitemap_urls = Vec::new();
        
        let pages = website.get_pages();
        for page in pages {
            let url = page.get_url();
            if url.contains("sitemap") && (url.ends_with(".xml") || url.contains("sitemap")) {
                sitemap_urls.push(url.to_string());
            }
        }

        Ok(sitemap_urls)
    }
}

impl Default for SpiderScraper {
    fn default() -> Self {
        Self::new().expect("Failed to create SpiderScraper")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_creation() {
        let scraper = SpiderScraper::new();
        assert!(scraper.is_ok());
    }

    #[test]
    fn test_text_extraction() {
        let scraper = SpiderScraper::new().unwrap();
        let html = r#"
            <html>
                <head><title>Test Page</title></head>
                <body>
                    <h1>Main Title</h1>
                    <p>This is a paragraph with some content.</p>
                    <script>console.log('script');</script>
                </body>
            </html>
        "#;
        
        let document = Html::parse_document(html);
        let text = scraper.extract_text_content(&document);
        assert!(text.contains("Main Title"));
        assert!(text.contains("This is a paragraph"));
        assert!(!text.contains("console.log"));
    }

    #[test]
    fn test_metadata_extraction() {
        let scraper = SpiderScraper::new().unwrap();
        let html = r#"
            <html lang="en">
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="A test page description">
                    <meta name="keywords" content="test, page, example">
                    <meta property="og:title" content="OG Test Page">
                    <meta name="twitter:card" content="summary">
                    <link rel="canonical" href="https://example.com/test">
                </head>
                <body></body>
            </html>
        "#;
        
        let document = Html::parse_document(html);
        let metadata = scraper.extract_metadata(&document);
        
        assert_eq!(metadata.description, Some("A test page description".to_string()));
        assert_eq!(metadata.keywords, Some(vec!["test".to_string(), "page".to_string(), "example".to_string()]));
        assert_eq!(metadata.og_title, Some("OG Test Page".to_string()));
        assert_eq!(metadata.twitter_card, Some("summary".to_string()));
        assert_eq!(metadata.canonical_url, Some("https://example.com/test".to_string()));
    }
}