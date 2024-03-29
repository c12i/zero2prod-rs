use z2p::configuration::get_configuration;
use z2p::startup::Application;
use z2p::telemetry::{get_subscriber, initialize_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "info".into(), std::io::stdout);
    initialize_subscriber(subscriber);
    let configuration = get_configuration().expect("Error reading configurations");
    let application = Application::build(configuration).await?;
    application.run_server_until_stopped().await?;
    Ok(())
}
