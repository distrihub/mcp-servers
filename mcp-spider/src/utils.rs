use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use url::Url;
use tracing::{debug, warn};

/// URL utilities for processing and validating URLs
pub struct UrlUtils;

impl UrlUtils {
    /// Normalize a URL by removing fragments, sorting query parameters, etc.
    pub fn normalize_url(url: &str) -> Result<String> {
        let mut parsed = Url::parse(url)
            .map_err(|e| anyhow!("Failed to parse URL '{}': {}", url, e))?;

        // Remove fragment
        parsed.set_fragment(None);

        // Sort query parameters for consistency
        if let Some(query) = parsed.query() {
            let mut params: Vec<(&str, &str)> = parsed.query_pairs().collect();
            params.sort_by(|a, b| a.0.cmp(&b.0));
            
            let sorted_query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            
            parsed.set_query(Some(&sorted_query));
        }

        Ok(parsed.to_string())
    }

    /// Check if a URL matches any of the given regex patterns
    pub fn matches_patterns(url: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(url) {
                    return true;
                }
            } else {
                warn!("Invalid regex pattern: {}", pattern);
            }
        }
        false
    }

    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> Result<String> {
        let parsed = Url::parse(url)
            .map_err(|e| anyhow!("Failed to parse URL '{}': {}", url, e))?;
        
        parsed.host_str()
            .map(|host| host.to_string())
            .ok_or_else(|| anyhow!("No host found in URL: {}", url))
    }

    /// Check if two URLs are from the same domain
    pub fn same_domain(url1: &str, url2: &str) -> Result<bool> {
        let domain1 = Self::extract_domain(url1)?;
        let domain2 = Self::extract_domain(url2)?;
        Ok(domain1 == domain2)
    }

    /// Check if URL is a subdomain of the given domain
    pub fn is_subdomain(url: &str, base_domain: &str) -> Result<bool> {
        let domain = Self::extract_domain(url)?;
        Ok(domain == base_domain || domain.ends_with(&format!(".{}", base_domain)))
    }

    /// Convert relative URL to absolute using base URL
    pub fn resolve_relative_url(base_url: &str, relative_url: &str) -> Result<String> {
        let base = Url::parse(base_url)
            .map_err(|e| anyhow!("Failed to parse base URL '{}': {}", base_url, e))?;
        
        let resolved = base.join(relative_url)
            .map_err(|e| anyhow!("Failed to resolve '{}' against '{}': {}", relative_url, base_url, e))?;
        
        Ok(resolved.to_string())
    }

    /// Check if URL is likely a file download (based on extension)
    pub fn is_file_download(url: &str) -> bool {
        let download_extensions = [
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
            "zip", "rar", "tar", "gz", "7z",
            "exe", "msi", "dmg", "pkg",
            "mp3", "mp4", "avi", "mov", "wmv",
            "jpg", "jpeg", "png", "gif", "svg", "webp",
        ];

        if let Ok(parsed) = Url::parse(url) {
            if let Some(segments) = parsed.path_segments() {
                if let Some(last_segment) = segments.last() {
                    if let Some(extension) = last_segment.split('.').last() {
                        return download_extensions.contains(&extension.to_lowercase().as_str());
                    }
                }
            }
        }

        false
    }

    /// Get file extension from URL
    pub fn get_file_extension(url: &str) -> Option<String> {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(segments) = parsed.path_segments() {
                if let Some(last_segment) = segments.last() {
                    if let Some(extension) = last_segment.split('.').last() {
                        return Some(extension.to_lowercase());
                    }
                }
            }
        }
        None
    }
}

/// Content filtering utilities
pub struct ContentFilter;

impl ContentFilter {
    /// Clean text content by removing extra whitespace, newlines, etc.
    pub fn clean_text(text: &str) -> String {
        // Replace multiple whitespace characters with single space
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        let cleaned = whitespace_regex.replace_all(text.trim(), " ");
        
        // Remove common unwanted characters
        let unwanted_regex = Regex::new(r"[\x00-\x1F\x7F-\x9F]").unwrap();
        unwanted_regex.replace_all(&cleaned, "").to_string()
    }

