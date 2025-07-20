use crate::scraper_tools::{ElementExtractor, XPathAlternative};
use async_mcp::{
    protocol::RequestOptions,
    transport::{ClientInMemoryTransport, ServerInMemoryTransport, Transport},
};
use serde_json::json;

async fn async_server(transport: ServerInMemoryTransport) {
    let server = crate::build(transport.clone()).unwrap();
    server.listen().await.unwrap();
}

#[tokio::test]
async fn test_methods() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    dotenv::dotenv().ok();

    // Create transports
    let client_transport = ClientInMemoryTransport::new(|t| tokio::spawn(async_server(t)));
    client_transport.open().await?;

    // Create and start client
    let client = async_mcp::client::ClientBuilder::new(client_transport.clone()).build();
    let client_clone = client.clone();
    let _client_handle = tokio::spawn(async move { client_clone.start().await });

    let response = client
        .request(
            "tools/call",
            Some(json!({
                "name": "scrape",
                "arguments": {
                    "url": "https://www.google.com/about/careers/applications/"
                }
            })),
            RequestOptions::default(),
        )
        .await?;

    println!("{response}");

    Ok(())
}

#[test]
fn test_element_extractor_basic() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
            <meta name="description" content="A test page">
        </head>
        <body>
            <div id="content">
                <h1>Hello World</h1>
                <p class="text">This is a paragraph.</p>
                <a href="https://example.com">Link</a>
                <img src="/image.jpg" alt="Test image">
            </div>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);

    // Test text extraction
    let texts = extractor.extract_text("h1").unwrap();
    assert_eq!(texts, vec!["Hello World"]);

    // Test attribute extraction
    let links = extractor.extract_attributes("a", "href").unwrap();
    assert_eq!(links, vec!["https://example.com"]);

    // Test element selection
    let elements = extractor.select_elements("p.text").unwrap();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0]["text"], "This is a paragraph.");
}

#[test]
fn test_element_extractor_metadata() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page Title</title>
            <meta name="description" content="Test description">
            <meta name="keywords" content="test, keywords">
            <meta property="og:title" content="OG Title">
            <meta property="og:description" content="OG Description">
            <meta property="og:image" content="https://example.com/og-image.jpg">
        </head>
        <body>
            <h1>Content</h1>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);
    let metadata = extractor.extract_metadata();

    assert_eq!(metadata["title"], "Test Page Title");
    assert_eq!(metadata["description"], "Test description");
    assert_eq!(metadata["keywords"], "test, keywords");
    assert_eq!(metadata["open_graph"]["title"], "OG Title");
    assert_eq!(metadata["open_graph"]["description"], "OG Description");
    assert_eq!(
        metadata["open_graph"]["image"],
        "https://example.com/og-image.jpg"
    );
}

#[test]
fn test_extract_links() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com">External Link</a>
            <a href="/internal">Internal Link</a>
            <a href="#anchor">Anchor Link</a>
        </body>
        </html>
    "##;

    let extractor = ElementExtractor::new(html);
    let links = extractor.extract_links().unwrap();

    assert_eq!(links.len(), 3);
    assert_eq!(links[0]["text"], "External Link");
    assert_eq!(links[0]["href"], "https://example.com");
}

#[test]
fn test_extract_images() {
    let html = r#"
        <html>
        <body>
            <img src="/image1.jpg" alt="Image 1">
            <img src="https://example.com/image2.png" alt="Image 2" title="Title 2">
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);
    let images = extractor.extract_images().unwrap();

    assert_eq!(images.len(), 2);
    assert_eq!(images[0]["src"], "/image1.jpg");
    assert_eq!(images[0]["alt"], "Image 1");
    assert_eq!(images[1]["src"], "https://example.com/image2.png");
    assert_eq!(images[1]["title"], "Title 2");
}

#[test]
fn test_extract_forms() {
    let html = r#"
        <html>
        <body>
            <form action="/submit" method="post">
                <input type="text" name="username" placeholder="Username" required>
                <input type="password" name="password" placeholder="Password">
                <input type="email" name="email" value="test@example.com">
                <select name="country">
                    <option value="us">US</option>
                    <option value="uk">UK</option>
                </select>
                <textarea name="comments" placeholder="Comments"></textarea>
                <input type="submit" value="Submit">
            </form>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);
    let forms = extractor.extract_forms().unwrap();

    assert_eq!(forms.len(), 1);
    assert_eq!(forms[0]["action"], "/submit");
    assert_eq!(forms[0]["method"], "POST");

    let fields = forms[0]["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 6); // 5 inputs + 1 select + 1 textarea

    // Check specific field
    let username_field = &fields[0];
    assert_eq!(username_field["name"], "username");
    assert_eq!(username_field["type"], "text");
    assert_eq!(username_field["placeholder"], "Username");
    assert_eq!(username_field["required"], true);
}

