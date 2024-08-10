use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

use crate::config::AppConfig;

#[derive(Clone, Debug)]
pub struct DbManager {
    db: PgPool,
}

impl DbManager {
    pub async fn init(config: &AppConfig) -> Result<Self> {
        info!("{:<20} - Initializing the DB pool", "init_db");
        let max_cons = if cfg!(test) { 1 } else { 5 };

        let con_opts = config.db_config.connection_options();

        let db_pool = PgPoolOptions::new()
            .max_connections(max_cons)
            .acquire_timeout(Duration::from_millis(500))
            .connect_with(con_opts)
            .await
            .map_err(|_| Error::FailToCreatePool)?;

        Ok(Self { db: db_pool })
    }

    pub fn db(&self) -> &PgPool {
        &self.db
    }
}

// ###################################
// ->   ERROR
// ###################################
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create db pool")]
    FailToCreatePool,
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migration error: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
}
