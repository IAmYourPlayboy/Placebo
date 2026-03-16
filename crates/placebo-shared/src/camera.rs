use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum CameraType {
    Public,
    Enterprise,
    Yourself,
}

impl fmt::Display for CameraType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Enterprise => write!(f, "enterprise"),
            Self::Yourself => write!(f, "yourself"),
        }
    }
}

impl FromStr for CameraType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Self::Public),
            "enterprise" => Ok(Self::Enterprise),
            "yourself" => Ok(Self::Yourself),
            _ => Err(format!("unknown CameraType: {s}")),
        }
    }
}

impl TryFrom<&str> for CameraType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RetentionTier {
    Tier1,
    Tier2,
    Tier3,
    Tier4,
    Tier5,
}

impl fmt::Display for RetentionTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tier1 => write!(f, "tier1"),
            Self::Tier2 => write!(f, "tier2"),
            Self::Tier3 => write!(f, "tier3"),
            Self::Tier4 => write!(f, "tier4"),
            Self::Tier5 => write!(f, "tier5"),
        }
    }
}

impl FromStr for RetentionTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tier1" => Ok(Self::Tier1),
            "tier2" => Ok(Self::Tier2),
            "tier3" => Ok(Self::Tier3),
            "tier4" => Ok(Self::Tier4),
            "tier5" => Ok(Self::Tier5),
            _ => Err(format!("unknown RetentionTier: {s}")),
        }
    }
}

impl TryFrom<&str> for RetentionTier {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Rtsp,
    Hls,
    Youtube,
    Dash,
    Webrtc,
    Mjpeg,
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rtsp => write!(f, "rtsp"),
            Self::Hls => write!(f, "hls"),
            Self::Youtube => write!(f, "youtube"),
            Self::Dash => write!(f, "dash"),
            Self::Webrtc => write!(f, "webrtc"),
            Self::Mjpeg => write!(f, "mjpeg"),
        }
    }
}

impl FromStr for StreamType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rtsp" => Ok(Self::Rtsp),
            "hls" => Ok(Self::Hls),
            "youtube" => Ok(Self::Youtube),
            "dash" => Ok(Self::Dash),
            "webrtc" => Ok(Self::Webrtc),
            "mjpeg" => Ok(Self::Mjpeg),
            _ => Err(format!("unknown StreamType: {s}")),
        }
    }
}

impl TryFrom<&str> for StreamType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StreamProtocol {
    Tcp,
    Udp,
    Https,
}

impl fmt::Display for StreamProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp => write!(f, "tcp"),
            Self::Udp => write!(f, "udp"),
            Self::Https => write!(f, "https"),
        }
    }
}

impl FromStr for StreamProtocol {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(Self::Tcp),
            "udp" => Ok(Self::Udp),
            "https" => Ok(Self::Https),
            _ => Err(format!("unknown StreamProtocol: {s}")),
        }
    }
}

impl TryFrom<&str> for StreamProtocol {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
    Vp9,
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::H264 => write!(f, "h264"),
            Self::H265 => write!(f, "h265"),
            Self::Av1 => write!(f, "av1"),
            Self::Vp9 => write!(f, "vp9"),
        }
    }
}

impl FromStr for VideoCodec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "h264" => Ok(Self::H264),
            "h265" => Ok(Self::H265),
            "av1" => Ok(Self::Av1),
            "vp9" => Ok(Self::Vp9),
            _ => Err(format!("unknown VideoCodec: {s}")),
        }
    }
}

impl TryFrom<&str> for VideoCodec {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    City,
    Traffic,
    Nature,
    Harbor,
    Airport,
    Beach,
    Mountain,
    Underwater,
    Space,
    Weather,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::City => write!(f, "city"),
            Self::Traffic => write!(f, "traffic"),
            Self::Nature => write!(f, "nature"),
            Self::Harbor => write!(f, "harbor"),
            Self::Airport => write!(f, "airport"),
            Self::Beach => write!(f, "beach"),
            Self::Mountain => write!(f, "mountain"),
            Self::Underwater => write!(f, "underwater"),
            Self::Space => write!(f, "space"),
            Self::Weather => write!(f, "weather"),
        }
    }
}

