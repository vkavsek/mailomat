use std::ops::Deref;

use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use tracing::info;
use uuid::Uuid;

use crate::web::{
    self,
    types::{SubscribeConfirmQuery, SubscriptionToken},
    WebResult,
};
use crate::AppState;

// ###################################
// ->   ERROR
// ###################################
#[derive(Debug, thiserror::Error)]
pub enum SubscribeConfirmError {
    #[error("subscriber token was not found in the database")]
    SubTokenInDbNotFound,

    #[error("data parsing error: {0}")]
    DataParsing(#[from] web::types::DataParsingError),

    #[error("error awaiting a blocking tokio task: {0}")]
    BlockingTask(#[from] tokio::task::JoinError),
}

// ###################################
// ->   API
// ###################################
#[tracing::instrument(
    name = "Confirming a pending subscriber",
    skip(app_state, subscription_token),
    fields(sub_token = %subscription_token.subscription_token)
)]
pub async fn subscribe_confirm(
    State(app_state): State<AppState>,
    Query(subscription_token): Query<SubscribeConfirmQuery>,
) -> WebResult<StatusCode> {
    let db_pool = app_state.database_mgr.db();
    // Parse subscription token
    let subscription_token =
        tokio::task::spawn_blocking(move || SubscriptionToken::parse(subscription_token.deref()))
            .await
            .map_err(SubscribeConfirmError::BlockingTask)?
            .map_err(SubscribeConfirmError::DataParsing)?;
    let subscription_token = subscription_token.deref();

    // Get the subscriber_id record from the database.
    let subscriber_id: Uuid = sqlx::query_scalar(
        r#"SELECT subscriber_id FROM subscription_tokens
    WHERE subscription_token = $1"#,
    )
    .bind(subscription_token)
    .fetch_optional(db_pool)
    .await?
    .ok_or_else(|| SubscribeConfirmError::SubTokenInDbNotFound)?;

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