    /// Extract email addresses from text
    pub fn extract_emails(text: &str) -> Vec<String> {
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        email_regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Extract phone numbers from text (basic patterns)
    pub fn extract_phone_numbers(text: &str) -> Vec<String> {
        let phone_regex = Regex::new(r"\b(?:\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})\b").unwrap();
        phone_regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Check if content appears to be a login or registration page
    pub fn is_auth_page(text: &str, url: &str) -> bool {
        let auth_keywords = [
            "login", "signin", "sign in", "log in",
            "register", "signup", "sign up",
            "password", "username", "email",
            "authentication", "forgot password"
        ];

        let text_lower = text.to_lowercase();
        let url_lower = url.to_lowercase();

        auth_keywords.iter().any(|keyword| {
            text_lower.contains(keyword) || url_lower.contains(keyword)
        })
    }

    /// Check if content appears to be an error page
    pub fn is_error_page(text: &str, status_code: Option<u16>) -> bool {
        if let Some(code) = status_code {
            if code >= 400 {
                return true;
            }
        }

        let error_keywords = [
            "404", "not found", "page not found",
            "500", "internal server error",
            "403", "forbidden", "access denied",
            "502", "bad gateway",
            "503", "service unavailable"
        ];

        let text_lower = text.to_lowercase();
        error_keywords.iter().any(|keyword| text_lower.contains(keyword))
    }

    /// Extract social media links from content
    pub fn extract_social_links(links: &[String]) -> HashMap<String, Vec<String>> {
        let mut social_links = HashMap::new();
        
        let platforms = [
            ("twitter", vec!["twitter.com", "x.com"]),
            ("facebook", vec!["facebook.com", "fb.com"]),
            ("instagram", vec!["instagram.com"]),
            ("linkedin", vec!["linkedin.com"]),
            ("youtube", vec!["youtube.com", "youtu.be"]),
            ("tiktok", vec!["tiktok.com"]),
            ("github", vec!["github.com"]),
        ];

        for link in links {
            for (platform, domains) in &platforms {
                if domains.iter().any(|domain| link.contains(domain)) {
                    social_links
                        .entry(platform.to_string())
                        .or_insert_with(Vec::new)
                        .push(link.clone());
                }
            }
        }

        social_links
    }
}

/// Duplicate detection utilities
pub struct DuplicateDetector {
    seen_urls: HashSet<String>,
    seen_content_hashes: HashSet<u64>,
}

impl DuplicateDetector {
    pub fn new() -> Self {
        Self {
            seen_urls: HashSet::new(),
            seen_content_hashes: HashSet::new(),
        }
    }

    /// Check if URL has been seen before
    pub fn is_duplicate_url(&mut self, url: &str) -> bool {
        !self.seen_urls.insert(url.to_string())
    }

    /// Check if content is duplicate based on hash
    pub fn is_duplicate_content(&mut self, content: &str) -> bool {
        let hash = self.hash_content(content);
        !self.seen_content_hashes.insert(hash)
    }

    /// Simple hash function for content
    fn hash_content(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Get statistics about duplicates found
    pub fn get_stats(&self) -> (usize, usize) {
        (self.seen_urls.len(), self.seen_content_hashes.len())
    }
}

impl Default for DuplicateDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting utilities
pub struct RateLimiter {
    last_request_time: std::time::Instant,
    min_delay: std::time::Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        let min_delay = std::time::Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            last_request_time: std::time::Instant::now() - min_delay,
            min_delay,
        }
    }

    /// Wait if necessary to respect rate limit
    pub async fn wait_if_needed(&mut self) {
        let elapsed = self.last_request_time.elapsed();
        if elapsed < self.min_delay {
            let wait_time = self.min_delay - elapsed;
            tokio::time::sleep(wait_time).await;
        }
        self.last_request_time = std::time::Instant::now();
    }
}

/// Utility functions for working with robots.txt
pub struct RobotsUtils;

impl RobotsUtils {
    /// Check if URL is allowed according to robots.txt rules
    pub fn is_allowed(_robots_txt: &str, _user_agent: &str, _url: &str) -> bool {
        // This is a simplified implementation
        // In a real implementation, you would parse the robots.txt file
        // and check the rules for the given user agent and URL
        true
    }

    /// Extract sitemap URLs from robots.txt
    pub fn extract_sitemaps(robots_txt: &str) -> Vec<String> {
        let mut sitemaps = Vec::new();
        
        for line in robots_txt.lines() {
            let line = line.trim();
            if line.to_lowercase().starts_with("sitemap:") {
                if let Some(url) = line.split(':').nth(1) {
                    let sitemap_url = url.trim();
                    if !sitemap_url.is_empty() {
                        sitemaps.push(sitemap_url.to_string());
                    }
                }
            }
        }

        sitemaps
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
    request_count: u64,
    bytes_downloaded: u64,
    errors: u64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            request_count: 0,
            bytes_downloaded: 0,
            errors: 0,
        }
    }

