use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use tracing::info;

use crate::{model::ModelManager, web::Result};

use super::Subscriber;

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

#[tracing::instrument(name = "Saving new subscriber to the database", skip(mm, subscriber))]
async fn api_subscribe(
    State(mm): State<ModelManager>,
    Json(subscriber): Json<Subscriber>,
) -> Result<StatusCode> {
    let db = mm.db();

    // TODO: Check email vailidity

    sqlx::query(
        r#"
        INSERT INTO subscriptions (email, name, subscribed_at)
        VALUES ($1, $2, $3)
    "#,
    )
    .bind(subscriber.email)
    .bind(subscriber.name)
    .bind(Utc::now())
    .execute(db)
    .await?;

    info!("New subscriber succesfully added to the list.");

    Ok(StatusCode::OK)
}
