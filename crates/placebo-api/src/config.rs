use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Environment {
    Dev,
    Staging,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Dev => write!(f, "dev"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl Environment {
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Dev),
            "staging" => Ok(Self::Staging),
            "prod" | "production" => Ok(Self::Production),
            other => anyhow::bail!("unknown ENVIRONMENT value: {other}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    pub database_url: String,
    pub db_max_connections: u32,
    pub redis_url: String,
    pub auth_service_url: String,
    pub jwt_secret: String,
    pub r2_account_id: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_bucket: String,
    pub r2_public_url: String,
    pub rate_limit_per_minute: u32,
    pub max_room_members_free: u8,
    pub max_room_members_premium: u8,
    pub boost_tokens_per_month: u8,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let environment = Environment::from_str(
            &std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".into()),
        )?;

        let config = Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("PORT must be a valid u16"))?,
            environment,
            database_url: required_var("DATABASE_URL")?,
            db_max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("DB_MAX_CONNECTIONS must be a valid u32"))?,
            redis_url: required_var("REDIS_URL")?,
            auth_service_url: std::env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".into()),
            jwt_secret: required_var("JWT_SECRET")?,
            r2_account_id: std::env::var("R2_ACCOUNT_ID").unwrap_or_default(),
            r2_access_key: std::env::var("R2_ACCESS_KEY").unwrap_or_default(),
            r2_secret_key: std::env::var("R2_SECRET_KEY").unwrap_or_default(),
            r2_bucket: std::env::var("R2_BUCKET")
                .unwrap_or_else(|_| "placebo-media".into()),
            r2_public_url: std::env::var("R2_PUBLIC_URL")
                .unwrap_or_else(|_| "https://assets.placebo.tv".into()),
            rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("RATE_LIMIT_PER_MINUTE must be a valid u32"))?,
            max_room_members_free: std::env::var("MAX_ROOM_MEMBERS_FREE")
                .unwrap_or_else(|_| "4".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("MAX_ROOM_MEMBERS_FREE must be a valid u8"))?,
            max_room_members_premium: std::env::var("MAX_ROOM_MEMBERS_PREMIUM")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("MAX_ROOM_MEMBERS_PREMIUM must be a valid u8"))?,
            boost_tokens_per_month: std::env::var("BOOST_TOKENS_PER_MONTH")
                .unwrap_or_else(|_| "4".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("BOOST_TOKENS_PER_MONTH must be a valid u8"))?,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.jwt_secret.len() < 16 && self.environment == Environment::Production {
            anyhow::bail!("JWT_SECRET must be at least 16 characters in production");
        }
        if self.database_url.is_empty() {
            anyhow::bail!("DATABASE_URL cannot be empty");
        }
        if self.redis_url.is_empty() {
            anyhow::bail!("REDIS_URL cannot be empty");
        }
        if self.environment == Environment::Production {
            if self.r2_account_id.is_empty() {
                anyhow::bail!("R2_ACCOUNT_ID is required in production");
            }
            if self.r2_access_key.is_empty() {
                anyhow::bail!("R2_ACCESS_KEY is required in production");
            }
            if self.r2_secret_key.is_empty() {
                anyhow::bail!("R2_SECRET_KEY is required in production");
            }
        }
        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

fn required_var(name: &str) -> anyhow::Result<String> {
    std::env::var(name).map_err(|_| anyhow::anyhow!("{name} environment variable is required"))
}
