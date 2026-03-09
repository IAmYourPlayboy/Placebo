use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::peer_connection::configuration::RTCConfiguration;

#[tauri::command]
async fn create_peer_connection() -> Result<String, String> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()
        .map_err(|e| e.to_string())?;

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .build();

    let config = RTCConfiguration::default();
    
    let peer_connection = api.new_peer_connection(config)
        .await
        .map_err(|e| e.to_string())?;

    Ok("PeerConnection created".to_string())
}