impl FromStr for Category {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "city" => Ok(Self::City),
            "traffic" => Ok(Self::Traffic),
            "nature" => Ok(Self::Nature),
            "harbor" => Ok(Self::Harbor),
            "airport" => Ok(Self::Airport),
            "beach" => Ok(Self::Beach),
            "mountain" => Ok(Self::Mountain),
            "underwater" => Ok(Self::Underwater),
            "space" => Ok(Self::Space),
            "weather" => Ok(Self::Weather),
            _ => Err(format!("unknown Category: {s}")),
        }
    }
}

impl TryFrom<&str> for Category {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------
// CameraResponse – public API shape
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CameraResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub camera_type: CameraType,

    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub address: Option<String>,
    pub custom_label: Option<String>,

    pub lat: f64,
    pub lng: f64,
    pub timezone: Option<String>,

    pub stream_type: Option<StreamType>,
    pub stream_protocol: Option<StreamProtocol>,
    pub stream_quality_default: Option<String>,
    pub available_qualities: Vec<String>,

    pub bitrate_kbps: Option<i32>,
    pub codec: Option<VideoCodec>,
    pub resolution_w: Option<i32>,
    pub resolution_h: Option<i32>,
    pub latency_ms: Option<i32>,

    pub has_audio: bool,
    pub has_night_vision: bool,
    pub is_underwater: bool,

    pub category: Category,
    pub subcategory: Option<String>,
    pub tags: Vec<String>,

    pub description_en: Option<String>,
    pub thumbnail_url: Option<String>,
    pub source_url: Option<String>,
    pub attribution: Option<String>,

    pub recording_enabled: bool,
    pub retention_tier: RetentionTier,
    pub recording_retention_days: i32,
    pub recording_codec: VideoCodec,

    pub height_above_ground: Option<f64>,
    pub camera_azimuth: Option<f64>,
    pub camera_elevation: Option<f64>,
    pub fov_horizontal: Option<f64>,
    pub fov_vertical: Option<f64>,

    pub manufacturer: Option<String>,
    pub camera_model: Option<String>,

    pub added_to_placebo_at: Option<DateTime<Utc>>,

