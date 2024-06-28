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
    pub fn init() -> Result<Self> {
        let db = init_db()?;
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

fn init_db() -> Result<PgPool> {
    let config = get_or_init_config();

    let con_opts = config
        .db_config
        .connection_options()
        // NOTE: Disabled SSL for production, since fly.io's DATABASE_URL returns ?ssl=disable,
        // since all our applications esentially run on an internal network (I think).
        .ssl_mode(sqlx::postgres::PgSslMode::Disable)
        .log_statements(tracing::log::LevelFilter::Debug);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect_lazy_with(con_opts);

    Ok(pool)
}

async fn init_test_db() -> Result<PgPool> {
    // Initialize special AppConfig for Testing
    let mut config = get_or_init_config().to_owned();

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
