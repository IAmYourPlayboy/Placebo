use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_premium: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub is_premium: bool,
    pub premium_until: Option<DateTime<Utc>>,
    pub cloud_used_bytes: i64,
    pub cloud_limit_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Response body for `GET /api/v1/me`. Returns the authenticated user's full profile,
/// including username and DOB visibility flag added in M2.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub is_premium: bool,
    pub premium_until: Option<DateTime<Utc>>,
    pub date_of_birth: Option<NaiveDate>,
    pub date_of_birth_hidden: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "export-types",
    derive(TS),
    ts(export, export_to = "../bindings/")
)]
#[serde(rename_all = "camelCase")]
pub struct BoostTokenInfo {
    pub camera_id: Uuid,
    pub camera_name: String,
    pub days_added: i16,
    pub applied_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_response_serde() {
        let now = Utc::now();
        let user = UserResponse {
            id: Uuid::new_v4(),
            display_name: "Bob".to_string(),
            avatar_url: None,
            is_premium: true,
            created_at: now,
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: UserResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.display_name, "Bob");
        assert_eq!(deserialized.is_premium, true);
        assert!(json.contains("\"isPremium\""));
    }

    #[test]
    fn user_profile_serde() {
        let now = Utc::now();
        let profile = UserProfile {
            id: Uuid::new_v4(),
            email: "bob@example.com".to_string(),
            display_name: "Bob".to_string(),
            avatar_url: Some("https://example.com/bob.jpg".to_string()),
            locale: "en".to_string(),
            is_premium: true,
            premium_until: Some(now),
            cloud_used_bytes: 5_000_000_000,
            cloud_limit_bytes: 50_000_000_000,
            created_at: now,
            updated_at: Some(now),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: UserProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.email, "bob@example.com");
        assert_eq!(deserialized.cloud_used_bytes, 5_000_000_000);
        assert!(json.contains("\"cloudUsedBytes\""));
        assert!(json.contains("\"premiumUntil\""));
    }

    #[test]
    fn me_response_serde() {
        let now = Utc::now();
        let me = MeResponse {
            id: Uuid::new_v4(),
            email: "alice@placebo.dev".into(),
            username: "alice".into(),
            display_name: "Alice".into(),
            avatar_url: None,
            locale: "ru".into(),
            is_premium: true,
            premium_until: Some(now),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 5, 17).unwrap()),
            date_of_birth_hidden: true,
            email_verified: false,
            created_at: now,
        };
        let json = serde_json::to_string(&me).unwrap();
        let back: MeResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.username, "alice");
        assert_eq!(back.date_of_birth_hidden, true);
        assert!(json.contains("\"dateOfBirth\""));
        assert!(json.contains("\"dateOfBirthHidden\""));
        assert!(json.contains("\"emailVerified\""));
    }

    #[test]
    fn boost_token_info_serde() {
        let now = Utc::now();
        let token = BoostTokenInfo {
            camera_id: Uuid::new_v4(),
            camera_name: "Tokyo Tower".to_string(),
            days_added: 30,
            applied_at: now,
            expires_at: now,
        };

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: BoostTokenInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.camera_name, "Tokyo Tower");
        assert_eq!(deserialized.days_added, 30);
        assert!(json.contains("\"daysAdded\""));
    }
}
