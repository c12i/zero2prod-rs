use sqlx::PgPool;
use std::net::TcpListener;

use z2p::{get_configuration, get_subscriber, initialize_subscriber, run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "info".into());
    initialize_subscriber(subscriber);
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    let db_connection_pool = PgPool::connect(&config.database.get_connection_string())
        .await
        .expect("Error connecting to Postgres");
    run(listener, db_connection_pool)?.await
}
