use std::{char, sync::Arc};

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tracing::info;
use uuid::Uuid;

use super::{
    data::{DeserSubscriber, ValidSubscriber},
    Result,
};
use crate::{email_client::MessageStream, model::ModelManager, AppState, EmailClient};

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
    // Spawn a blocking task to validate the subscriber info and generate subscription token.
    let (subscriber, subscription_token) =
        tokio::task::spawn_blocking(move || (subscriber.try_into(), generate_subscription_token()))
            .await?;
    let subscriber: ValidSubscriber = subscriber?;

    let subscriber_id = insert_subscriber(app_state.mm.clone(), subscriber.clone()).await?;
    insert_subscription_token(app_state.mm.clone(), &subscription_token, subscriber_id).await?;

    send_confirmation_email(
        &app_state.email_client,
        &subscriber,
        &app_state.base_url,
        &subscription_token,
    )
    .await?;

    Ok(StatusCode::OK)
}

async fn insert_subscription_token(
    mm: ModelManager,
    subscription_token: &str,
    subscriber_id: Uuid,
) -> Result<()> {
    let db_pool = mm.db();

    sqlx::query(
        r#"INSERT INTO subscription_tokens(subscription_token, subscriber_id)
    VALUES ($1, $2)"#,
    )
    .bind(subscription_token)
    .bind(subscriber_id)
    .execute(db_pool)
    .await?;

    Ok(())
}

async fn insert_subscriber(mm: ModelManager, subscriber: ValidSubscriber) -> Result<Uuid> {
    let db_pool = mm.db();
    let subscriber_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
    )
    .bind(subscriber_id)
    .bind(subscriber.email.as_ref())
    .bind(subscriber.name.as_ref())
    .bind(Utc::now())
    .execute(db_pool)
    .await?;

    info!("New subscriber succesfully added to the list.");

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Sending confirmation email",
    skip(email_client, base_url, subscription_token)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber: &ValidSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<()> {
    let confirmation_link =
        format!("{base_url}/subscriptions/confirm?subscription_token={subscription_token}",);

    email_client
        .send_email(
            &subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter! <br/>\
                Click <a href={}>here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\n Visit {} to confirm your subscription",
                confirmation_link
            ),
            MessageStream::Outbound,
        )
        .await?;

    Ok(())
}

/// Generate a random 25 character-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
