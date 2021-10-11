use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
#[allow(unused)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    let FormData { name, email } = form.0;
    let request_id = uuid::Uuid::new_v4();
    log::info!(
        "request_id: {}; saving new subscriber with name: {} and email: {} to the database",
        request_id,
        name,
        email
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        email,
        name,
        chrono::Utc::now()
    )
    .execute(connection_pool.as_ref())
    .await
    {
        Ok(_) => {
            log::info!("request_id: {}; new subscriber details have been saved", request_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
