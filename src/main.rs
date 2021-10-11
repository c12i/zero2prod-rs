use std::net::TcpListener;
use env_logger::Env;

use z2p::{get_configuration, run};
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // `init` calls `set_logger`, all that is needed
    // fall back to printing all logs at inflo level or above
    // if RUST_LOG env var has not been set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    let db_connection_pool = PgPool::connect(&config.database.get_connection_string())
        .await
        .expect("Error connecting to Postgres");
    run(listener, db_connection_pool)?.await
}
