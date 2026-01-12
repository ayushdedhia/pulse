mod client;
mod messages;

pub use client::WebSocketClient;
pub use messages::{WsMessage, WsUrlPreview};

use std::sync::OnceLock;

// Global WebSocket client instance
static WS_CLIENT: OnceLock<WebSocketClient> = OnceLock::new();

pub fn get_ws_client() -> &'static WebSocketClient {
    WS_CLIENT.get_or_init(WebSocketClient::new)
}

/// Initialize WebSocket client and connect to server
pub async fn init_websocket(user_id: &str) -> Result<(), String> {
    let client = get_ws_client();
    client.connect(user_id).await
}
