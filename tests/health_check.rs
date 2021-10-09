use std::net::TcpListener;

use null_to_prod::get_configuration;
use sqlx::PgPool;

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub fn new(address: String, db_pool: PgPool) -> Self {
        TestApp { address, db_pool }
    }
}

// spawn app and return bound TCP address
async fn spawn_app() -> TestApp {
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Cannot bind to TCP listener");
    let port = listener.local_addr().unwrap().port();
    let db_connection_pool = PgPool::connect(&config.database.get_connection_string())
        .await
        .expect("Error connecting to Postgres");
    let server =
        null_to_prod::run(listener, db_connection_pool.clone()).expect("Cannot start server");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);
    TestApp::new(address, db_connection_pool)
}

#[actix_rt::test]
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

#[actix_rt::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Filed to execute request");
    assert_eq!(200, response.status().as_u16());
    // assert saved subscription
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("name=urslula_leguin%40gmail", "missing name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // act
        let respose = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Filed to execute request");
        assert_eq!(
            400,
            respose.status().as_u16(),
            "The API did not fail with a 400 request when payload was {}",
            error_message
        );
    }
}
