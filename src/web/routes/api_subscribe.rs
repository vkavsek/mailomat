use std::ops::Deref;

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use sqlx::{postgres::PgQueryResult, Executor, Postgres, Transaction};
use tera::{Context, Tera};
use tracing::info;
use uuid::Uuid;

use crate::{
    email_client::MessageStream,
    web::{
        data::{DeserSubscriber, SubscriptionToken, ValidSubscriber},
        Result,
    },
    AppState,
};

#[tracing::instrument(
    name = "Saving new subscriber to the database",
    skip(app_state, subscriber),
    fields(
        subscriber_name = %subscriber.name,
        subscriber_email = %subscriber.email
    )
)]
pub async fn subscribe(
    State(app_state): State<AppState>,
    Json(subscriber): Json<DeserSubscriber>,
) -> Result<(StatusCode, &'static str)> {
    // Spawn a blocking task to validate the subscriber info and generate subscription token.
    let (subscriber, subscription_token) =
        tokio::task::spawn_blocking(move || (subscriber.try_into(), SubscriptionToken::generate()))
            .await?;
    let subscriber: ValidSubscriber = subscriber?;
    let standard_response = (
        StatusCode::OK,
        "If this email is not already subscribed, you will receive a confirmation email shortly.",
    );

    // BEGIN sql transaction
    let mut transaction = app_state.model_mgr.db().begin().await?;
    let (subscriber_id, was_subscribed) =
        insert_subscriber(&mut transaction, subscriber.clone()).await?;
    // If the user was already subscribed we want to rollback the changes and fail silently.
    if was_subscribed {
        transaction.rollback().await?;
        return Ok(standard_response);
    }
    insert_subscription_token(&mut transaction, &subscription_token, subscriber_id).await?;
    transaction.commit().await?;
    // END sql transaction

    send_confirmation_email(app_state, &subscriber, &subscription_token).await?;

    Ok(standard_response)
}

/// Tries to insert a new subscriber into the Database, and returns `Result<(user_id, was_subscribed)`.
/// If it fails because the subscriber was already in the DB it will ***NOT*** return an `Err` instead
/// the `was_subscribed` flag is set to `true`, so that we don't expose personal information.
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: ValidSubscriber,
) -> Result<(Uuid, bool)> {
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

    // If error is returned from the executed SQL query we want to check if the user is already subscribed.
    // If they are we don't want to let them know, instead we tell them that if they
    // are not already subscribed they will receive a confirmation email.
    let query_result = transaction.execute(query).await;
    let was_subscribed = was_user_subscribed(query_result)?;

    Ok((subscriber_id, was_subscribed))
}

async fn insert_subscription_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscription_token: &SubscriptionToken,
    subscriber_id: Uuid,
) -> Result<()> {
    let subscription_token = subscription_token.deref();
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
    subscription_token: &SubscriptionToken,
) -> Result<()> {
    let subscription_token = subscription_token.deref();
    let email_client = &app_state.email_client;
    let base_url = &app_state.base_url;
    let tera = app_state.templ_mgr.tera();

    let confirmation_link =
        format!("{base_url}/subscriptions/confirm?subscription_token={subscription_token}",);

    // I think blocking here shouldn't matter much
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

// ###################################
// ->   HELPERS
// ###################################

/// A helper function that checks if the user was already subscribed prior to making the SQL query.
/// Propagates the errors except if the user was already subscribed.
/// In that case it returns `Ok(bool)` where the `bool` signalizes whether we got an `Err` because
/// the user was already subscribed (true), or we got an `Ok` because the user was just subscribed (false).
fn was_user_subscribed(
    query_result: std::result::Result<PgQueryResult, sqlx::Error>,
) -> Result<bool> {
    use sqlx::postgres::PgDatabaseError;

    let is_unique_violation_err = |er: Option<&PgDatabaseError>| {
        if let Some(er) = er {
            er.code() == "23505"
        } else {
            false
        }
    };

    match query_result {
        Err(error) => match error {
            sqlx::Error::Database(er)
                // The user is already subscribed, fail silently
                if is_unique_violation_err(er.try_downcast_ref::<PgDatabaseError>()) =>
            {
                Ok(true)
            }
            // The user is not already subscribed, propagate error
            _ => Err(error.into()),
        },
        Ok(_) => Ok(false),
    }
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
