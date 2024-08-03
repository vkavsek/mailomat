use std::ops::Deref;

use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use derive_more::Deref;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use super::{data::SubscriptionToken, Error, Result};
use crate::AppState;

#[derive(Debug, Deserialize, Deref)]
pub struct SubscribeConfirmQuery {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirming a pending subscriber",
    skip(app_state, subscription_token),
    fields(sub_token = %subscription_token.subscription_token)
)]
pub async fn confirm(
    State(app_state): State<AppState>,
    Query(subscription_token): Query<SubscribeConfirmQuery>,
) -> Result<StatusCode> {
    let db_pool = app_state.model_mgr.db();
    // Parse subscription token
    let subscription_token =
        tokio::task::spawn_blocking(move || SubscriptionToken::parse(subscription_token.deref()))
            .await??;
    let subscription_token = subscription_token.deref();

    // Get the subscriber_id record from the database.
    // We also retrieve subscription_token because of the quirks of query_as
    let (subscriber_id, _): (Uuid, String) = sqlx::query_as(
        r#"SELECT subscriber_id, subscription_token FROM subscription_tokens
    WHERE subscription_token = $1"#,
    )
    .bind(subscription_token)
    .fetch_optional(db_pool)
    .await?
    .ok_or_else(|| Error::Unauthorized)?;

    // Update the status of the subscriber - CONFIRM SUBSCRIBER
    sqlx::query(
        r#"UPDATE subscriptions
        SET status = 'confirmed' 
        WHERE id = $1 AND status != 'confirmed'"#,
    )
    .bind(subscriber_id)
    .execute(db_pool)
    .await?;
    info!("SUCCESS!");

    Ok(StatusCode::OK)
}
