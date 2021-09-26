fn spawn_app() {
    let server = null_to_prod::run("127.0.0.1:0").expect("Cannot start server");
    let _ = tokio::spawn(server);
}

#[actix_rt::test]
async fn health_check_works() {
    spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8000/healthz")
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(Some(2), response.content_length());
}
