#[cfg(test)]
mod tests {
    use super::*;
    use async_mcp::{
        client::ClientBuilder,
        protocol::RequestOptions,
        transport::{ClientInMemoryTransport, ServerInMemoryTransport, Transport},
    };
    use serde_json::json;
    use std::time::Duration;

    use crate::build;

    #[tokio::test]
    async fn test_reddit_tools() -> anyhow::Result<()> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stderr)
            .init();

        async fn async_server(transport: ServerInMemoryTransport) {
            let server = build(transport.clone()).unwrap();
            server.listen().await.unwrap();
        }

        let transport = ClientInMemoryTransport::new(|t| tokio::spawn(async_server(t)));
        transport.open().await?;

        let client = ClientBuilder::new(transport).build();
        let client_clone = client.clone();
        tokio::spawn(async move { client_clone.start().await });

        // Test tools/list
        let response = client
            .request(
                "tools/list",
                None,
                RequestOptions::default().timeout(Duration::from_secs(10)),
            )
            .await?;
        println!("Tools list: {:?}", response);

        // Test resources/list
        let response = client
            .request(
                "resources/list",
                None,
                RequestOptions::default().timeout(Duration::from_secs(10)),
            )
            .await?;
        println!("Resources list: {:?}", response);

        Ok(())
    }
}