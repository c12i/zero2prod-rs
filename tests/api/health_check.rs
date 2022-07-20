use crate::helpers::spawn_app;

#[tokio::test]
async fn healtz_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/healthz", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(Some(2), response.content_length());
}
