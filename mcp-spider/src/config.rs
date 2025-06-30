use anyhow::Result;
use serde::{Deserialize, Serialize};
use spider::configuration::Configuration;
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for crawling operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlConfig {
    pub respect_robots_txt: bool,
    pub delay: Duration,
    pub concurrency: usize,
    pub subdomains: bool,
    pub tld: bool,
    pub cache: bool,
    pub use_cookies: bool,
    pub redirect_limit: u32,
    pub accept_invalid_certs: bool,
    pub user_agent: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub blacklist: Option<Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    pub budget: Option<BudgetConfig>,
    pub chrome_config: Option<ChromeConfig>,
}

/// Configuration for scraping operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeConfig {
    pub crawl_config: CrawlConfig,
    pub extract_text: bool,
    pub extract_links: bool,
    pub extract_images: bool,
    pub extract_metadata: bool,
    pub take_screenshots: bool,
    pub screenshot_config: Option<ScreenshotConfig>,
}

/// Budget configuration for controlling crawl depth and resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub max_pages: Option<u32>,
    pub max_depth: Option<u32>,
    pub max_duration: Option<Duration>,
    pub max_file_size: Option<u64>,
    pub request_timeout: Option<Duration>,
}

/// Chrome browser configuration for advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeConfig {
    pub stealth_mode: bool,
    pub intercept_requests: bool,
    pub block_ads: bool,
    pub block_images: bool,
    pub block_javascript: bool,
    pub block_css: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub user_agent: Option<String>,
    pub extra_headers: Option<HashMap<String, String>>,
}

/// Screenshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    pub full_page: bool,
    pub quality: u8,
    pub format: ImageFormat,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub delay_before_screenshot: Option<Duration>,
}

/// Image format for screenshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

/// Comprehensive spider configuration that combines all options
#[derive(Debug, Clone)]
pub struct SpiderConfiguration {
    pub base_config: Configuration,
    pub crawl_config: CrawlConfig,
    pub scrape_config: Option<ScrapeConfig>,
    pub advanced_options: AdvancedOptions,
}

/// Advanced spider options for fine-tuning behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedOptions {
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub handle_javascript: bool,
    pub extract_resources: bool,
    pub respect_meta_robots: bool,
    pub ignore_query_params: bool,
    pub normalize_urls: bool,
    pub custom_selectors: Option<HashMap<String, String>>,
    pub proxy_config: Option<ProxyConfig>,
    pub rate_limiting: Option<RateLimitConfig>,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub socks5_proxy: Option<String>,
    pub proxy_auth: Option<ProxyAuth>,
    pub no_proxy: Option<Vec<String>>,
}

/// Proxy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
    pub burst_size: u32,
    pub delay_on_error: Duration,
    pub max_retries: u32,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            respect_robots_txt: true,
            delay: Duration::from_secs(1),
            concurrency: 10,
            subdomains: false,
            tld: false,
            cache: false,
            use_cookies: false,
            redirect_limit: 5,
            accept_invalid_certs: false,
            user_agent: Some("mcp-spider/1.0".to_string()),
            headers: None,
            blacklist: None,
            whitelist: None,
            budget: None,
            chrome_config: None,
        }
    }
}

impl Default for ScrapeConfig {
    fn default() -> Self {
        Self {
            crawl_config: CrawlConfig::default(),
            extract_text: true,
            extract_links: true,
            extract_images: true,
            extract_metadata: true,
            take_screenshots: false,
            screenshot_config: None,
        }
    }
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_pages: Some(1000),
            max_depth: Some(3),
            max_duration: Some(Duration::from_secs(300)), // 5 minutes
            max_file_size: Some(10 * 1024 * 1024),        // 10MB
            request_timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl Default for ChromeConfig {
    fn default() -> Self {
        Self {
            stealth_mode: false,
            intercept_requests: false,
            block_ads: false,
            block_images: false,
            block_javascript: false,
            block_css: false,
            viewport_width: 1920,
            viewport_height: 1080,
            user_agent: None,
            extra_headers: None,
        }
    }
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            full_page: true,
            quality: 90,
            format: ImageFormat::Png,
            viewport_width: 1920,
            viewport_height: 1080,
            delay_before_screenshot: Some(Duration::from_millis(500)),
        }
    }
}