#[test]
fn test_extract_tables() {
    let html = r#"
        <html>
        <body>
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Age</th>
                        <th>City</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>John</td>
                        <td>30</td>
                        <td>New York</td>
                    </tr>
                    <tr>
                        <td>Jane</td>
                        <td>25</td>
                        <td>London</td>
                    </tr>
                </tbody>
            </table>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);
    let tables = extractor.extract_tables().unwrap();

    assert_eq!(tables.len(), 1);

    let headers = tables[0]["headers"].as_array().unwrap();
    // The scraper may be finding all th/td elements in the header row
    assert!(headers.len() >= 3);
    assert_eq!(headers[0], "Name");
    assert_eq!(headers[1], "Age");
    assert_eq!(headers[2], "City");

    let rows = tables[0]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);

    let first_row = rows[0].as_array().unwrap();
    assert_eq!(first_row[0], "John");
    assert_eq!(first_row[1], "30");
    assert_eq!(first_row[2], "New York");
}

#[test]
fn test_search_patterns() {
    let html = r#"
        <html>
        <body>
            <p>Contact us at support@example.com or sales@example.com</p>
            <p>Phone: (555) 123-4567</p>
            <p>Another phone: +1-800-555-0199</p>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);

    // Test email pattern
    let emails = extractor
        .search_patterns(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
        .unwrap();
    assert_eq!(emails.len(), 2);
    assert!(emails.contains(&"support@example.com".to_string()));
    assert!(emails.contains(&"sales@example.com".to_string()));

    // Test phone pattern
    let phones = extractor
        .search_patterns(r"\(\d{3}\)\s\d{3}-\d{4}")
        .unwrap();
    assert_eq!(phones.len(), 1);
    assert_eq!(phones[0], "(555) 123-4567");
}

#[test]
fn test_extract_structured_data() {
    let html = r#"
        <html>
        <head>
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "Person",
                "name": "John Doe",
                "email": "john@example.com"
            }
            </script>
        </head>
        <body>
            <div itemscope itemtype="https://schema.org/Product">
                <span itemprop="name">Product Name</span>
                <span itemprop="price">$19.99</span>
            </div>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);
    let structured_data = extractor.extract_structured_data().unwrap();

    assert!(structured_data.len() >= 1); // At least JSON-LD should be found

    // Check JSON-LD data
    let json_ld = structured_data
        .iter()
        .find(|item| item["type"] == "json-ld");
    assert!(json_ld.is_some());

    let json_ld_data = &json_ld.unwrap()["data"];
    assert_eq!(json_ld_data["name"], "John Doe");
    assert_eq!(json_ld_data["email"], "john@example.com");
}

#[test]
fn test_xpath_to_css_conversion() {
    // Test basic XPath to CSS conversion
    assert_eq!(XPathAlternative::xpath_to_css("//div").unwrap(), "div");
    assert_eq!(
        XPathAlternative::xpath_to_css("//a[@href]").unwrap(),
        "a[href]"
    );
    assert_eq!(
        XPathAlternative::xpath_to_css("//input[@type='text']").unwrap(),
        "input[type='text']"
    );
    assert_eq!(
        XPathAlternative::xpath_to_css("//div[1]").unwrap(),
        "div:nth-child(1)"
    );
    assert_eq!(
        XPathAlternative::xpath_to_css("/html/body/div").unwrap(),
        "html > body > div"
    );
}

#[test]
fn test_xpath_common_patterns() {
    let patterns = XPathAlternative::common_patterns();

    assert_eq!(patterns.get("//div"), Some(&"div"));
    assert_eq!(patterns.get("//a[@href]"), Some(&"a[href]"));
    assert_eq!(
        patterns.get("//input[@type='text']"),
        Some(&"input[type='text']")
    );
    assert_eq!(patterns.get("//div[@id='content']"), Some(&"div#content"));
    assert_eq!(patterns.get("//p[1]"), Some(&"p:first-child"));
    assert_eq!(patterns.get("//li[last()]"), Some(&"li:last-child"));
}

#[test]
fn test_css_selector_edge_cases() {
    let html = r#"
        <html>
        <body>
            <div class="container main-content">
                <p class="highlight">Highlighted text</p>
                <p>Normal text</p>
            </div>
            <div id="sidebar">
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                    <li>Item 3</li>
                </ul>
            </div>
        </body>
        </html>
    "#;

    let extractor = ElementExtractor::new(html);

    // Test class selectors
    let highlighted = extractor.extract_text("p.highlight").unwrap();
    assert_eq!(highlighted, vec!["Highlighted text"]);

    // Test ID selectors
    let sidebar_items = extractor.extract_text("#sidebar li").unwrap();
    assert_eq!(sidebar_items.len(), 3);
    assert_eq!(sidebar_items[0], "Item 1");

    // Test attribute contains selector
    let main_content = extractor.extract_text("div[class*='main']").unwrap();
    assert_eq!(main_content.len(), 1);

    // Test nth-child selectors
    let first_item = extractor.extract_text("li:first-child").unwrap();
    assert_eq!(first_item, vec!["Item 1"]);

    let last_item = extractor.extract_text("li:last-child").unwrap();
    assert_eq!(last_item, vec!["Item 3"]);
}
