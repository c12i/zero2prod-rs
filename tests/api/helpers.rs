use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

use z2p::configuration::{get_configuration, DatabaseSettings};
use z2p::email_client::EmailClient;
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

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub fn new(address: String, db_pool: PgPool) -> Self {
        TestApp { address, db_pool }
    }
}

// spawn app and return bound TCP address
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked, the code in `TRACING` is executed
    // all other invocations will skip the execution
    Lazy::force(&TRACING);
    let mut config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Cannot bind to TCP listener");
    let port = listener.local_addr().unwrap().port();

    // test db config
    // overriding database name to a randonm uuid
    config.database.database_name = uuid::Uuid::new_v4().to_string();
    // TODO: Duplicated code; needs refactor
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );
    let db_connection_pool = configure_database(&config.database).await;
    let server =
        z2p::run(listener, db_connection_pool.clone(), email_client).expect("Cannot start server");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);
    TestApp::new(address, db_connection_pool)
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
