use actix_web::{HttpRequest, Responder, HttpResponse};


pub async fn health_check(_req: HttpRequest) -> impl Responder {
	HttpResponse::Ok().body("Ok")
}