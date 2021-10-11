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
    log::info!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now()
    )
    .execute(connection_pool.as_ref())
    .await
    {
        Ok(_) => {
            log::info!("New subscriber details have been saved");
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            log::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
