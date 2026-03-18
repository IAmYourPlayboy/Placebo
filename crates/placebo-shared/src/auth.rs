use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Requests ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    /// BCP-47 locale tag, e.g. "ru", "en", "ja"
    pub locale: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

// ─── Responses ───────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_premium: bool,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub message: String,
}

// ─── Validation ──────────────────────────────────────────

impl RegisterRequest {
    /// Validate registration input. Returns list of error messages.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Email: basic format check
        let email = self.email.trim();
        if email.is_empty() {
            errors.push("Email is required".into());
        } else if !email.contains('@') || !email.contains('.') {
            errors.push("Invalid email format".into());
        } else if email.len() > 254 {
            errors.push("Email too long (max 254 chars)".into());
        }

        // Password: min 8 chars
        if self.password.len() < 8 {
            errors.push("Password must be at least 8 characters".into());
        }
        if self.password.len() > 128 {
            errors.push("Password too long (max 128 chars)".into());
        }

        // Display name: 1-50 chars
        let name = self.display_name.trim();
        if name.is_empty() {
            errors.push("Display name is required".into());
        } else if name.len() > 50 {
            errors.push("Display name too long (max 50 chars)".into());
        }

        errors
    }
}

impl LoginRequest {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.email.trim().is_empty() {
            errors.push("Email is required".into());
        }
        if self.password.is_empty() {
            errors.push("Password is required".into());
        }
        errors
    }
}

impl PasswordResetConfirm {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.token.trim().is_empty() {
            errors.push("Reset token is required".into());
        }
        if self.new_password.len() < 8 {
            errors.push("Password must be at least 8 characters".into());
        }
        if self.new_password.len() > 128 {
            errors.push("Password too long (max 128 chars)".into());
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_validates_email() {
        let req = RegisterRequest {
            email: "bad".into(),
            password: "12345678".into(),
            display_name: "Test".into(),
            locale: None,
        };
        let errs = req.validate();
        assert!(errs.iter().any(|e| e.contains("email")), "should reject bad email");
    }

    #[test]
    fn register_request_validates_short_password() {
        let req = RegisterRequest {
            email: "user@test.com".into(),
            password: "short".into(),
            display_name: "Test".into(),
            locale: None,
        };
        let errs = req.validate();
        assert!(errs.iter().any(|e| e.contains("8 characters")), "should reject short password");
    }

    #[test]
    fn register_request_passes_valid_input() {
        let req = RegisterRequest {
            email: "user@test.com".into(),
            password: "securepassword123".into(),
            display_name: "Test User".into(),
            locale: Some("ru".into()),
        };
        assert!(req.validate().is_empty(), "valid input should pass");
    }

    #[test]
    fn auth_response_serializes_camel_case() {
        let resp = AuthResponse {
            token: "abc".into(),
            user_id: uuid::Uuid::nil(),
            email: "a@b.com".into(),
            display_name: "A".into(),
            is_premium: false,
            expires_in_seconds: 3600,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("userId"), "should use camelCase: {json}");
        assert!(json.contains("expiresInSeconds"), "should use camelCase: {json}");
        assert!(json.contains("displayName"), "should use camelCase: {json}");
        // Ensure no sensitive data leaks
        assert!(!json.contains("password"), "must not contain password");
    }

    #[test]
    fn login_request_validates() {
        let req = LoginRequest {
            email: "".into(),
            password: "".into(),
        };
        let errs = req.validate();
        assert_eq!(errs.len(), 2);
    }
}