impl Default for AdvancedOptions {
    fn default() -> Self {
        Self {
            follow_redirects: true,
            max_redirects: 5,
            handle_javascript: false,
            extract_resources: false,
            respect_meta_robots: true,
            ignore_query_params: false,
            normalize_urls: true,
            custom_selectors: None,
            proxy_config: None,
            rate_limiting: None,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 2.0,
            burst_size: 10,
            delay_on_error: Duration::from_secs(5),
            max_retries: 3,
        }
    }
}

impl SpiderConfiguration {
    pub fn new() -> Self {
        Self {
            base_config: Configuration::new(),
            crawl_config: CrawlConfig::default(),
            scrape_config: None,
            advanced_options: AdvancedOptions::default(),
        }
    }

    pub fn with_crawl_config(mut self, config: CrawlConfig) -> Self {
        self.crawl_config = config;
        self
    }

    pub fn with_scrape_config(mut self, config: ScrapeConfig) -> Self {
        self.scrape_config = Some(config);
        self
    }

    pub fn with_advanced_options(mut self, options: AdvancedOptions) -> Self {
        self.advanced_options = options;
        self
    }

    pub fn apply_to_spider_config(&self) -> Result<Configuration> {
        let mut config = self.base_config.clone();

        // Apply crawl configuration
        config.respect_robots_txt = self.crawl_config.respect_robots_txt;
        config.delay = self.crawl_config.delay.as_millis() as u64;

        config.subdomains = self.crawl_config.subdomains;
        config.tld = self.crawl_config.tld;
        config.cache = self.crawl_config.cache;

        config.accept_invalid_certs = self.crawl_config.accept_invalid_certs;

        // Apply advanced options
        if !self.advanced_options.follow_redirects {
            config.redirect_limit = Box::new(0);
        }

        Ok(config)
    }

    pub fn set_budget(&mut self, budget: BudgetConfig) {
        self.crawl_config.budget = Some(budget);
    }

    pub fn set_chrome_config(&mut self, chrome: ChromeConfig) {
        self.crawl_config.chrome_config = Some(chrome);
    }

    pub fn add_header(&mut self, key: String, value: String) {
        if self.crawl_config.headers.is_none() {
            self.crawl_config.headers = Some(HashMap::new());
        }
        if let Some(ref mut headers) = self.crawl_config.headers {
            headers.insert(key, value);
        }
    }

    pub fn add_blacklist_pattern(&mut self, pattern: String) {
        if self.crawl_config.blacklist.is_none() {
            self.crawl_config.blacklist = Some(Vec::new());
        }
        if let Some(ref mut blacklist) = self.crawl_config.blacklist {
            blacklist.push(pattern);
        }
    }

    pub fn add_whitelist_pattern(&mut self, pattern: String) {
        if self.crawl_config.whitelist.is_none() {
            self.crawl_config.whitelist = Some(Vec::new());
        }
        if let Some(ref mut whitelist) = self.crawl_config.whitelist {
            whitelist.push(pattern);
        }
    }

    pub fn enable_screenshots(&mut self, config: ScreenshotConfig) {
        if self.scrape_config.is_none() {
            self.scrape_config = Some(ScrapeConfig::default());
        }
        if let Some(ref mut scrape_config) = self.scrape_config {
            scrape_config.take_screenshots = true;
            scrape_config.screenshot_config = Some(config);
        }
    }

    pub fn set_proxy(&mut self, proxy_config: ProxyConfig) {
        self.advanced_options.proxy_config = Some(proxy_config);
    }

    pub fn set_rate_limiting(&mut self, rate_config: RateLimitConfig) {
        self.advanced_options.rate_limiting = Some(rate_config);
    }
}