    pub fn record_request(&mut self, bytes: usize, success: bool) {
        self.request_count += 1;
        self.bytes_downloaded += bytes as u64;
        if !success {
            self.errors += 1;
        }
    }

    pub fn get_stats(&self) -> PerformanceStats {
        let elapsed = self.start_time.elapsed();
        let requests_per_second = if elapsed.as_secs() > 0 {
            self.request_count as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        PerformanceStats {
            duration: elapsed,
            requests: self.request_count,
            bytes_downloaded: self.bytes_downloaded,
            errors: self.errors,
            requests_per_second,
            error_rate: if self.request_count > 0 {
                self.errors as f64 / self.request_count as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub duration: std::time::Duration,
    pub requests: u64,
    pub bytes_downloaded: u64,
    pub errors: u64,
    pub requests_per_second: f64,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normalization() {
        let url = "https://example.com/path?b=2&a=1#fragment";
        let normalized = UrlUtils::normalize_url(url).unwrap();
        assert_eq!(normalized, "https://example.com/path?a=1&b=2");
    }

    #[test]
    fn test_domain_extraction() {
        let url = "https://www.example.com/path";
        let domain = UrlUtils::extract_domain(url).unwrap();
        assert_eq!(domain, "www.example.com");
    }

    #[test]
    fn test_same_domain() {
        let url1 = "https://example.com/page1";
        let url2 = "https://example.com/page2";
        let url3 = "https://other.com/page";
        
        assert!(UrlUtils::same_domain(url1, url2).unwrap());
        assert!(!UrlUtils::same_domain(url1, url3).unwrap());
    }

    #[test]
    fn test_subdomain_check() {
        let url = "https://sub.example.com/path";
        let base_domain = "example.com";
        
        assert!(UrlUtils::is_subdomain(url, base_domain).unwrap());
    }

    #[test]
    fn test_file_download_detection() {
        assert!(UrlUtils::is_file_download("https://example.com/file.pdf"));
        assert!(UrlUtils::is_file_download("https://example.com/image.jpg"));
        assert!(!UrlUtils::is_file_download("https://example.com/page.html"));
    }

    #[test]
    fn test_text_cleaning() {
        let dirty_text = "  Hello   \n\n  World  \t  ";
        let clean = ContentFilter::clean_text(dirty_text);
        assert_eq!(clean, "Hello World");
    }

    #[test]
    fn test_email_extraction() {
        let text = "Contact us at info@example.com or support@test.org";
        let emails = ContentFilter::extract_emails(text);
        assert_eq!(emails.len(), 2);
        assert!(emails.contains(&"info@example.com".to_string()));
        assert!(emails.contains(&"support@test.org".to_string()));
    }

    #[test]
    fn test_auth_page_detection() {
        let text = "Please enter your username and password to login";
        let url = "https://example.com/login";
        assert!(ContentFilter::is_auth_page(text, url));
    }

    #[test]
    fn test_error_page_detection() {
        let text = "404 - Page not found";
        assert!(ContentFilter::is_error_page(text, Some(404)));
    }

    #[test]
    fn test_duplicate_detector() {
        let mut detector = DuplicateDetector::new();
        
        assert!(!detector.is_duplicate_url("https://example.com"));
        assert!(detector.is_duplicate_url("https://example.com"));
        
        assert!(!detector.is_duplicate_content("Hello World"));
        assert!(detector.is_duplicate_content("Hello World"));
    }

    #[test]
    fn test_robots_sitemap_extraction() {
        let robots_txt = r#"
User-agent: *
Disallow: /private/

Sitemap: https://example.com/sitemap.xml
Sitemap: https://example.com/sitemap2.xml
"#;
        
        let sitemaps = RobotsUtils::extract_sitemaps(robots_txt);
        assert_eq!(sitemaps.len(), 2);
        assert!(sitemaps.contains(&"https://example.com/sitemap.xml".to_string()));
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record_request(1024, true);
        monitor.record_request(2048, false);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.requests, 2);
        assert_eq!(stats.bytes_downloaded, 3072);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.error_rate, 0.5);
    }
}