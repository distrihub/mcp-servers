use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde_json::{json, Value};
use std::collections::HashMap;
use url::Url;

pub struct ScrapingSession {
    client: Client,
    base_url: Option<Url>,
}

impl ScrapingSession {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent("mcp-spider/1.0")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url: None,
        })
    }

    pub async fn fetch_page(&mut self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let html = response.text().await?;

        if let Ok(parsed_url) = Url::parse(url) {
            self.base_url = Some(parsed_url);
        }

        Ok(html)
    }

    pub fn parse_html(&self, html: &str) -> Html {
        Html::parse_document(html)
    }
}

pub struct ElementExtractor {
    document: Html,
}

impl ElementExtractor {
    pub fn new(html: &str) -> Self {
        Self {
            document: Html::parse_document(html),
        }
    }

    /// Extract elements using CSS selectors
    pub fn select_elements(&self, selector: &str) -> Result<Vec<Value>> {
        let css_selector = Selector::parse(selector)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {}", e))?;

        let elements: Vec<Value> = self
            .document
            .select(&css_selector)
            .map(|element| self.element_to_json(element))
            .collect();

        Ok(elements)
    }

    /// Extract text content from elements matching a selector
    pub fn extract_text(&self, selector: &str) -> Result<Vec<String>> {
        let css_selector = Selector::parse(selector)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {}", e))?;

        let texts: Vec<String> = self
            .document
            .select(&css_selector)
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|text| !text.is_empty())
            .collect();

