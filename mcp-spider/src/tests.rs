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

    // let response = client
    //     .request(
    //         "tools/call",
    //         Some(json!({
    //             "name": "crawl",
    //             "arguments": {
    //                 "url": "https://www.google.com/about/careers/applications/",
    //                 "depth": 2,
    //                 "subdomains": false
    //             }
    //         })),
    //         RequestOptions::default(),
    //     )
    //     .await?;
    println!("{response}");

    Ok(())
}
