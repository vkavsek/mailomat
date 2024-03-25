use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{config::get_config, Result};

#[derive(Clone, Debug)]
pub struct ModelManager {
    db: PgPool,
}

impl ModelManager {
    pub async fn init() -> Result<Self> {
        let db = init_db().await?;

        Ok(Self { db })
    }
    pub fn db(&self) -> &PgPool {
        &self.db
    }
}

async fn init_db() -> Result<PgPool> {
    let max_connections = if cfg!(test) { 1 } else { 5 };
    let config = get_config().unwrap();

    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&config.db_config.connection_string())
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))
}
