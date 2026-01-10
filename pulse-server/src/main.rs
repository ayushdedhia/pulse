use std::sync::Arc;

use pulse_server::{handle_connection, ServerState};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const DEFAULT_ADDR: &str = "0.0.0.0:9001";

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Get bind address from env or use default
    let addr = std::env::var("PULSE_SERVER_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());

    // Create server state
    let state = Arc::new(ServerState::new());

    // Bind TCP listener
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("Pulse server listening on {}", addr);

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!("New connection from {}", peer_addr);

                let state = state.clone();
                tokio::spawn(async move {
                    match accept_async(stream).await {
                        Ok(ws_stream) => {
                            handle_connection(ws_stream, state).await;
                        }
                        Err(e) => {
                            error!("WebSocket handshake failed for {}: {}", peer_addr, e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
