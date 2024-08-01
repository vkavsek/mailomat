use std::ops::Deref;

use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use derive_more::Deref;
use serde::Deserialize;

use crate::AppState;

use super::Result;

#[derive(Debug, Deserialize, Deref)]
pub struct SubscribeConfirmQuery {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(app_state))]
pub async fn confirm(
    State(app_state): State<AppState>,
    Query(subscription_token): Query<SubscribeConfirmQuery>,
) -> Result<StatusCode> {
    let db_pool = app_state.mm.db();

    // Get the subscriber_id record from the database.
    let sub_id_record = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens
    WHERE subscription_token = $1"#,
        subscription_token.deref()
    )
    .fetch_one(db_pool)
    .await?;
    let subscriber_id = sub_id_record.subscriber_id;

    // Update the status of the subscriber
    sqlx::query!(
        r#"UPDATE subscriptions 
        SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(db_pool)
    .await?;

    // Delete the entry from the subscription_tokens table
    sqlx::query!(
        r#"DELETE FROM subscription_tokens
    WHERE subscription_token = $1"#,
        subscription_token.deref()
    )
    .execute(db_pool)
    .await?;

    Ok(StatusCode::OK)
}
