use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use z2p::configuration::{get_configuration, DatabaseSettings};
use z2p::startup::Application;
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

// spawn app and return bound TCP address
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked, the code in `TRACING` is executed
    // all other invocations will skip the execution
    Lazy::force(&TRACING);
    let configuration = {
        let mut c = get_configuration().expect("Error reading configurations");
        // overriding database name to a randonm uuid
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        // override port to a random OS port
        c.application.port = 0;
        c
    };
    // create and migrate database
    configure_database(&configuration.database).await;
    // launch application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let address = format!("http://127.0.0.1:{}", application.port());
    let db_pool = Application::get_connection_pool(&configuration.database)
        .await
        .expect("Failed to connect to database");
    let _ = tokio::spawn(application.run_server_until_stopped());
    TestApp { address, db_pool }
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
