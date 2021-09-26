use std::net::TcpListener;

// spawn app and return bound TCP address
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Cannot bind to TCP listener");
    let port = listener.local_addr().unwrap().port();
    let server = null_to_prod::run(listener).expect("Cannot start server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[actix_rt::test]
async fn healtz_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/healthz", &address))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(Some(2), response.content_length());
}
