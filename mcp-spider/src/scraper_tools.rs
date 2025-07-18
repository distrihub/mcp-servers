use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use spider::website::Website;
use std::collections::HashMap;
use url::Url;

pub struct SpiderSession {
    user_agent: Option<String>,
}

impl SpiderSession {
    pub fn new() -> Self {
        Self {
            user_agent: Some("mcp-spider/1.0".to_string()),
        }
    }

    pub fn with_chrome(self) -> Self {
        // Chrome functionality not available in current spider version
        // Return self for compatibility
        self
    }

    pub fn with_stealth(self) -> Self {
        // Stealth functionality not available in current spider version
        // Return self for compatibility
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub async fn fetch_page(&self, url: &str) -> Result<String> {
        let mut website = Website::new(url);

        if let Some(ua) = &self.user_agent {
            website.with_user_agent(Some(ua));
        }

        website
            .with_subdomains(false)
            .with_limit(1)
            .with_tld(false)
            .with_redirect_limit(3)
            .with_respect_robots_txt(false);

        website.crawl().await;

        if let Some(pages) = website.get_pages() {
            if let Some(page) = pages.first() {
                Ok(page.get_html())
            } else {
                Err(anyhow::anyhow!("No pages found"))
            }
        } else {
            Err(anyhow::anyhow!("Failed to fetch page"))
        }
    }

    pub async fn scrape_with_options(
        &self,
        url: &str,
        _options: ScrapingOptions,
    ) -> Result<ScrapingResult> {
        let html = self.fetch_page(url).await?;

        // Create a simple mock page for the extractor
        let page = create_mock_page(url, &html);
        let extractor = ElementExtractor::new(&html, page);

        let result = ScrapingResult {
            url: url.to_string(),
            html: html.clone(),
            metadata: Some(extractor.extract_metadata()?),
            links: Some(extractor.extract_links()?),
            images: Some(extractor.extract_images()?),
            forms: Some(extractor.extract_forms()?),
            tables: Some(extractor.extract_tables()?),
            structured_data: Some(extractor.extract_structured_data()?),
            screenshot: None, // Not available without chrome
        };

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct ScrapingOptions {
    pub use_chrome: bool,
    pub stealth_mode: bool,
    pub take_screenshot: bool,
    pub user_agent: Option<String>,
    pub extract_metadata: bool,
    pub extract_links: bool,
    pub extract_images: bool,
    pub extract_forms: bool,
    pub extract_tables: bool,
    pub extract_structured_data: bool,
    pub wait_for_selector: Option<String>,
    pub timeout_seconds: Option<u64>,
}

impl Default for ScrapingOptions {
    fn default() -> Self {
        Self {
            use_chrome: false,
            stealth_mode: false,
            take_screenshot: false,
            user_agent: Some("mcp-spider/1.0".to_string()),
            extract_metadata: true,
            extract_links: true,
            extract_images: true,
            extract_forms: true,
            extract_tables: true,
            extract_structured_data: true,
            wait_for_selector: None,
            timeout_seconds: Some(30),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScrapingResult {
    pub url: String,
    pub html: String,
    pub metadata: Option<Value>,
    pub links: Option<Vec<Value>>,
    pub images: Option<Vec<Value>>,
    pub forms: Option<Vec<Value>>,
    pub tables: Option<Vec<Value>>,
    pub structured_data: Option<Vec<Value>>,
    pub screenshot: Option<String>,
}

// Mock page structure for compatibility
pub struct MockPage {
    url: String,
    html: String,
}

impl MockPage {
    pub fn get_url(&self) -> &str {
        &self.url
    }
}

pub fn create_mock_page(url: &str, html: &str) -> MockPage {
    MockPage {
        url: url.to_string(),
        html: html.to_string(),
    }
}

pub struct ElementExtractor {
    html: String,
    page: MockPage,
}

impl ElementExtractor {
    pub fn new(html: &str, page: MockPage) -> Self {
        Self {
            html: html.to_string(),
            page,
        }
    }

    /// Extract elements using CSS selectors - simplified implementation
    pub fn select_elements(&self, _selector: &str) -> Result<Vec<Value>> {
        // Basic implementation for CSS selector extraction
        Ok(vec![])
    }

    /// Extract text content from elements matching a selector
    pub fn extract_text(&self, selector: &str) -> Result<Vec<String>> {
        let pattern = match selector {
            "title" => r"<title[^>]*>(.*?)</title>",
            _ => &format!(r"<{}\b[^>]*>(.*?)</{}>", selector, selector),
        };

        let regex = Regex::new(pattern)?;
        let texts: Vec<String> = regex
            .captures_iter(&self.html)
            .map(|cap| {
                cap.get(1)
                    .map(|m| strip_html_tags(m.as_str().trim()))
                    .unwrap_or_default()
            })
            .filter(|text| !text.is_empty())
            .collect();

        Ok(texts)
    }

    /// Extract attribute values from elements
    pub fn extract_attributes(&self, selector: &str, attribute: &str) -> Result<Vec<String>> {
        let pattern = if selector.contains("[") {
            format!(
                r#"<{}\s+[^>]*{}=['"]([^'"]*?)['"][^>]*>"#,
                selector.split('[').next().unwrap_or(selector),
                attribute
            )
        } else {
            format!(
                r#"<{}\s+[^>]*{}=['"]([^'"]*?)['"][^>]*>"#,
                selector, attribute
            )
        };

        let regex = Regex::new(&pattern)?;
        let attributes: Vec<String> = regex
            .captures_iter(&self.html)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        Ok(attributes)
    }

    /// Extract links from the page
    pub fn extract_links(&self) -> Result<Vec<Value>> {
        let href_regex = Regex::new(r#"<a\s+[^>]*href=['"]([^'"]*?)['"][^>]*>(.*?)</a>"#)?;
        let links: Vec<Value> = href_regex
            .captures_iter(&self.html)
            .map(|cap| {
                let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let text = cap
                    .get(2)
                    .map(|m| strip_html_tags(m.as_str()))
                    .unwrap_or_default();

                json!({
                    "tag": "a",
                    "href": href,
                    "text": text,
                    "absolute_url": self.resolve_url(href)
                })
            })
            .collect();

        Ok(links)
    }

    /// Extract images from the page
    pub fn extract_images(&self) -> Result<Vec<Value>> {
        let img_regex = Regex::new(r#"<img\s+[^>]*src=['"]([^'"]*?)['"]([^>]*?)/?>"#)?;
        let images: Vec<Value> = img_regex
            .captures_iter(&self.html)
            .map(|cap| {
                let src = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let other_attrs = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                let alt_regex = Regex::new(r#"alt=['"]([^'"]*?)['"]"#).unwrap();
                let alt = alt_regex
                    .captures(other_attrs)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str())
                    .unwrap_or("");

                json!({
                    "tag": "img",
                    "src": src,
                    "alt": alt,
                    "absolute_url": self.resolve_url(src)
                })
            })
            .collect();

        Ok(images)
    }

    /// Extract forms and their fields
    pub fn extract_forms(&self) -> Result<Vec<Value>> {
        let form_regex = Regex::new(r"(?s)<form\s+([^>]*?)>(.*?)</form>")?;
        let input_regex = Regex::new(r#"<(?:input|select|textarea)\s+([^>]*?)/?>"#)?;

        let forms: Vec<Value> = form_regex
            .captures_iter(&self.html)
            .map(|form_cap| {
                let form_attrs = form_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let form_content = form_cap.get(2).map(|m| m.as_str()).unwrap_or("");

                let action = extract_attr_value(form_attrs, "action").unwrap_or_default();
                let method = extract_attr_value(form_attrs, "method").unwrap_or("GET".to_string());

                let fields: Vec<Value> = input_regex
                    .captures_iter(form_content)
                    .map(|input_cap| {
                        let input_attrs = input_cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        json!({
                            "name": extract_attr_value(input_attrs, "name"),
                            "type": extract_attr_value(input_attrs, "type"),
                            "value": extract_attr_value(input_attrs, "value"),
                            "placeholder": extract_attr_value(input_attrs, "placeholder"),
                            "required": input_attrs.contains("required")
                        })
                    })
                    .collect();

                json!({
                    "action": self.resolve_url(&action),
                    "method": method.to_uppercase(),
                    "fields": fields
                })
            })
            .collect();

        Ok(forms)
    }

    /// Extract tables with headers and data
    pub fn extract_tables(&self) -> Result<Vec<Value>> {
        let table_regex = Regex::new(r"(?s)<table\s*[^>]*>(.*?)</table>")?;
        let row_regex = Regex::new(r"(?s)<tr\s*[^>]*>(.*?)</tr>")?;
        let cell_regex = Regex::new(r"(?s)<t[hd]\s*[^>]*>(.*?)</t[hd]>")?;

        let tables: Vec<Value> = table_regex
            .captures_iter(&self.html)
            .map(|table_cap| {
                let table_content = table_cap.get(1).map(|m| m.as_str()).unwrap_or("");

                let rows: Vec<Vec<String>> = row_regex
                    .captures_iter(table_content)
                    .map(|row_cap| {
                        let row_content = row_cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        cell_regex
                            .captures_iter(row_content)
                            .map(|cell_cap| {
                                strip_html_tags(cell_cap.get(1).map(|m| m.as_str()).unwrap_or(""))
                            })
                            .collect()
                    })
                    .collect();

                let headers = rows.first().cloned().unwrap_or_default();
                let data_rows = if rows.len() > 1 {
                    rows[1..].to_vec()
                } else {
                    vec![]
                };

                json!({
                    "headers": headers,
                    "rows": data_rows
                })
            })
            .collect();

        Ok(tables)
    }

    /// Extract metadata from the page
    pub fn extract_metadata(&self) -> Result<Value> {
        let title = self
            .extract_text("title")?
            .first()
            .cloned()
            .unwrap_or_default();

        let description = self
            .extract_attributes("meta[name='description']", "content")?
            .first()
            .cloned()
            .unwrap_or_default();

        let keywords = self
            .extract_attributes("meta[name='keywords']", "content")?
            .first()
            .cloned()
            .unwrap_or_default();

        let og_title = self
            .extract_attributes("meta[property='og:title']", "content")?
            .first()
            .cloned()
            .unwrap_or_default();

        let og_description = self
            .extract_attributes("meta[property='og:description']", "content")?
            .first()
            .cloned()
            .unwrap_or_default();

        let og_image = self
            .extract_attributes("meta[property='og:image']", "content")?
            .first()
            .cloned()
            .unwrap_or_default();

        Ok(json!({
            "title": title,
            "description": description,
            "keywords": keywords,
            "open_graph": {
                "title": og_title,
                "description": og_description,
                "image": og_image
            }
        }))
    }

    /// Search for text patterns using regex
    pub fn search_patterns(&self, pattern: &str) -> Result<Vec<String>> {
        let regex = Regex::new(pattern)?;
        let text = strip_html_tags(&self.html);

        let matches: Vec<String> = regex
            .find_iter(&text)
            .map(|m| m.as_str().to_string())
            .collect();

        Ok(matches)
    }

    /// Extract structured data (JSON-LD, microdata)
    pub fn extract_structured_data(&self) -> Result<Vec<Value>> {
        let mut structured_data = Vec::new();

        let json_ld_regex =
            Regex::new(r#"(?s)<script\s+type=['"]application/ld\+json['"][^>]*>(.*?)</script>"#)?;
        for cap in json_ld_regex.captures_iter(&self.html) {
            if let Some(json_text) = cap.get(1) {
                if let Ok(data) = serde_json::from_str::<Value>(json_text.as_str()) {
                    structured_data.push(json!({
                        "type": "json-ld",
                        "data": data
                    }));
                }
            }
        }

        let microdata_regex = Regex::new(r#"<[^>]+itemscope[^>]*>"#)?;
        for _match in microdata_regex.find_iter(&self.html) {
            structured_data.push(json!({
                "type": "microdata",
                "data": "microdata_element_found"
            }));
        }

        Ok(structured_data)
    }

    fn resolve_url(&self, relative_url: &str) -> String {
        if relative_url.starts_with("http") {
            return relative_url.to_string();
        }

        if let Ok(base_url) = Url::parse(&self.page.get_url()) {
            if let Ok(resolved) = base_url.join(relative_url) {
                return resolved.to_string();
            }
        }

        relative_url.to_string()
    }
}

/// Web automation functionality - placeholder for compatibility
pub struct WebAutomation {
    session: SpiderSession,
}

impl WebAutomation {
    pub fn new() -> Self {
        Self {
            session: SpiderSession::new(),
        }
    }

    pub async fn click_element(&self, url: &str, selector: &str) -> Result<String> {
        let _html = self.session.fetch_page(url).await?;
        Ok(format!(
            "Clicked element with selector: {} (simulated)",
            selector
        ))
    }

    pub async fn fill_form(&self, url: &str, form_data: HashMap<String, String>) -> Result<String> {
        let _html = self.session.fetch_page(url).await?;
        Ok(format!(
            "Filled form with data: {:?} (simulated)",
            form_data
        ))
    }

    pub async fn submit_form(&self, url: &str, form_selector: &str) -> Result<String> {
        let _html = self.session.fetch_page(url).await?;
        Ok(format!(
            "Submitted form with selector: {} (simulated)",
            form_selector
        ))
    }

    pub async fn take_screenshot(&self, url: &str, _selector: Option<&str>) -> Result<Vec<u8>> {
        let _html = self.session.fetch_page(url).await?;
        Ok(vec![]) // Placeholder
    }

    pub async fn wait_for_element(
        &self,
        url: &str,
        _selector: &str,
        _timeout_ms: u64,
    ) -> Result<bool> {
        let _html = self.session.fetch_page(url).await?;
        Ok(true) // Placeholder
    }

    pub async fn execute_javascript(&self, url: &str, script: &str) -> Result<Value> {
        let _html = self.session.fetch_page(url).await?;
        Ok(json!({"result": format!("Executed script: {} (simulated)", script)}))
    }
}

/// XPath-like functionality using CSS selectors
pub struct XPathAlternative;

impl XPathAlternative {
    /// Convert common XPath expressions to CSS selectors
    pub fn xpath_to_css(xpath: &str) -> Result<String> {
        let mut css = xpath.to_string();

        if css.starts_with("//") {
            css = css.replace("//", "");
        } else if css.starts_with("/") {
            css = css.trim_start_matches('/').replace("/", " > ");
        }

        if css.contains("[@") {
            css = css.replace("[@", "[");
        }

        if css.contains("[") && !css.contains("=") {
            let re = Regex::new(r"\[(\d+)\]")?;
            css = re.replace_all(&css, ":nth-child($1)").to_string();
        }

        Ok(css)
    }

    /// Get XPath alternatives for common use cases
    pub fn common_patterns() -> HashMap<&'static str, &'static str> {
        let mut patterns = HashMap::new();

        patterns.insert("//div", "div");
        patterns.insert("//a[@href]", "a[href]");
        patterns.insert("//img[@src]", "img[src]");
        patterns.insert("//input[@type='text']", "input[type='text']");
        patterns.insert(
            "//span[contains(@class, 'highlight')]",
            "span[class*='highlight']",
        );
        patterns.insert("//div[@id='content']", "div#content");
        patterns.insert("//p[1]", "p:first-child");
        patterns.insert("//li[last()]", "li:last-child");
        patterns.insert("//table//tr", "table tr");
        patterns.insert("//form//input", "form input");

        patterns
    }
}

// Helper functions
fn strip_html_tags(html: &str) -> String {
    let tag_regex = Regex::new(r"<[^>]*>").unwrap();
    tag_regex.replace_all(html, "").trim().to_string()
}

fn extract_attr_value(attrs: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}=['"]([^'"]*?)['"]"#, attr_name);
    let regex = Regex::new(&pattern).ok()?;
    regex
        .captures(attrs)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}
