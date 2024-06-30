use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, ConnectOptions, Connection, PgConnection, PgPool};
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{get_or_init_config, DbConfig},
    Result,
};

#[derive(Clone, Debug)]
pub struct ModelManager {
    db: PgPool,
}

impl ModelManager {
    pub async fn init() -> Result<Self> {
        let db = init_db().await?;
        info!("{:<12} - Initializing the DB pool", "init_db");

        Ok(Self { db })
    }
    pub async fn test_init() -> Result<Self> {
        let db = init_test_db().await?;

        Ok(Self { db })
    }
    pub fn db(&self) -> &PgPool {
        &self.db
    }
}

async fn init_db() -> Result<PgPool> {
    let config = tokio::task::spawn_blocking(get_or_init_config).await?;

    let con_opts = config
        .db_config
        .connection_options()
        .log_statements(tracing::log::LevelFilter::Debug);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect_with(con_opts)
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))?;

    Ok(pool)
}

async fn init_test_db() -> Result<PgPool> {
    // Initialize special AppConfig for Testing
    let mut config = tokio::task::spawn_blocking(get_or_init_config)
        .await?
        .to_owned();

    config_test_db(&mut config.db_config).await
}

async fn config_test_db(db_config: &mut DbConfig) -> Result<PgPool> {
    db_config.db_name = Uuid::new_v4().to_string();
    let mut connection =
        PgConnection::connect_with(&db_config.connection_options_without_db()).await?;

    let sql = format!(r#"CREATE DATABASE "{}";"#, db_config.db_name.clone());
    sqlx::query(&sql).execute(&mut connection).await?;

    // Create pool
    let pg_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect_with(db_config.connection_options())
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))?;
    // Migrate DB
    sqlx::migrate!("./migrations").run(&pg_pool).await?;

    Ok(pg_pool)
}
