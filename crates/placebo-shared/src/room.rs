use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoomResponse {
    pub id: Uuid,
    pub name: String,
    pub camera_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub is_private: bool,
    pub max_members: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoomMemberResponse {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub joined_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_response_serde() {
        let now = Utc::now();
        let room = RoomResponse {
            id: Uuid::new_v4(),
            name: "Chill Tokyo".to_string(),
            camera_id: Some(Uuid::new_v4()),
            owner_id: Uuid::new_v4(),
            is_private: false,
            max_members: 50,
            created_at: now,
        };

        let json = serde_json::to_string(&room).unwrap();
        let deserialized: RoomResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Chill Tokyo");
        assert_eq!(deserialized.is_private, false);
        assert!(json.contains("\"isPrivate\""));
        assert!(json.contains("\"maxMembers\""));
    }

    #[test]
    fn room_member_response_serde() {
        let now = Utc::now();
        let member = RoomMemberResponse {
            user_id: Uuid::new_v4(),
            display_name: "Alice".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            joined_at: now,
        };

        let json = serde_json::to_string(&member).unwrap();
        let deserialized: RoomMemberResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.display_name, "Alice");
        assert!(json.contains("\"avatarUrl\""));
    }
}
