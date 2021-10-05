use std::net::TcpListener;

use null_to_prod::{get_configuration, run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Error reading configurations");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    run(listener)?.await
}