impl Default for SpiderConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common configurations
impl SpiderConfiguration {
    /// Create a configuration optimized for fast crawling
    pub fn fast_crawl() -> Self {
        let mut config = Self::new();
        config.crawl_config.concurrency = 20;
        config.crawl_config.delay = Duration::from_millis(250);
        config.crawl_config.cache = true;
        config.advanced_options.extract_resources = false;
        config
    }

    /// Create a configuration optimized for polite crawling
    pub fn polite_crawl() -> Self {
        let mut config = Self::new();
        config.crawl_config.concurrency = 2;
        config.crawl_config.delay = Duration::from_secs(2);
        config.crawl_config.respect_robots_txt = true;
        config.advanced_options.respect_meta_robots = true;
        config
    }

    /// Create a configuration for comprehensive scraping
    pub fn comprehensive_scrape() -> Self {
        let mut config = Self::new();
        config.scrape_config = Some(ScrapeConfig {
            crawl_config: CrawlConfig::default(),
            extract_text: true,
            extract_links: true,
            extract_images: true,
            extract_metadata: true,
            take_screenshots: true,
            screenshot_config: Some(ScreenshotConfig::default()),
        });
        config.advanced_options.extract_resources = true;
        config.advanced_options.handle_javascript = true;
        config
    }

    /// Create a configuration for stealth crawling
    pub fn stealth_crawl() -> Self {
        let mut config = Self::new();
        config.crawl_config.chrome_config = Some(ChromeConfig {
            stealth_mode: true,
            intercept_requests: true,
            block_ads: true,
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string()),
            ..ChromeConfig::default()
        });
        config.crawl_config.delay = Duration::from_secs(3);
        config.advanced_options.rate_limiting = Some(RateLimitConfig {
            requests_per_second: 0.5,
            burst_size: 1,
            delay_on_error: Duration::from_secs(10),
            max_retries: 5,
        });
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_crawl_config() {
        let config = CrawlConfig::default();
        assert_eq!(config.respect_robots_txt, true);
        assert_eq!(config.delay, Duration::from_secs(1));
        assert_eq!(config.concurrency, 10);
    }

    #[test]
    fn test_spider_configuration_creation() {
        let config = SpiderConfiguration::new();
        assert_eq!(config.crawl_config.respect_robots_txt, true);
        assert_eq!(config.advanced_options.follow_redirects, true);
    }

    #[test]
    fn test_fast_crawl_config() {
        let config = SpiderConfiguration::fast_crawl();
        assert_eq!(config.crawl_config.concurrency, 20);
        assert_eq!(config.crawl_config.delay, Duration::from_millis(250));
        assert_eq!(config.crawl_config.cache, true);
    }

    #[test]
    fn test_polite_crawl_config() {
        let config = SpiderConfiguration::polite_crawl();
        assert_eq!(config.crawl_config.concurrency, 2);
        assert_eq!(config.crawl_config.delay, Duration::from_secs(2));
        assert_eq!(config.advanced_options.respect_meta_robots, true);
    }

    #[test]
    fn test_comprehensive_scrape_config() {
        let config = SpiderConfiguration::comprehensive_scrape();
        assert!(config.scrape_config.is_some());
        if let Some(scrape_config) = config.scrape_config {
            assert_eq!(scrape_config.extract_text, true);
            assert_eq!(scrape_config.extract_links, true);
            assert_eq!(scrape_config.extract_images, true);
            assert_eq!(scrape_config.take_screenshots, true);
        }
    }

    #[test]
    fn test_stealth_crawl_config() {
        let config = SpiderConfiguration::stealth_crawl();
        assert!(config.crawl_config.chrome_config.is_some());
        if let Some(chrome_config) = config.crawl_config.chrome_config {
            assert_eq!(chrome_config.stealth_mode, true);
            assert_eq!(chrome_config.intercept_requests, true);
            assert_eq!(chrome_config.block_ads, true);
        }
    }
}
