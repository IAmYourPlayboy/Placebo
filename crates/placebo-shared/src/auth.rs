use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

// ─── Requests ────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    /// Desired username. If `None`, the server generates one from `display_name`.
    pub username: Option<String>,
    /// Optional ISO-8601 date (yyyy-mm-dd).
    pub date_of_birth: Option<NaiveDate>,
    /// Hide DOB on the public profile. Defaults to `true` server-side when omitted.
    pub date_of_birth_hidden: Option<bool>,
    /// BCP-47 locale tag, e.g. "ru".
    pub locale: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

// ─── Responses ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub is_premium: bool,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub message: String,
}

/// Returned in the body of a 409 when a requested username is taken.
/// Allows the client to render "Try one of these" chips without a second round-trip.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct UsernameTakenError {
    pub error: String,
    pub message: String,
    pub suggestions: Vec<String>,
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

        // Password: 8-128 chars
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

        // Username: optional on the wire (server generates one if absent), but if provided
        // it must be 3-24 chars of [A-Za-z0-9_], not starting/ending with `_`.
        // An empty/whitespace string is treated as "not provided" – we'll let the server
        // generate one rather than rejecting the request.
        if let Some(u) = &self.username {
            let u = u.trim();
            if !u.is_empty() {
                if u.len() < 3 {
                    errors.push("Username must be at least 3 characters".into());
                } else if u.len() > 24 {
                    errors.push("Username too long (max 24 chars)".into());
                } else if !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    errors
                        .push("Username may contain only Latin letters, digits, and underscore".into());
                } else if u.starts_with('_') || u.ends_with('_') {
                    errors.push("Username must not start or end with underscore".into());
                }
            }
        }

        // Date of birth sanity: must be in the past, age <= 120 years.
        if let Some(dob) = self.date_of_birth {
            let today = chrono::Utc::now().date_naive();
            if dob > today {
                errors.push("Date of birth cannot be in the future".into());
            } else if today.years_since(dob).unwrap_or(0) > 120 {
                errors.push("Date of birth looks invalid".into());
            }
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

    fn baseline() -> RegisterRequest {
        RegisterRequest {
            email: "user@test.com".into(),
            password: "securepassword123".into(),
            display_name: "Test User".into(),
            username: None,
            date_of_birth: None,
            date_of_birth_hidden: None,
            locale: None,
        }
    }

    #[test]
    fn register_request_validates_email() {
        let mut req = baseline();
        req.email = "bad".into();
        let errs = req.validate();
        assert!(
            errs.iter().any(|e| e.to_lowercase().contains("email")),
            "should reject bad email: {errs:?}"
        );
    }

    #[test]
    fn register_request_validates_short_password() {
        let mut req = baseline();
        req.password = "short".into();
        let errs = req.validate();
        assert!(
            errs.iter().any(|e| e.contains("8 characters")),
            "should reject short password"
        );
    }

    #[test]
    fn register_request_passes_valid_input() {
        let req = baseline();
        assert!(req.validate().is_empty(), "valid input should pass");
    }

    #[test]
    fn register_request_rejects_short_username() {
        let mut req = baseline();
        req.username = Some("ab".into());
        assert!(req.validate().iter().any(|e| e.contains("3 characters")));
    }

    #[test]
    fn register_request_rejects_username_with_space() {
        let mut req = baseline();
        req.username = Some("has space".into());
        assert!(req.validate().iter().any(|e| e.contains("Latin")));
    }

    #[test]
    fn register_request_rejects_username_edges() {
        let mut req = baseline();
        req.username = Some("_leading".into());
        assert!(req.validate().iter().any(|e| e.contains("underscore")));
        req.username = Some("trailing_".into());
        assert!(req.validate().iter().any(|e| e.contains("underscore")));
    }

    #[test]
    fn register_request_accepts_valid_username_and_dob() {
        let mut req = baseline();
        req.username = Some("cool_user123".into());
        req.date_of_birth = Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap());
        req.date_of_birth_hidden = Some(true);
        assert!(req.validate().is_empty(), "errors: {:?}", req.validate());
    }

    #[test]
    fn register_request_treats_empty_username_as_unset() {
        let mut req = baseline();
        req.username = Some("   ".into());
        // whitespace-only is normalised to "not provided" – no error, server will generate one
        assert!(req.validate().is_empty());
    }

    #[test]
    fn register_request_rejects_future_dob() {
        let mut req = baseline();
        req.date_of_birth = Some(chrono::Utc::now().date_naive() + chrono::Duration::days(1));
        assert!(req.validate().iter().any(|e| e.contains("future")));
    }

    #[test]
    fn register_request_rejects_implausible_age() {
        let mut req = baseline();
        req.date_of_birth = Some(NaiveDate::from_ymd_opt(1800, 1, 1).unwrap());
        assert!(req.validate().iter().any(|e| e.contains("invalid")));
    }

    #[test]
    fn auth_response_serializes_camel_case() {
        let resp = AuthResponse {
            token: "abc".into(),
            user_id: uuid::Uuid::nil(),
            email: "a@b.com".into(),
            username: "alice".into(),
            display_name: "A".into(),
            is_premium: false,
            expires_in_seconds: 3600,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("userId"), "should use camelCase: {json}");
        assert!(json.contains("expiresInSeconds"), "should use camelCase: {json}");
        assert!(json.contains("displayName"), "should use camelCase: {json}");
        assert!(json.contains("\"username\":\"alice\""), "username present: {json}");
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
