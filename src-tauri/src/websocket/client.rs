use super::messages::WsMessage;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::{broadcast, mpsc, Mutex as TokioMutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Server URL: checked at compile time via env!, falls back to runtime env var, then default
const DEFAULT_SERVER_URL: &str = "ws://localhost:9001";

/// Internal message type for the write channel
enum WriteMessage {
    Data(String),
    Close,
}

/// WebSocket client that connects to the central Pulse server
pub struct WebSocketClient {
    server_url: Arc<TokioMutex<String>>,
    /// Use std::sync::Mutex for write_tx so it can be accessed from sync Tauri commands
    write_tx: Arc<StdMutex<Option<mpsc::UnboundedSender<WriteMessage>>>>,
    connected: Arc<TokioMutex<bool>>,
    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketClient {
    pub fn new() -> Self {
        // Priority: build-time env -> runtime env -> default
        let build_time_url = option_env!("PULSE_SERVER_URL");
        let runtime_url = std::env::var("PULSE_SERVER_URL").ok();

        info!(
            build_time = ?build_time_url,
            runtime = ?runtime_url,
            "WebSocket URL sources"
        );

        let server_url = build_time_url
            .map(String::from)
            .or(runtime_url)
            .unwrap_or_else(|| DEFAULT_SERVER_URL.to_string());

        info!(url = %server_url, "Using WebSocket server URL");

        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            server_url: Arc::new(TokioMutex::new(server_url)),
            write_tx: Arc::new(StdMutex::new(None)),
            connected: Arc::new(TokioMutex::new(false)),
            shutdown_tx,
        }
    }

    /// Get the server URL
    pub async fn get_server_url(&self) -> String {
        self.server_url.lock().await.clone()
    }

    /// Check if connected to server
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Connect to the central server
    pub async fn connect(&self, user_id: &str) -> Result<(), String> {
        let server_url = self.server_url.lock().await.clone();
        let user_id = user_id.to_string();
        let write_tx = self.write_tx.clone();
        let connected = self.connected.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                // Check for shutdown before attempting connection
                if shutdown_rx.try_recv().is_ok() {
                    info!("Shutdown signal received, stopping reconnection");
                    break;
                }

                info!(url = %server_url, "Connecting to Pulse server");

                match connect_async(&server_url).await {
                    Ok((ws_stream, _)) => {
                        info!("Connected to Pulse server");
                        *connected.lock().await = true;

                        let (mut ws_write, mut ws_read) = ws_stream.split();

                        // Send Connect message
                        let connect_msg = WsMessage::Connect {
                            user_id: user_id.clone(),
                        };
                        let connect_json = serde_json::to_string(&connect_msg).unwrap();

                        if ws_write.send(Message::Text(connect_json.into())).await.is_err() {
                            error!("Failed to send connect message");
                            *connected.lock().await = false;
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            continue;
                        }

                        // Wait for auth response
                        if let Some(Ok(Message::Text(response))) = ws_read.next().await {
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&response) {
                                match msg {
                                    WsMessage::AuthResponse { success, message } => {
                                        if success {
                                            info!("Authenticated with server: {}", message);
                                        } else {
                                            error!("Authentication failed: {}", message);
                                            *connected.lock().await = false;
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                            continue;
                                        }
                                    }
                                    _ => {
                                        warn!("Unexpected response during auth");
                                    }
                                }
                            }
                        }

                        // Create channel for outgoing messages
                        let (tx, mut rx) = mpsc::unbounded_channel::<WriteMessage>();
                        {
                            let mut guard = write_tx.lock().unwrap();
                            *guard = Some(tx);
                        }

                        // Message loop
                        let mut should_reconnect = true;
                        loop {
                            tokio::select! {
                                // Check for shutdown signal
                                _ = shutdown_rx.recv() => {
                                    info!("Shutdown signal received, closing connection gracefully");
                                    // Send close frame
                                    if let Err(e) = ws_write.send(Message::Close(None)).await {
                                        warn!(error = %e, "Failed to send close frame");
                                    }
                                    should_reconnect = false;
                                    break;
                                }
                                // Send outgoing messages
                                Some(msg) = rx.recv() => {
                                    match msg {
                                        WriteMessage::Data(data) => {
                                            if ws_write.send(Message::Text(data.into())).await.is_err() {
                                                error!("Failed to send message to server");
                                                break;
                                            }
                                        }
                                        WriteMessage::Close => {
                                            info!("Close requested, sending close frame");
                                            if let Err(e) = ws_write.send(Message::Close(None)).await {
                                                warn!(error = %e, "Failed to send close frame");
                                            }
                                            should_reconnect = false;
                                            break;
                                        }
                                    }
                                }
                                // Receive incoming messages (frontend handles these directly via its own WS connection)
                                msg = ws_read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            debug!(preview = %&text[..100.min(text.len())], "Received from server");
                                        }
                                        Some(Ok(Message::Close(_))) | None => {
                                            info!("Server closed connection");
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            error!(error = %e, "WebSocket error");
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // Cleanup
                        {
                            let mut guard = write_tx.lock().unwrap();
                            *guard = None;
                        }
                        *connected.lock().await = false;
                        info!("Disconnected from Pulse server");

                        if !should_reconnect {
                            break;
                        }
                    }
                    Err(e) => {
                        error!(error = %e, url = %server_url, "Failed to connect to Pulse server");
                    }
                }

                // Reconnect after delay
                debug!("Reconnecting in 3 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        });

        Ok(())
    }

    /// Gracefully disconnect from the server
    pub fn disconnect(&self) {
        info!("Initiating graceful disconnect");
        // Signal shutdown to stop reconnection loop
        let _ = self.shutdown_tx.send(());
        // Also send close message through the channel if connected
        if let Ok(guard) = self.write_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(WriteMessage::Close);
            }
        }
    }

    /// Send a message to the server
    pub fn send(&self, message: WsMessage) -> Result<(), String> {
        let json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
        debug!(preview = %&json[..100.min(json.len())], "Sending message to server");

        // Use blocking lock since this is called from sync Tauri commands
        let guard = self.write_tx.lock().map_err(|e| format!("Lock poisoned: {}", e))?;

        if let Some(tx) = guard.as_ref() {
            tx.send(WriteMessage::Data(json)).map_err(|e| format!("Failed to send to server: {}", e))?;
            Ok(())
        } else {
            // Not connected to server - log warning but don't fail
            // Message will be lost but we don't want to block the UI
            warn!("Cannot send message: not connected to server");
            Err("Not connected to server".to_string())
        }
    }

    /// Broadcast a message (sends to server which relays to all clients)
    pub fn broadcast(&self, message: WsMessage) -> Result<(), String> {
        self.send(message)
    }
}
