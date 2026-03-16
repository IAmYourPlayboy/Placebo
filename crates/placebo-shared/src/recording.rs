use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::camera::VideoCodec;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StorageTier {
    Hot,
    Warm,
    Cold,
    Archive,
}

impl fmt::Display for StorageTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hot => write!(f, "hot"),
            Self::Warm => write!(f, "warm"),
            Self::Cold => write!(f, "cold"),
            Self::Archive => write!(f, "archive"),
        }
    }
}

impl FromStr for StorageTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hot" => Ok(Self::Hot),
            "warm" => Ok(Self::Warm),
            "cold" => Ok(Self::Cold),
            "archive" => Ok(Self::Archive),
            _ => Err(format!("unknown StorageTier: {s}")),
        }
    }
}

impl TryFrom<&str> for StorageTier {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ClipStatus {
    Pending,
    Processing,
    Ready,
    Failed,
}

impl fmt::Display for ClipStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Processing => write!(f, "processing"),
            Self::Ready => write!(f, "ready"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for ClipStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "ready" => Ok(Self::Ready),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unknown ClipStatus: {s}")),
        }
    }
}

impl TryFrom<&str> for ClipStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSegmentResponse {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: i32,
    pub storage_tier: StorageTier,
    pub codec: VideoCodec,
    pub file_size_bytes: i64,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipResponse {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: ClipStatus,
    pub output_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_tier_display_roundtrip() {
        for variant in [StorageTier::Hot, StorageTier::Warm, StorageTier::Cold, StorageTier::Archive] {
            let s = variant.to_string();
            let parsed: StorageTier = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn clip_status_display_roundtrip() {
        for variant in [
            ClipStatus::Pending,
            ClipStatus::Processing,
            ClipStatus::Ready,
            ClipStatus::Failed,
        ] {
            let s = variant.to_string();
            let parsed: ClipStatus = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn recording_segment_serde() {
        let now = Utc::now();
        let seg = RecordingSegmentResponse {
            id: Uuid::new_v4(),
            camera_id: Uuid::new_v4(),
            start_time: now,
            end_time: now,
            duration_seconds: 3600,
            storage_tier: StorageTier::Hot,
            codec: VideoCodec::H264,
            file_size_bytes: 1_500_000_000,
            resolution: Some("1920x1080".to_string()),
            created_at: now,
        };

        let json = serde_json::to_string(&seg).unwrap();
        let deserialized: RecordingSegmentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.duration_seconds, 3600);
        assert_eq!(deserialized.storage_tier, StorageTier::Hot);

        // file_path must NOT be present
        assert!(!json.contains("filePath"));
        assert!(!json.contains("file_path"));
    }

    #[test]
    fn clip_response_serde() {
        let now = Utc::now();
        let clip = ClipResponse {
            id: Uuid::new_v4(),
            camera_id: Uuid::new_v4(),
            start_time: now,
            end_time: now,
            status: ClipStatus::Ready,
            output_url: Some("https://r2.example.com/clip.mp4".to_string()),
            created_at: now,
            completed_at: Some(now),
        };

        let json = serde_json::to_string(&clip).unwrap();
        assert!(json.contains("\"status\":\"ready\""));
        assert!(json.contains("\"outputUrl\""));
    }
}
