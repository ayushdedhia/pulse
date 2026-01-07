mod handlers;
mod messages;
mod server;

pub use messages::WsMessage;
pub use server::WebSocketServer;

use std::sync::OnceLock;

// Global WebSocket server instance
static WS_SERVER: OnceLock<WebSocketServer> = OnceLock::new();

pub fn get_ws_server() -> &'static WebSocketServer {
   WS_SERVER.get_or_init(WebSocketServer::new)
}

pub async fn init_websocket_server() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
   let server = get_ws_server();
   server.start("127.0.0.1:9001").await
}
