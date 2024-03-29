use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use tracing::{info, Instrument};

use crate::{model::ModelManager, web::Result};

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/subscribe", post(api_subscribe))
        .with_state(mm)
        .route("/health-check", get(health_check))
}

async fn health_check() -> StatusCode {
    let _ = tokio::time::sleep(Duration::from_secs(1)).await;

    StatusCode::OK
}

#[derive(Deserialize, Debug)]
struct Subscriber {
    pub name: String,
    pub email: String,
}

async fn api_subscribe(
    State(mm): State<ModelManager>,
    Json(subscriber): Json<Subscriber>,
) -> Result<StatusCode> {
    let db = mm.db();

    let q_span = tracing::info_span!("Adding subscriber to the database:");
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
    .instrument(q_span)
    .await?;

    info!("New subscriber succesfully added to the list.");

    Ok(StatusCode::OK)
}
