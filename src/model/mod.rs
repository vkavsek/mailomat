use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    config::{get_config, DatabaseConfig},
    Result,
};

#[derive(Clone, Debug)]
pub struct ModelManager {
    db: PgPool,
}

impl ModelManager {
    pub async fn init() -> Result<Self> {
        let db = init_db().await?;

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
    let config = get_config()?;

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&config.db_config.connection_string())
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))
}

async fn init_test_db() -> Result<PgPool> {
    let mut config = get_config()?;

    config_test_db(&mut config.db_config).await
}

async fn config_test_db(db_config: &mut DatabaseConfig) -> Result<PgPool> {
    db_config.db_name = Uuid::new_v4().to_string();
    let mut connection = PgConnection::connect(&db_config.connection_string_without_db()).await?;

    let sql = format!(r#"CREATE DATABASE "{}";"#, db_config.db_name.clone());
    sqlx::query(&sql).execute(&mut connection).await?;

    // Migrate DB
    let pg_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&db_config.connection_string())
        .await
        .map_err(|ex| crate::Error::ModelFailToCreatePool(ex.to_string()))?;
    sqlx::migrate!("./migrations").run(&pg_pool).await?;

    Ok(pg_pool)
}
