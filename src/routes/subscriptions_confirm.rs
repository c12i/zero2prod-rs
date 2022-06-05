use std::fmt::Debug;

use actix_http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use serde::Deserialize;
use sqlx::PgPool;

use crate::utils::error_chain_fmt;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
	name = "Confirm a pending subscriber"
	skip(parameters)
)]
#[allow(clippy::async_yields_async)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscriptionConfirmError> {
    let result = get_subscriber_id_from_token(&pool, &parameters.subscription_token).await?;
    if let Some(subscriber_id) = result {
        confirm_subscriber(&pool, subscriber_id).await?;
        return Ok(HttpResponse::Ok().finish());
    }
    Ok(HttpResponse::Unauthorized().finish())
}

#[tracing::instrument(
    "Get subscriber_id from subscription token",
    skip(subscription_token, pool)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<uuid::Uuid>, SubscriptionConfirmError> {
    let result = sqlx::query!(
        r#"
		SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1
		"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(SubscriptionConfirmError::UnexpectedError)?;
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Mark a subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: uuid::Uuid,
) -> Result<(), SubscriptionConfirmError> {
    sqlx::query!(
        r#"
			UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
		"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(SubscriptionConfirmError::UnexpectedError)?;
    Ok(())
}

#[derive(thiserror::Error)]
pub enum SubscriptionConfirmError {
    #[error("An error has occurred: {0}")]
    UnexpectedError(#[from] sqlx::Error),
}

impl Debug for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscriptionConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscriptionConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
