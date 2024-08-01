use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use derive_more::Deref;
use serde::Deserialize;
use tracing::debug;

use super::Result;
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
    let db_pool = app_state.mm.db();

    // Get the subscriber_id record from the database.
    let sub_id_record = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens
    WHERE subscription_token = $1"#,
        *subscription_token
    )
    .fetch_one(db_pool)
    .await?;
    let subscriber_id = sub_id_record.subscriber_id;
    debug!("Retrieved subscriber_id: {subscriber_id}");

    // Update the status of the subscriber
    sqlx::query!(
        r#"UPDATE subscriptions 
        SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(db_pool)
    .await?;
    debug!("Updated 'status' to 'confirmed'!");

    // Delete the entry from the subscription_tokens table
    sqlx::query!(
        r#"DELETE FROM subscription_tokens
    WHERE subscription_token = $1"#,
        *subscription_token
    )
    .execute(db_pool)
    .await?;
    debug!("Deleted the entry from 'subscription_tokens'");

    Ok(StatusCode::OK)
}
