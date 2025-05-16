mod dashboard;

use anyhow::anyhow;
use axum::{extract::FromRequestParts, http::request::Parts};
// re-exports
pub use dashboard::dashboard;

use serde::{Deserialize, Serialize};
use tower_cookies::cookie::time::OffsetDateTime;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::web;

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("unauthorized access")]
    Unauthorized,
    #[error("tower_sessions error: {0}")]
    Session(#[from] tower_sessions::session::Error),

    #[error("unexpected error: {0}")]
    Unexpected(#[from] anyhow::Error),
}

/// Admin information
#[derive(Clone, Deserialize, Serialize)]
pub struct AdminData {
    user_id: Uuid,
    first_seen: OffsetDateTime,
    last_seen: OffsetDateTime,
}

impl AdminData {
    pub fn new(user_id: Uuid) -> Self {
        AdminData {
            user_id,
            first_seen: OffsetDateTime::now_utc(),
            last_seen: OffsetDateTime::now_utc(),
        }
    }
}

/// An implementation of admin sessions.
/// Can be extracted from the request and can therefore be used in a handler.
/// Contains the information about the admin and the session.
/// Admins are (somewhat confusingly) stored in the `users` table in the database.
pub struct AdminSession {
    session: Session,
    admin_data: AdminData,
}

#[allow(dead_code)]
impl AdminSession {
    const ADMIN_DATA_KEY: &'static str = "user_id";

    // —> constructor
    pub fn new(session: Session, admin_data: AdminData) -> Self {
        Self {
            session,
            admin_data,
        }
    }

    // —> getters
    pub fn data(&self) -> &AdminData {
        &self.admin_data
    }

    pub fn user_id(&self) -> Uuid {
        self.admin_data.user_id
    }

    pub fn first_seen(&self) -> OffsetDateTime {
        self.admin_data.first_seen
    }

    pub fn last_seen(&self) -> OffsetDateTime {
        self.admin_data.last_seen
    }

    pub async fn cycle_id(&self) -> Result<(), AdminError> {
        self.session.cycle_id().await.map_err(AdminError::Session)
    }

    /// updates the contained session with the contained data.
    /// `update_session` or `update_session_with` needs to be called when first building the AdminSession.
    pub async fn update_session(&self) -> Result<(), AdminError> {
        self.update_session_with(self.admin_data.clone()).await
    }

    /// updates the contained session with the provided data.
    pub async fn update_session_with(&self, admin_data: AdminData) -> Result<(), AdminError> {
        self.session
            .insert(Self::ADMIN_DATA_KEY, admin_data)
            .await?;
        Ok(())
    }
}

impl<S> FromRequestParts<S> for AdminSession
where
    S: Send + Sync,
{
    type Rejection = web::Error;

    #[instrument(skip_all, name = "AdminSession from_request_parts")]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(session) = Session::from_request_parts(parts, state).await else {
            return Err(anyhow!("unable to extract the session from request parts").into());
        };

        let Some(mut admin_data) = session
            .get::<AdminData>(Self::ADMIN_DATA_KEY)
            .await
            .map_err(AdminError::Session)?
        else {
            return Err(AdminError::Unauthorized.into());
        };

        admin_data.last_seen = OffsetDateTime::now_utc();

        // build a new AdminSession from the changed data.
        let admin_session = Self::new(session, admin_data);
        // update with the contained data
        admin_session.update_session().await?;

        Ok(admin_session)
    }
}
