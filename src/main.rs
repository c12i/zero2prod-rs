use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use sqlx::PgPool;
use z2p::{get_configuration, run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // fall back to printing all logs at inflo level or above
    // if RUST_LOG env var has not been set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("z2p".into(), std::io::stdout);
    // the `with` method is provided by `SubsciberExt`, an extension trait for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    // `set_global_default` can be used by applications to specify what subscriber should be used to process spans
    set_global_default(subscriber).expect("Error getting subscriber");
    // redirect all Actix web `log` events to subscriber
    LogTracer::init().expect("Error setting logger");
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    let db_connection_pool = PgPool::connect(&config.database.get_connection_string())
        .await
        .expect("Error connecting to Postgres");
    run(listener, db_connection_pool)?.await
}