    pub is_partner_camera: bool,
    pub owner_name: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_type_display_roundtrip() {
        for variant in [CameraType::Public, CameraType::Enterprise, CameraType::Yourself] {
            let s = variant.to_string();
            let parsed: CameraType = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn retention_tier_display_roundtrip() {
        for variant in [
            RetentionTier::Tier1,
            RetentionTier::Tier2,
            RetentionTier::Tier3,
            RetentionTier::Tier4,
            RetentionTier::Tier5,
        ] {
            let s = variant.to_string();
            let parsed: RetentionTier = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn stream_type_display_roundtrip() {
        for variant in [
            StreamType::Rtsp,
            StreamType::Hls,
            StreamType::Youtube,
            StreamType::Dash,
            StreamType::Webrtc,
            StreamType::Mjpeg,
        ] {
            let s = variant.to_string();
            let parsed: StreamType = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn stream_protocol_display_roundtrip() {
        for variant in [StreamProtocol::Tcp, StreamProtocol::Udp, StreamProtocol::Https] {
            let s = variant.to_string();
            let parsed: StreamProtocol = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn video_codec_display_roundtrip() {
        for variant in [VideoCodec::H264, VideoCodec::H265, VideoCodec::Av1, VideoCodec::Vp9] {
            let s = variant.to_string();
            let parsed: VideoCodec = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn category_display_roundtrip() {
        for variant in [
            Category::City,
            Category::Traffic,
            Category::Nature,
            Category::Harbor,
            Category::Airport,
            Category::Beach,
            Category::Mountain,
            Category::Underwater,
            Category::Space,
            Category::Weather,
        ] {
            let s = variant.to_string();
            let parsed: Category = s.parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn camera_type_try_from_str() {
        assert_eq!(CameraType::try_from("public"), Ok(CameraType::Public));
        assert!(CameraType::try_from("invalid").is_err());
    }

    #[test]
    fn camera_response_serde_roundtrip() {
        let now = Utc::now();
        let resp = CameraResponse {
            id: Uuid::new_v4(),
            name: "Tokyo Tower Cam".to_string(),
            slug: "tokyo-tower-cam".to_string(),
            camera_type: CameraType::Public,
            country: Some("Japan".to_string()),
            country_code: Some("JP".to_string()),
            region: Some("Kanto".to_string()),
            city: Some("Tokyo".to_string()),
            district: Some("Minato".to_string()),
            address: None,
            custom_label: None,
            lat: 35.6586,
            lng: 139.7454,
            timezone: Some("Asia/Tokyo".to_string()),
            stream_type: Some(StreamType::Hls),
            stream_protocol: Some(StreamProtocol::Https),
            stream_quality_default: Some("1080p".to_string()),
            available_qualities: vec!["720p".to_string(), "1080p".to_string()],
            bitrate_kbps: Some(4000),
            codec: Some(VideoCodec::H264),
            resolution_w: Some(1920),
            resolution_h: Some(1080),
            latency_ms: Some(3000),
            has_audio: false,
            has_night_vision: true,
            is_underwater: false,
            category: Category::City,
            subcategory: Some("landmark".to_string()),
            tags: vec!["tokyo".to_string(), "tower".to_string()],
            description_en: Some("Live view of Tokyo Tower".to_string()),
            thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
            source_url: Some("https://example.com".to_string()),
            attribution: Some("Example Corp".to_string()),
            recording_enabled: true,
            retention_tier: RetentionTier::Tier2,
            recording_retention_days: 30,
            recording_codec: VideoCodec::H264,
            height_above_ground: Some(150.0),
            camera_azimuth: Some(180.0),
            camera_elevation: Some(-15.0),
            fov_horizontal: Some(90.0),
            fov_vertical: Some(60.0),
            manufacturer: Some("Hikvision".to_string()),
            camera_model: Some("DS-2CD2T85G1".to_string()),
            added_to_placebo_at: Some(now),
            is_partner_camera: false,
            owner_name: None,
            created_at: now,
            updated_at: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: CameraResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Tokyo Tower Cam");
        assert_eq!(deserialized.camera_type, CameraType::Public);
        assert_eq!(deserialized.category, Category::City);
        assert_eq!(deserialized.has_audio, false);
        assert_eq!(deserialized.recording_enabled, true);
        assert_eq!(deserialized.tags.len(), 2);

        // Verify camelCase serialization
        assert!(json.contains("\"cameraType\""));
        assert!(json.contains("\"hasAudio\""));
        assert!(json.contains("\"streamType\""));
        assert!(json.contains("\"retentionTier\""));
        assert!(json.contains("\"recordingRetentionDays\""));
        assert!(json.contains("\"isPartnerCamera\""));

        // Verify sensitive fields are NOT present
        assert!(!json.contains("stream_url"));
        assert!(!json.contains("streamUrl"));
        assert!(!json.contains("backup_url"));
        assert!(!json.contains("backupUrl"));
        assert!(!json.contains("external_id"));
        assert!(!json.contains("externalId"));
        assert!(!json.contains("frame_rate"));
        assert!(!json.contains("frameRate"));
    }

    #[test]
    fn enum_serde_lowercase() {
        let json = serde_json::to_string(&CameraType::Enterprise).unwrap();
        assert_eq!(json, "\"enterprise\"");

        let json = serde_json::to_string(&StreamType::Youtube).unwrap();
        assert_eq!(json, "\"youtube\"");

        let json = serde_json::to_string(&VideoCodec::H265).unwrap();
        assert_eq!(json, "\"h265\"");

        let json = serde_json::to_string(&Category::Underwater).unwrap();
        assert_eq!(json, "\"underwater\"");
    }
}
