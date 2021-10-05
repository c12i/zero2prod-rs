use std::net::TcpListener;

use null_to_prod::get_configuration;
use sqlx::{Connection, PgConnection};

// spawn app and return bound TCP address
async fn spawn_app() -> String {
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Cannot bind to TCP listener");
    let port = listener.local_addr().unwrap().port();
    let db_connection = PgConnection::connect(&config.database.get_connection_string())
        .await
        .unwrap();
    let server = null_to_prod::run(listener, db_connection).expect("Cannot start server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[actix_rt::test]
async fn healtz_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/healthz", &address))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(Some(2), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let address = spawn_app().await;
    let config = get_configuration().expect("Error reading configurations");
    let db_connection_string = config.database.get_connection_string();
    let mut db_connection = PgConnection::connect(&db_connection_string)
        .await
        .expect("Failed to connect to postgres");
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=urslula_leguin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Filed to execute request");
    assert_eq!(200, response.status().as_u16());
    // assert saved subscription
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut db_connection)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("name=urslula_leguin%40gmail", "missing name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // act
        let respose = client
            .post(&format!("{}/subscriptions", &address))
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
