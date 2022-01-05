use z2p::configuration::get_configuration;
use z2p::startup::build;
use z2p::telemetry::{get_subscriber, initialize_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "info".into(), std::io::stdout);
    initialize_subscriber(subscriber);
    let configuration = get_configuration().expect("Error reading configurations");
    let server = build(configuration);
    server.await?;
    Ok(())
}
