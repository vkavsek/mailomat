use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use tracing::info;

use super::{
    structs::{DeserSubscriber, ValidSubscriber},
    Result,
};
use crate::model::ModelManager;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/subscribe", post(api_subscribe))
        .with_state(mm)
        .route("/health-check", get(health_check))
}

#[tracing::instrument(name = "HEALTHCHECK")]
async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[tracing::instrument(
    name = "Saving new subscriber to the database",
    skip(mm, subscriber),
    fields(
        subscriber_name = %subscriber.name,
        subscriber_email = %subscriber.email
    )
)]
async fn api_subscribe(
    State(mm): State<ModelManager>,
    Json(subscriber): Json<DeserSubscriber>,
) -> Result<StatusCode> {
    let subscriber = ValidSubscriber::try_from(subscriber)?;

    insert_subscriber(mm, subscriber).await
}

async fn insert_subscriber(mm: ModelManager, subscriber: ValidSubscriber) -> Result<StatusCode> {
    let db = mm.db();

    sqlx::query(
        r#"
        INSERT INTO subscriptions (email, name, subscribed_at)
        VALUES ($1, $2, $3)
    "#,
    )
    .bind(subscriber.email.get())
    .bind(subscriber.name.get())
    .bind(Utc::now())
    .execute(db)
    .await?;

    info!("New subscriber succesfully added to the list.");

    Ok(StatusCode::OK)
}
