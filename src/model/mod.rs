use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool};
use tracing::info;

use crate::{config::AppConfig, Result};

#[derive(Clone, Debug)]
pub struct ModelManager {
    db: PgPool,
}

impl ModelManager {
    pub async fn init(config: &AppConfig) -> Result<Self> {
        let db_pool = init_db(config).await?;
        info!("{:<12} - Initializing the DB pool", "init_db");

        Ok(Self { db: db_pool })
    }

    pub async fn configure_for_test(config: &AppConfig) -> Result<()> {
        configure_test_db(config).await?;
        Ok(())
    }

    pub fn db(&self) -> &PgPool {
        &self.db
    }
}

async fn init_db(config: &AppConfig) -> Result<PgPool> {
    let con_opts = config.db_config.connection_options();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect_with(con_opts)
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))?;

    Ok(db_pool)
}

async fn configure_test_db(config: &AppConfig) -> Result<()> {
    let db_config = &config.db_config;
    let mut connection =
        PgConnection::connect_with(&db_config.connection_options_without_db()).await?;

    let sql = format!(r#"CREATE DATABASE "{}";"#, db_config.db_name.clone());
    sqlx::query(&sql).execute(&mut connection).await?;

    // Create pool only used to migrate the DB
    let db_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect_with(db_config.connection_options())
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))?;
    // Migrate DB
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    Ok(())
}
