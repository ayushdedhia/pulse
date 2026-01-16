use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

const METERED_DOMAIN: &str = "pulse-app.metered.live";
const FETCH_TIMEOUT_SECS: u64 = 5;

/// ICE server configuration returned to frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IceServer {
    pub urls: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

/// Response from Metered API
#[derive(Debug, Deserialize)]
struct MeteredIceServer {
    urls: String,
    username: Option<String>,
    credential: Option<String>,
}

/// Fallback STUN servers (Google's public STUN)
fn get_fallback_servers() -> Vec<IceServer> {
    vec![
        IceServer {
            urls: "stun:stun.l.google.com:19302".to_string(),
            username: None,
            credential: None,
        },
        IceServer {
            urls: "stun:stun1.l.google.com:19302".to_string(),
            username: None,
            credential: None,
        },
    ]
}

/// Fetch TURN credentials from Metered.ca API
#[tauri::command]
pub async fn get_turn_credentials() -> Result<Vec<IceServer>, String> {
    // Get API key from environment
    let api_key = match env::var("METERED_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        Ok(_) => {
            tracing::warn!("METERED_API_KEY is empty, using fallback STUN servers");
            return Ok(get_fallback_servers());
        }
        Err(_) => {
            tracing::warn!("METERED_API_KEY not set, using fallback STUN servers");
            return Ok(get_fallback_servers());
        }
    };

    let url = format!(
        "https://{}/api/v1/turn/credentials?apiKey={}",
        METERED_DOMAIN, api_key
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| e.to_string())?;

    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!("Failed to fetch TURN credentials: {}, using fallback", e);
            return Ok(get_fallback_servers());
        }
    };

    if !response.status().is_success() {
        tracing::warn!(
            "Metered API returned {}, using fallback",
            response.status()
        );
        return Ok(get_fallback_servers());
    }

    let servers: Vec<MeteredIceServer> = match response.json().await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to parse TURN credentials: {}, using fallback", e);
            return Ok(get_fallback_servers());
        }
    };

    // Convert to our IceServer type
    let ice_servers: Vec<IceServer> = servers
        .into_iter()
        .map(|s| IceServer {
            urls: s.urls,
            username: s.username,
            credential: s.credential,
        })
        .collect();

    if ice_servers.is_empty() {
        tracing::warn!("Metered returned empty servers, using fallback");
        return Ok(get_fallback_servers());
    }

    tracing::info!("Fetched {} ICE servers from Metered", ice_servers.len());
    Ok(ice_servers)
}
