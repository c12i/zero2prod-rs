use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};

fn not_found() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

pub fn run(
    listener: TcpListener,
    db_connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_connection_pool = web::Data::new(db_connection_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthz", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection_pool.clone())
            .app_data(email_client.clone())
            .default_service(web::route().to(not_found))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
