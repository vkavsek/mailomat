use axum::extract::Query;
use serde::Deserialize;

use super::Result;

#[derive(Debug, Deserialize)]
pub struct SubscribeConfirmQuery {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn confirm(query: Query<SubscribeConfirmQuery>) -> Result<()> {
    Ok(())
}
