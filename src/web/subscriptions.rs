use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use tracing::info;

use super::{
    data::{DeserSubscriber, ValidSubscriber},
    Result,
};
use crate::model::ModelManager;

#[tracing::instrument(
    name = "Saving new subscriber to the database",
    skip(mm, subscriber),
    fields(
        subscriber_name = %subscriber.name,
        subscriber_email = %subscriber.email
    )
)]
pub async fn api_subscribe(
    State(mm): State<ModelManager>,
    Json(subscriber): Json<DeserSubscriber>,
) -> Result<StatusCode> {
    let subscriber = tokio::task::spawn_blocking(move || subscriber.try_into()).await??;

    insert_subscriber(mm, subscriber).await
}

async fn insert_subscriber(mm: ModelManager, subscriber: ValidSubscriber) -> Result<StatusCode> {
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

    Ok(StatusCode::OK)
}
