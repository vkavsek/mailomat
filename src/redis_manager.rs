use crate::config::AppConfig;

use std::time::Duration;

use secrecy::ExposeSecret;
use tower_sessions_redis_store::fred::{
    self,
    prelude::{ClientLike, Config, Pool},
    types::Builder,
};
use tracing::info;

type Result<T> = core::result::Result<T, fred::error::Error>;

/// Contains a redis connection pool that is cheaply cloneable
#[derive(Clone, Debug)]
pub struct RedisManager {
    pool: Pool,
}

impl RedisManager {
    pub async fn init(app_config: &AppConfig) -> Result<Self> {
        info!("{:<20} - Initializing the REDIS client", "init_redis_cl");
        let conf = Config::from_url(app_config.net_config.redis_uri.expose_secret())?;

        // TODO: is this okay?
        let pool = Builder::from_config(conf)
            .with_connection_config(|config| config.connection_timeout = Duration::from_secs(10))
            .build_pool(10)?;

        pool.init().await?;
        info!("connected to REDIS");

        Ok(RedisManager { pool })
    }

    /// Retrieves Redis connection pool.
    /// Returns an owned pool since it uses a reference under the hood.
    pub fn get_pool(&self) -> Pool {
        self.pool.clone()
    }
}
