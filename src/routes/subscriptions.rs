use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use tracing_futures::Instrument;

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
    // creae span at info level
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id, // %: use Display implementation
        subscriber_name = %name,
        subscriber_email = %email
    );
    // XXX: calling `enter` in an async function, not good
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id: {}; new subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
