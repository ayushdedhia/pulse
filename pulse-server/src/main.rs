use std::sync::Arc;

use pulse_server::{handle_connection, ServerState};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const DEFAULT_PORT: u16 = 9001;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Get bind address: PULSE_SERVER_ADDR takes priority, then PORT (Railway), then default
    let addr = if let Ok(addr) = std::env::var("PULSE_SERVER_ADDR") {
        addr
    } else {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        format!("0.0.0.0:{}", port)
    };

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

    // Accept connections with graceful shutdown
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal, closing server...");
                break;
            }
            result = listener.accept() => {
                match result {
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
    }

    info!("Server shutdown complete");
}
