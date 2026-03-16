// Placebo Camera types – mirrors Rust db::camera::Camera struct
// All fields use camelCase (Rust struct has #[serde(rename_all = "camelCase")])

export interface Camera {
  id: string;
  name: string;
  slug: string;
  cameraType: 'public' | 'enterprise' | 'yourself';
  externalId: string | null;

  // Location
  country: string | null;
  countryCode: string | null;
  region: string | null;
  city: string | null;
  district: string | null;
  address: string | null;
  customLabel: string | null;
  lat: number;
  lng: number;
  timezone: string | null;

  // Stream
  streamUrl: string;
  backupUrl: string | null;
  streamType: 'rtsp' | 'hls' | 'youtube' | 'dash' | 'webrtc' | 'mjpeg' | null;
  streamProtocol: 'tcp' | 'udp' | 'https' | null;
  streamQualityDefault: string | null;
  availableQualities: string | null;
  frameRate: number | null;
  bitrateKbps: number | null;
  codec: 'h264' | 'h265' | 'av1' | 'vp9' | null;
  resolutionW: number | null;
  resolutionH: number | null;
  latencyMs: number | null;

  // Capabilities
  hasAudio: number;
  hasNightVision: number;
  isUnderwater: number;

  // Meta
  category: string;
  subcategory: string | null;
  tags: string;
  descriptionEn: string | null;
  thumbnailUrl: string | null;
  sourceUrl: string | null;
  attribution: string | null;

  // Recording
  recordingEnabled: number;
  retentionTier: 'tier1' | 'tier2' | 'tier3' | 'tier4' | 'tier5';
  recordingRetentionDays: number;
  recordingCodec: string;

  // Partner / Hardware
  manufacturer: string | null;
  cameraModel: string | null;
  addedToPlaceboAt: string | null;
  isPartnerCamera: number;
  ownerName: string | null;

  // Timestamps
  createdAt: string;
  updatedAt: string | null;
}

export type CameraCategory =
  | 'city'
  | 'traffic'
  | 'nature'
  | 'harbor'
  | 'airport'
  | 'beach'
  | 'mountain'
  | 'weather'
  | 'construction';
