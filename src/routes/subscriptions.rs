use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection_pool),
    fields(
        subsciber_email = %form.email,
        subsciber_name = %form.name
    )
)]
#[allow(clippy::async_yields_async)]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }
    match insert_subscriber(&connection_pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details to database",
    skip(form, connection_pool)
)]
async fn insert_subscriber(connection_pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now()
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error executingg query: {:?}", e);
        e
    })?;
    Ok(())
}

// Returns `true` if the input satisfies all our validation constraints
// on subscriber names, `false` otherwise.
pub fn is_valid_name(s: &str) -> bool {
    // `.trim()` returns a view over the input `s` without trailing whitespace-like
    // characters
    // `.is_empty` checks if the view contains any character
    let is_empty_or_whitespace = s.trim().is_empty();
    // A grapheme is defined by the unicode standard as a "user-perceived"
    // character `å` is a single grapheme, but it's composed of two characters
    //
    // `graphemes` returns an iterator over the graphemes for input `s`
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one
    let is_too_long = s.graphemes(true).count() > 256;
    // Iterate over all characters in the input `s` to check if any of them matches one of
    // the characters in the forbidden array
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_character = s.chars().any(|c| forbidden_characters.contains(&c));
    // Return `false` if any of our conditions have been violated
    !(is_empty_or_whitespace || is_too_long || contains_forbidden_character)
}
