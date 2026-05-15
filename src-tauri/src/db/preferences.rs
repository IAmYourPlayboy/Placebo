use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub key: String,
    pub value: String,
}

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, String>("SELECT value FROM user_preferences WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_preferences (key, value, updated_at) \
         VALUES (?, ?, datetime('now')) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn all(pool: &SqlitePool) -> Result<Vec<Preference>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM user_preferences")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(k, v)| Preference { key: k, value: v }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE user_preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT)")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn set_then_get_returns_value() {
        let pool = test_pool().await;
        set(&pool, "theme", "dark").await.unwrap();
        assert_eq!(get(&pool, "theme").await.unwrap(), Some("dark".to_string()));
    }

    #[tokio::test]
    async fn set_overwrites_existing_key() {
        let pool = test_pool().await;
        set(&pool, "theme", "light").await.unwrap();
        set(&pool, "theme", "dark").await.unwrap();
        assert_eq!(get(&pool, "theme").await.unwrap(), Some("dark".to_string()));
    }

    #[tokio::test]
    async fn get_unknown_key_returns_none() {
        let pool = test_pool().await;
        assert_eq!(get(&pool, "nope").await.unwrap(), None);
    }
}
