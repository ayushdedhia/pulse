use super::messages::WsMessage;
use super::{add_trusted_token, validate_token};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Maximum message size in bytes (64KB)
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    broadcast_tx: broadcast::Sender<String>,
) {
    info!(peer = %addr, "New WebSocket connection");

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!(peer = %addr, error = %e, "WebSocket handshake failed");
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let mut broadcast_rx = broadcast_tx.subscribe();
    let mut user_id: Option<String> = None;
    let mut is_authenticated = false;

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Validate message size
                        if text.len() > MAX_MESSAGE_SIZE {
                            warn!(peer = %addr, size = text.len(), "Message too large");
                            let error = WsMessage::Error {
                                message: "Message too large".to_string(),
                            };
                            let _ = write.send(Message::Text(serde_json::to_string(&error).unwrap())).await;
                            continue;
                        }

                        debug!(peer = %addr, preview = %&text[..100.min(text.len())], "Received message");
                        match serde_json::from_str::<WsMessage>(&text) {
                            Ok(ws_msg) => {
                                match ws_msg {
                                    WsMessage::Connect { user_id: uid, auth_token } => {
                                        // Validate authentication token
                                        let token_valid = auth_token
                                            .as_ref()
                                            .map(|t| validate_token(t))
                                            .unwrap_or(false);

                                        if !token_valid {
                                            warn!(user_id = %uid, peer = %addr, "Authentication failed");
                                            let response = WsMessage::AuthResponse {
                                                success: false,
                                                message: "Invalid or missing authentication token".to_string(),
                                            };
                                            let _ = write.send(Message::Text(serde_json::to_string(&response).unwrap())).await;
                                            // Don't disconnect immediately - give client a chance to retry
                                            // But don't allow them to send messages
                                            continue;
                                        }

                                        is_authenticated = true;
                                        user_id = Some(uid.clone());
                                        info!(user_id = %uid, peer = %addr, "User authenticated");

                                        // Send auth success response
                                        let response = WsMessage::AuthResponse {
                                            success: true,
                                            message: "Authenticated successfully".to_string(),
                                        };
                                        let _ = write.send(Message::Text(serde_json::to_string(&response).unwrap())).await;

                                        // Broadcast presence
                                        let presence = WsMessage::Presence {
                                            user_id: uid,
                                            is_online: true,
                                            last_seen: None,
                                        };
                                        let _ = broadcast_tx.send(serde_json::to_string(&presence).unwrap());
                                    }
                                    WsMessage::PeerConnect { peer_token } => {
                                        // Peer-to-peer token exchange - trust the peer's token
                                        info!(peer = %addr, "Peer token exchange request");
                                        add_trusted_token(&peer_token);
                                        is_authenticated = true;
                                        user_id = Some(format!("peer_{}", addr));
                                        info!(peer = %addr, "Peer authenticated via token exchange");

                                        let response = WsMessage::AuthResponse {
                                            success: true,
                                            message: "Peer token accepted".to_string(),
                                        };
                                        let _ = write.send(Message::Text(serde_json::to_string(&response).unwrap())).await;
                                    }
                                    _ => {
                                        // Only allow broadcasting if authenticated
                                        if !is_authenticated {
                                            warn!(peer = %addr, "Unauthenticated client tried to send message");
                                            let error = WsMessage::Error {
                                                message: "Not authenticated. Send Connect message with auth_token first.".to_string(),
                                            };
                                            let _ = write.send(Message::Text(serde_json::to_string(&error).unwrap())).await;
                                            continue;
                                        }

                                        // Broadcast the message to all clients
                                        debug!("Broadcasting message to all clients");
                                        let _ = broadcast_tx.send(text);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(peer = %addr, error = %e, "Failed to parse message");
                                // Don't broadcast unparseable messages - security risk
                                let error = WsMessage::Error {
                                    message: format!("Invalid message format: {}", e),
                                };
                                let _ = write.send(Message::Text(serde_json::to_string(&error).unwrap())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(e)) => {
                        error!(peer = %addr, error = %e, "WebSocket error");
                        break;
                    }
                    _ => {}
                }
            }

            // Handle broadcast messages to send to this client
            msg = broadcast_rx.recv() => {
                if let Ok(text) = msg {
                    // Only send to authenticated clients
                    if is_authenticated {
                        if write.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    // User disconnected
    if let Some(uid) = user_id {
        info!(user_id = %uid, "User disconnected");
        clients.lock().await.remove(&uid);

        let presence = WsMessage::Presence {
            user_id: uid,
            is_online: false,
            last_seen: Some(chrono::Utc::now().timestamp_millis()),
        };
        let _ = broadcast_tx.send(serde_json::to_string(&presence).unwrap());
    }
}