        Ok(texts)
    }

    /// Extract attribute values from elements
    pub fn extract_attributes(&self, selector: &str, attribute: &str) -> Result<Vec<String>> {
        let css_selector = Selector::parse(selector)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {}", e))?;

        let attributes: Vec<String> = self
            .document
            .select(&css_selector)
            .filter_map(|element| element.value().attr(attribute))
            .map(|attr| attr.to_string())
            .collect();

        Ok(attributes)
    }

    /// Extract links from the page
    pub fn extract_links(&self) -> Result<Vec<Value>> {
        let links = self
            .select_elements("a[href]")?
            .into_iter()
            .filter_map(|mut link| {
                if let Some(href) = link.get("href") {
                    if let Some(href_str) = href.as_str() {
                        if !href_str.trim().is_empty() {
                            link["absolute_url"] = json!(self.resolve_url(href_str));
                            return Some(link);
                        }
                    }
                }
                None
            })
            .collect();

        Ok(links)
    }

    /// Extract images from the page
    pub fn extract_images(&self) -> Result<Vec<Value>> {
        let images = self
            .select_elements("img")?
            .into_iter()
            .filter_map(|mut img| {
                if let Some(src) = img.get("src") {
                    if let Some(src_str) = src.as_str() {
                        if !src_str.trim().is_empty() {
                            img["absolute_url"] = json!(self.resolve_url(src_str));
                            return Some(img);
                        }
                    }
                }
                None
            })
            .collect();

        Ok(images)
    }

    /// Extract forms and their fields
    pub fn extract_forms(&self) -> Result<Vec<Value>> {
        let form_selector = Selector::parse("form")
            .map_err(|e| anyhow::anyhow!("Failed to parse form selector: {}", e))?;
        let input_selector = Selector::parse("input, select, textarea")
            .map_err(|e| anyhow::anyhow!("Failed to parse input selector: {}", e))?;

        let forms: Vec<Value> = self
            .document
            .select(&form_selector)
            .map(|form| {
                let action = form.value().attr("action").unwrap_or("");
                let method = form.value().attr("method").unwrap_or("GET");

                let fields: Vec<Value> = form
                    .select(&input_selector)
                    .map(|field| {
                        json!({
                            "name": field.value().attr("name"),
                            "type": field.value().attr("type"),
                            "value": field.value().attr("value"),
                            "placeholder": field.value().attr("placeholder"),
                            "required": field.value().attr("required").is_some(),
                            "tag": field.value().name()
                        })
                    })
                    .collect();

                json!({
                    "action": self.resolve_url(action),
                    "method": method.to_uppercase(),
                    "fields": fields
                })
            })
            .collect();

        Ok(forms)
    }

    /// Extract tables with headers and data
    pub fn extract_tables(&self) -> Result<Vec<Value>> {
        let table_selector = Selector::parse("table")
            .map_err(|e| anyhow::anyhow!("Failed to parse table selector: {}", e))?;
        let header_selector = Selector::parse("thead tr th, tr:first-child th, tr:first-child td")
            .map_err(|e| anyhow::anyhow!("Failed to parse header selector: {}", e))?;
        let row_selector = Selector::parse("tbody tr, tr")
            .map_err(|e| anyhow::anyhow!("Failed to parse row selector: {}", e))?;
        let cell_selector = Selector::parse("td, th")
            .map_err(|e| anyhow::anyhow!("Failed to parse cell selector: {}", e))?;

        let tables: Vec<Value> = self
            .document
            .select(&table_selector)
            .map(|table| {
                // Extract headers
                let headers: Vec<String> = table
                    .select(&header_selector)
                    .map(|th| th.text().collect::<String>().trim().to_string())
                    .collect();

                // Extract rows
                let rows: Vec<Vec<String>> = table
                    .select(&row_selector)
                    .skip(if !headers.is_empty() { 1 } else { 0 }) // Skip header row if exists
                    .map(|row| {
                        row.select(&cell_selector)
                            .map(|cell| cell.text().collect::<String>().trim().to_string())
                            .collect()
                    })
                    .collect();

                json!({
                    "headers": headers,
                    "rows": rows
                })
            })
            .collect();

        Ok(tables)
    }

    /// Extract metadata from the page
    pub fn extract_metadata(&self) -> Value {
        let title = self
            .extract_text("title")
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();
        let description = self
            .extract_attributes("meta[name='description']", "content")
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();
        let keywords = self
            .extract_attributes("meta[name='keywords']", "content")
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();

        // Extract Open Graph metadata
        let og_title = self
            .extract_attributes("meta[property='og:title']", "content")
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();
        let og_description = self
            .extract_attributes("meta[property='og:description']", "content")
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();
        let og_image = self
            .extract_attributes("meta[property='og:image']", "content")
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();

        json!({
            "title": title,
            "description": description,
            "keywords": keywords,
            "open_graph": {
                "title": og_title,
                "description": og_description,
                "image": og_image
            }
        })
    }

    /// Search for text patterns using regex
    pub fn search_patterns(&self, pattern: &str) -> Result<Vec<String>> {
        let regex = Regex::new(pattern)?;
        let text = self.document.root_element().text().collect::<String>();

        let matches: Vec<String> = regex
            .find_iter(&text)
            .map(|m| m.as_str().to_string())
            .collect();

        Ok(matches)
    }

    /// Extract structured data (JSON-LD, microdata)
    pub fn extract_structured_data(&self) -> Result<Vec<Value>> {
        let mut structured_data = Vec::new();

        // Extract JSON-LD
        let json_ld_texts = self.extract_text("script[type='application/ld+json']")?;
        for text in json_ld_texts {
            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                structured_data.push(json!({
                    "type": "json-ld",
                    "data": data
                }));
            }
        }

        // Extract microdata
        let microdata_elements = self.select_elements("[itemscope]")?;
        for element in microdata_elements {
            structured_data.push(json!({
                "type": "microdata",
                "data": element
            }));
        }

        Ok(structured_data)
    }

    fn element_to_json(&self, element: ElementRef) -> Value {
        let tag_name = element.value().name();
        let mut attributes = HashMap::new();

        for attr in element.value().attrs() {
            attributes.insert(attr.0.to_string(), attr.1.to_string());
        }

        let text = element.text().collect::<String>().trim().to_string();
        let inner_html = element.inner_html();

        json!({
            "tag": tag_name,
            "attributes": attributes,
            "text": text,
            "inner_html": inner_html,
            "href": attributes.get("href"),
            "src": attributes.get("src"),
            "alt": attributes.get("alt"),
            "title": attributes.get("title")
        })
    }

    fn resolve_url(&self, relative_url: &str) -> String {
        if relative_url.starts_with("http") {
            return relative_url.to_string();
        }

        // This is a simplified URL resolution
        // In a real implementation, you'd want more robust URL handling
        relative_url.to_string()
    }
}

pub struct FormSubmitter {
    session: ScrapingSession,
}

impl FormSubmitter {
    pub fn new(session: ScrapingSession) -> Self {
        Self { session }
    }

    /// Submit a form with provided data
    pub async fn submit_form(
        &mut self,
        form_action: &str,
        method: &str,
        data: HashMap<String, String>,
    ) -> Result<String> {
        let response = match method.to_uppercase().as_str() {
            "POST" => {
                self.session
                    .client
                    .post(form_action)
                    .form(&data)
                    .send()
                    .await?
            }
            _ => {
                let url = Url::parse_with_params(form_action, data.iter())?;
                self.session.client.get(url).send().await?
            }
        };

        Ok(response.text().await?)
    }
}

/// XPath-like functionality using CSS selectors
/// Since Rust doesn't have robust XPath support, we provide CSS selector alternatives
pub struct XPathAlternative;

impl XPathAlternative {
    /// Convert common XPath expressions to CSS selectors
    pub fn xpath_to_css(xpath: &str) -> Result<String> {
        let mut css = xpath.to_string();

        // First handle // to remove it
        if css.starts_with("//") {
            css = css.replace("//", "");
        } else if css.starts_with("/") {
            css = css.trim_start_matches('/').replace("/", " > ");
        }

        // Handle attribute selection [@attr] -> [attr]
        if css.contains("[@") {
            css = css.replace("[@", "[");
        }

        // Handle position selectors [1] -> :nth-child(1)
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
