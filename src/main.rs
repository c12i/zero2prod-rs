use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use z2p::configuration::get_configuration;
use z2p::run;
use z2p::telemetry::{get_subscriber, initialize_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "info".into(), std::io::stdout);
    initialize_subscriber(subscriber);
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application.port))?;
    let db_connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect(&config.database.get_connection_string())
        .await
        .expect("Error connecting to Postgres");
    run(listener, db_connection_pool)?.await
}
