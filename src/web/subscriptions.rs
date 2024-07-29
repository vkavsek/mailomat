use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use tracing::info;

use super::{
    data::{DeserSubscriber, ValidSubscriber},
    Result,
};
use crate::{model::ModelManager, AppState};

#[tracing::instrument(
    name = "Saving new subscriber to the database",
    skip(app_state, subscriber),
    fields(
        subscriber_name = %subscriber.name,
        subscriber_email = %subscriber.email
    )
)]
pub async fn api_subscribe(
    State(app_state): State<Arc<AppState>>,
    Json(subscriber): Json<DeserSubscriber>,
) -> Result<StatusCode> {
    let subscriber: ValidSubscriber =
        tokio::task::spawn_blocking(move || subscriber.try_into()).await??;

    insert_subscriber(app_state.mm.clone(), subscriber.clone()).await?;

    app_state
        .email_client
        .send_email(
            &subscriber.email,
            "Welcome!",
            "Welcome to our newsletter!",
            "Welcome to our newsletter!",
            crate::email_client::MessageStream::Outbound,
        )
        .await?;

    Ok(StatusCode::OK)
}

async fn insert_subscriber(mm: ModelManager, subscriber: ValidSubscriber) -> Result<()> {
    let db_pool = mm.db();

    sqlx::query(
        r#"
        INSERT INTO subscriptions (email, name, subscribed_at, status)
        VALUES ($1, $2, $3, 'confirmed')
    "#,
    )
    .bind(subscriber.email.as_ref())
    .bind(subscriber.name.as_ref())
    .bind(Utc::now())
    .execute(db_pool)
    .await?;

    info!("New subscriber succesfully added to the list.");

    Ok(())
}
