use actix_http::header::LOCATION;
use actix_web::HttpResponse;

pub fn login() -> HttpResponse {
    HttpResponse::Ok().insert_header((LOCATION, "/")).finish()
}
