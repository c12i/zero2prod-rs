use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Ok")
}
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/healthz", web::get().to(health_check)))
        .listen(listener)?
        .run();
    Ok(server)
}
