use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgConnection;

use super::{health_check, subscribe};

pub fn run(listener: TcpListener, db_connection: PgConnection) -> Result<Server, std::io::Error> {
    let db_connection = web::Data::new(db_connection);
    let server = HttpServer::new(move || {
        App::new()
            .route("/healthz", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
