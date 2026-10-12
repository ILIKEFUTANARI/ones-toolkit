#![cfg(any(feature = "http", feature = "mcp"))]

mod support;

#[cfg(all(feature = "http", not(feature = "mcp-http")))]
#[tokio::test]
async fn standalone_http_keeps_auth_and_business_routes() {
    let (service, _, upstream, _) = support::spawn_test_service().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let router = ones::build_http_app(service);
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = reqwest::Client::new();
    let url = format!("{base}/api/workbench/views");
    assert_eq!(
        client.get(&url).send().await.unwrap().status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    let response = client
        .get(&url)
        .headers(support::auth_headers())
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["views"][0]["uuid"], "view-demo-001");
    assert_eq!(
        client
            .get(format!("{base}/mcp"))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::NOT_FOUND
    );
    let document: serde_json::Value = client
        .get(format!("{base}/openapi.json"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(document["paths"].as_object().unwrap().len(), 18);
    server.abort();
    upstream.abort();
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn mcp_async_io_reads_resources_with_static_credentials() {
    use rmcp::{ServiceExt, model::ReadResourceRequestParams};

    let (service, _, upstream, _) = support::spawn_test_service().await;
    let server = ones::OnesMcpServer::new(
        service,
        ones::McpAuthMode::Static(ones::models::AuthContext {
            email: "alex@example.test".to_string(),
            password: "dummy-password".to_string(),
        }),
    );
    let (server_io, client_io) = tokio::io::duplex(65536);
    let pending = tokio::spawn(async move { server.serve(server_io).await.unwrap() });
    let client = ().serve(client_io).await.unwrap();
    let server = pending.await.unwrap();
    assert_eq!(client.list_all_tools().await.unwrap().len(), 19);
    let result = client
        .read_resource(ReadResourceRequestParams::new("ones://fields"))
        .await
        .unwrap();
    let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &result.contents[0]
    else {
        panic!("expected JSON text resource");
    };
    let fields: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(fields["fields"].as_array().unwrap().len(), 18);
    client.cancel().await.unwrap();
    server.cancel().await.unwrap();
    upstream.abort();
}
