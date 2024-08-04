use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool};
use tracing::info;

use crate::config::AppConfig;

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
    // NOTE: Tests sometimes fail if there is more than 1 max connection. This fixes it.
    let max_cons = if cfg!(test) { 1 } else { 5 };

    let con_opts = config.db_config.connection_options();

    let db_pool = PgPoolOptions::new()
        .max_connections(max_cons)
        .acquire_timeout(Duration::from_millis(500))
        .connect_with(con_opts)
        .await
        .map_err(|ex| Error::FailToCreatePool(format!("Standard DB Pool: {}", ex)))?;

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
        .acquire_timeout(Duration::from_millis(1000))
        .connect_with(db_config.connection_options())
        .await
        .map_err(|ex| Error::FailToCreatePool(format!("Test Config: {}", ex)))?;
    // Migrate DB
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    Ok(())
}

// ###################################
// ->   ERROR
// ###################################
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, derive_more::From)]
pub enum Error {
    FailToCreatePool(String),
    #[from]
    Sqlx(sqlx::Error),
    #[from]
    SqlxMigrate(sqlx::migrate::MigrateError),
}
// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
