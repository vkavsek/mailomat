use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::AppState;

use super::Result;

#[derive(Debug, Deserialize)]
pub struct SubscribeConfirmQuery {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(app_state))]
pub async fn confirm(
    State(app_state): State<AppState>,
    query: Query<SubscribeConfirmQuery>,
) -> Result<StatusCode> {
    Ok(StatusCode::OK)
}
