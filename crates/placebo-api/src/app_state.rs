use sqlx::PgPool;
use deadpool_redis::Pool as RedisPool;
use std::sync::Arc;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: PgPool, redis: RedisPool, config: Config) -> Self {
        Self {
            db,
            redis,
            config: Arc::new(config),
        }
    }
}
