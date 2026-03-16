use std::net::SocketAddr;

use anyhow::Context;
use deadpool_redis::Config as RedisConfig;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use placebo_api::app_state::AppState;
use placebo_api::config::{Config, Environment};
use placebo_api::redis::session::{self, SessionData};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config = Config::from_env().context("failed to load config")?;

    // Init tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "placebo_api=debug,tower_http=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        environment = %config.environment,
        port = config.port,
        "starting placebo-api"
    );

    // Connect to PostgreSQL
    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .context("failed to run database migrations")?;

    tracing::info!("database connected and migrations applied");

    // Connect to Redis
    let redis_cfg = RedisConfig::from_url(&config.redis_url);
    let redis = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .context("failed to create Redis pool")?;

    // Verify Redis connection
    {
        let mut conn = redis.get().await.context("failed to connect to Redis")?;
        deadpool_redis::redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .context("Redis PING failed")?;
    }

    tracing::info!("redis connected");

    // In dev mode, seed Redis sessions for test users
    if config.environment == Environment::Dev {
        seed_dev_sessions(&redis).await;
    }

    // Build app state
    let state = AppState::new(db, redis, config.clone());

    // Build router via lib
    let app = placebo_api::build_app(state).await;

    // Start server
    let addr = SocketAddr::new(config.host.parse()?, config.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");

    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}

/// Seed Redis with dev session tokens matching 006_seed_users.sql
async fn seed_dev_sessions(redis: &deadpool_redis::Pool) {
    let sessions = [
        (
            "dev-token-alice",
            SessionData {
                user_id: uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000001").unwrap(),
                email: "alice@placebo.dev".into(),
                is_premium: true,
                created_at: chrono::Utc::now().timestamp(),
            },
        ),
        (
            "dev-token-bob",
            SessionData {
                user_id: uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap(),
                email: "bob@placebo.dev".into(),
                is_premium: false,
                created_at: chrono::Utc::now().timestamp(),
            },
        ),
        (
            "dev-token-carol",
            SessionData {
                user_id: uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000003").unwrap(),
                email: "carol@placebo.dev".into(),
                is_premium: true,
                created_at: chrono::Utc::now().timestamp(),
            },
        ),
    ];

    for (token, data) in &sessions {
        match session::create(redis, token, data, 30 * 24 * 3600).await {
            Ok(()) => tracing::debug!("dev session seeded: {token}"),
            Err(e) => tracing::warn!("failed to seed dev session {token}: {e}"),
        }
    }

    tracing::info!("dev sessions seeded (alice, bob, carol)");
}
