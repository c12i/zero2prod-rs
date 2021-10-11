use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use actix_web::middleware::Logger;
use sqlx::PgPool;

use super::{health_check, subscribe};

pub fn run(listener: TcpListener, db_connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_connection_pool = web::Data::new(db_connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/healthz", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
