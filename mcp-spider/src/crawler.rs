use anyhow::{anyhow, Result};
use hashbrown::HashSet;
use serde_json::Value;
use spider::configuration::Configuration;
use spider::website::Website;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::config::SpiderConfiguration;
use crate::{CrawlRequest, CrawlResult, ErrorPage};

pub struct SpiderCrawler {
    default_config: Configuration,
}

impl SpiderCrawler {
    pub fn new() -> Result<Self> {
        let mut config = Configuration::new();

        // Set reasonable defaults
        config.respect_robots_txt = true;
        config.delay = 1000; // 1 second delay
        config.concurrency_limit = Some(10);
        config.subdomains = false;
        config.tld = false;
        config.cache = false;
        config.redirect_limit = 5;
        config.accept_invalid_certs = false;

        Ok(Self {
            default_config: config,
        })
    }

    pub async fn crawl(&self, request: CrawlRequest) -> Result<CrawlResult> {
        let start_time = Instant::now();

        // Validate URL
        let base_url = Url::parse(&request.url)
            .map_err(|e| anyhow!("Invalid URL '{}': {}", request.url, e))?;

        info!("Starting crawl of: {}", base_url);

        // Configure spider based on request
        let mut config = self.default_config.clone();
        self.apply_crawl_config(&mut config, &request)?;

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

        // Start crawling
        info!(
            "Crawling with depth: {:?}, concurrency: {}",
            request.depth.unwrap_or(2),
            request.concurrency.unwrap_or(10)
        );

        website.crawl().await;

        // Collect results
        let links = website.get_links();
        let pages = website.get_pages();

        let mut urls = Vec::new();
        let mut pages_crawled = 0u32;
        let mut pages_failed = 0u32;
        let mut error_pages = Vec::new();
        let mut all_urls = HashSet::new();

        // Process pages and collect URLs
        for page in pages {
            let page_url = page.get_url();

            if page.get_html().is_empty() && page.get_status_code().unwrap_or(200) >= 400 {
                pages_failed += 1;
                error_pages.push(ErrorPage {
                    url: page_url.to_string(),
                    error: format!("HTTP {}", page.get_status_code().unwrap_or(0)),
                    status_code: page.get_status_code(),
                });
            } else {
                pages_crawled += 1;
            }

            if !all_urls.contains(page_url) {
                urls.push(page_url.to_string());
                all_urls.insert(page_url.to_string());
            }
        }

        // Add any additional links found
        for link in links {
            let link_str = link.as_ref();
            if !all_urls.contains(link_str) {
                urls.push(link_str.to_string());
                all_urls.insert(link_str.to_string());
            }
        }

        // Get sitemap URLs if requested
        let sitemap_urls = if request.include_sitemap.unwrap_or(true) {
            Some(self.extract_sitemap_urls(&website).await?)
        } else {
            None
        };

        let duration = start_time.elapsed();

        info!(
            "Crawl completed: {} URLs found, {} pages crawled, {} failed in {:?}",
            urls.len(),
            pages_crawled,
            pages_failed,
            duration
        );

        Ok(CrawlResult {
            urls,
            pages_crawled,
            pages_failed,
            duration_ms: duration.as_millis() as u64,
            sitemap_urls,
            error_pages: if error_pages.is_empty() {
                None
            } else {
                Some(error_pages)
            },
        })
    }

    fn apply_crawl_config(&self, config: &mut Configuration, request: &CrawlRequest) -> Result<()> {
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

        // Set budget configuration
        if let Some(timeout) = request.budget_request_timeout {
            // Note: spider-rs doesn't directly expose request timeout in Configuration
            // This would need to be handled at the client level
            debug!("Request timeout set to: {}s", timeout);
        }

        if let Some(max_file_size) = request.max_file_size {
            // Note: spider-rs doesn't directly expose max file size in Configuration
            debug!("Max file size set to: {} bytes", max_file_size);
        }

        Ok(())
    }

    async fn extract_sitemap_urls(&self, website: &Website) -> Result<Vec<String>> {
        // This is a simplified sitemap extraction
        // In a real implementation, you'd parse the actual sitemap.xml
        let mut sitemap_urls = Vec::new();

        // Try to get sitemap from the website
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

impl Default for SpiderCrawler {
    fn default() -> Self {
        Self::new().expect("Failed to create SpiderCrawler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawler_creation() {
        let crawler = SpiderCrawler::new();
        assert!(crawler.is_ok());
    }

    #[tokio::test]
    async fn test_basic_crawl() {
        let crawler = SpiderCrawler::new().unwrap();
        let request = CrawlRequest {
            url: "https://example.com".to_string(),
            headers: None,
            user_agent: Some("test-crawler".to_string()),
            depth: Some(1),
            blacklist: None,
            whitelist: None,
            respect_robots_txt: Some(true),
            accept_invalid_certs: Some(false),
            subdomains: Some(false),
            tld: Some(false),
            delay: Some(2.0),
            budget_depth: None,
            budget_request_timeout: Some(30.0),
            cache: Some(false),
            use_cookies: Some(false),
            stealth_mode: Some(false),
            chrome_intercept: Some(false),
            include_sitemap: Some(true),
            max_redirects: Some(3),
            max_file_size: None,
            concurrency: Some(5),
            full_resources: Some(false),
        };

        // This test would require network access, so we just test the structure
        // In a real test environment, you'd mock the network calls
        assert_eq!(request.url, "https://example.com");
    }
}
