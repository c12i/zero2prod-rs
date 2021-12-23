use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use z2p::configuration::get_configuration;
use z2p::email_client::EmailClient;
use z2p::run;
use z2p::telemetry::{get_subscriber, initialize_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "info".into(), std::io::stdout);
    initialize_subscriber(subscriber);
    let config = get_configuration().expect("Error reading configurations");
    // Build `TcpListener`
    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))?;
    // Build postgres connection pool
    let db_connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(config.database.with_db())
        .await
        .expect("Error connecting to Postgres");
    // Build an `EmailClient`
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(config.email_client.base_url, sender_email);
    run(listener, db_connection_pool, email_client)?.await
}
