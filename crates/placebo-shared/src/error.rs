use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum SharedError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error("rate limited: retry after {retry_after}s")]
    RateLimited { retry_after: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = SharedError::NotFound("camera 123".to_string());
        assert_eq!(err.to_string(), "not found: camera 123");

        let err = SharedError::RateLimited { retry_after: 30 };
        assert_eq!(err.to_string(), "rate limited: retry after 30s");
    }

    #[test]
    fn error_serde_roundtrip() {
        let err = SharedError::Validation("email is required".to_string());
        let json = serde_json::to_string(&err).unwrap();
        let deserialized: SharedError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.to_string(), "validation: email is required");
    }
}
