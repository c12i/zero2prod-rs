use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

use z2p::configuration::{get_configuration, DatabaseSettings};
use z2p::telemetry::{get_subscriber, initialize_subscriber};

/// Ensure tracing stack is only initialized once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        initialize_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        initialize_subscriber(subscriber);
    }
});

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
    // The first time `initialize` is invoked, the code in `TRACING` is executed
    // all other invocations will skip the execution
    Lazy::force(&TRACING);
    let mut config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Cannot bind to TCP listener");
    let port = listener.local_addr().unwrap().port();

    // test db config
    // overriding database name to a randonm uuid
    config.database.database_name = uuid::Uuid::new_v4().to_string();
    let db_connection_pool = configure_database(&config.database).await;
    let server = z2p::run(listener, db_connection_pool.clone()).expect("Cannot start server");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);
    TestApp::new(address, db_connection_pool)
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.get_connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.get_connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
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
