use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{Executor, Postgres, Transaction};
use tera::{Context, Tera};
use tracing::info;
use uuid::Uuid;

use super::{
    data::{DeserSubscriber, ValidSubscriber},
    Result,
};
use crate::{email_client::MessageStream, AppState};

#[tracing::instrument(
    name = "Saving new subscriber to the database",
    skip(app_state, subscriber),
    fields(
        subscriber_name = %subscriber.name,
        subscriber_email = %subscriber.email
    )
)]
pub async fn api_subscribe(
    State(app_state): State<AppState>,
    Json(subscriber): Json<DeserSubscriber>,
) -> Result<StatusCode> {
    // Spawn a blocking task to validate the subscriber info and generate subscription token.
    let (subscriber, subscription_token) =
        tokio::task::spawn_blocking(move || (subscriber.try_into(), generate_subscription_token()))
            .await?;
    let subscriber: ValidSubscriber = subscriber?;

    // BEGIN sql transaction
    let mut transaction = app_state.model_mgr.db().begin().await?;
    let subscriber_id = insert_subscriber(&mut transaction, subscriber.clone()).await?;
    insert_subscription_token(&mut transaction, &subscription_token, subscriber_id).await?;
    transaction.commit().await?;
    // END sql transaction

    send_confirmation_email(app_state, &subscriber, &subscription_token).await?;

    Ok(StatusCode::OK)
}

async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: ValidSubscriber,
) -> Result<Uuid> {
    let subscriber_id = Uuid::new_v4();

    let query = sqlx::query(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
    )
    .bind(subscriber_id)
    .bind(subscriber.email.as_ref())
    .bind(subscriber.name.as_ref())
    .bind(Utc::now());

    transaction.execute(query).await?;

    Ok(subscriber_id)
}

async fn insert_subscription_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscription_token: &str,
    subscriber_id: Uuid,
) -> Result<()> {
    let query = sqlx::query(
        r#"INSERT INTO subscription_tokens(subscription_token, subscriber_id)
    VALUES ($1, $2)"#,
    )
    .bind(subscription_token)
    .bind(subscriber_id);

    transaction.execute(query).await?;

    Ok(())
}

#[tracing::instrument(
    name = "Sending confirmation email",
    skip(app_state, subscription_token, subscriber)
)]
async fn send_confirmation_email(
    app_state: AppState,
    subscriber: &ValidSubscriber,
    subscription_token: &str,
) -> Result<()> {
    let email_client = &app_state.email_client;
    let base_url = &app_state.base_url;
    let tera = app_state.templ_mgr.tera();

    let confirmation_link =
        format!("{base_url}/subscriptions/confirm?subscription_token={subscription_token}",);

    let html_email = render_confirmation_email_from_template(
        "html_email.html",
        tera,
        subscriber,
        &confirmation_link,
    )?;
    let plain_email = render_confirmation_email_from_template(
        "plain_email.txt",
        tera,
        subscriber,
        &confirmation_link,
    )?;

    email_client
        .send_email(
            &subscriber.email,
            "Welcome to our newsletter!",
            &html_email,
            &plain_email,
            MessageStream::Outbound,
        )
        .await?;

    info!("SUCCESS");
    Ok(())
}

fn render_confirmation_email_from_template(
    template_name: &str,
    tera: &Tera,
    subscriber: &ValidSubscriber,
    confirmation_link: &str,
) -> Result<String> {
    let mut ctx = Context::new();
    ctx.insert("subscriber_name", subscriber.name.as_ref());
    ctx.insert("confirmation_link", confirmation_link);

    let out = tera.render(template_name, &ctx)?;
    Ok(out)
}

/// Generate a random 25 character-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
