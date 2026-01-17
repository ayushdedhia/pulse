use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};

use crate::messages::WsMessage;
use crate::state::ServerState;

/// Handle a single WebSocket connection
pub async fn handle_connection(ws_stream: WebSocketStream<TcpStream>, state: Arc<ServerState>) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Wait for Connect message to authenticate
    let user_id = match wait_for_connect(&mut ws_receiver).await {
        Some(id) => id,
        None => {
            warn!("Connection closed before authentication");
            return;
        }
    };

    info!("User connected: {}", user_id);

    // Create channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register client
    state.add_client(user_id.clone(), tx);

    // Send auth success response
    let auth_response = WsMessage::AuthResponse {
        success: true,
        message: "Connected to server".to_string(),
    };
    match serde_json::to_string(&auth_response) {
        Ok(json) => {
            if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                error!("Failed to send auth response to {}: {}", user_id, e);
            }
        }
        Err(e) => {
            error!("Failed to serialize auth response for {}: {}", user_id, e);
        }
    }

    // Broadcast presence to all other clients
    let presence = WsMessage::Presence {
        user_id: user_id.clone(),
        is_online: true,
        last_seen: None,
    };
    match serde_json::to_string(&presence) {
        Ok(json) => state.broadcast(&json, Some(&user_id)),
        Err(e) => error!("Failed to serialize presence for {}: {}", user_id, e),
    }

    // Send existing online users to the newly connected client
    for online_user_id in state.online_users() {
        if online_user_id != user_id {
            let existing_presence = WsMessage::Presence {
                user_id: online_user_id,
                is_online: true,
                last_seen: None,
            };
            if let Ok(json) = serde_json::to_string(&existing_presence) {
                state.send_to_user(&user_id, &json);
            }
        }
    }

    // Flush pending messages for this user (messages queued while offline)
    let pending = state.take_pending_messages(&user_id);
    if !pending.is_empty() {
        info!(
            "Delivering {} pending messages to {}",
            pending.len(),
            user_id
        );
        for msg in pending {
            state.send_to_user(&user_id, &msg);
        }
    }

    // Spawn task to forward messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages
    // Process incoming messages and monitor send task
    let user_id_clone = user_id.clone();
    let state_clone = state.clone();

    loop {
        tokio::select! {
            // Branch 1: Read from WebSocket
            res = ws_receiver.next() => {
                match res {
                    Some(Ok(Message::Text(text))) => {
                        handle_message(&text, &user_id_clone, &state_clone);
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("User {} sent close frame", user_id_clone);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = data;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for user {}: {}", user_id_clone, e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended for user {}", user_id_clone);
                        break;
                    }
                    _ => {}
                }
            }
            // Branch 2: Monitor Send Task (Write errors)
            _ = &mut send_task => {
                info!("Send task finished for user {} (likely connection lost)", user_id_clone);
                break;
            }
        }
    }

    // Cleanup
    send_task.abort();
    state.remove_client(&user_id);

    // Broadcast offline presence
    let offline_presence = WsMessage::Presence {
        user_id: user_id.clone(),
        is_online: false,
        last_seen: Some(chrono::Utc::now().timestamp_millis()),
    };
    if let Ok(json) = serde_json::to_string(&offline_presence) {
        state.broadcast(&json, None);
    }

    info!("User disconnected: {}", user_id);
}

/// Wait for the Connect message from a new connection
async fn wait_for_connect(
    receiver: &mut futures_util::stream::SplitStream<WebSocketStream<TcpStream>>,
) -> Option<String> {
    // Give client 10 seconds to authenticate
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while let Some(result) = receiver.next().await {
            if let Ok(Message::Text(text)) = result {
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(msg) => {
                        if let WsMessage::Connect { user_id, token } = msg {
                            // Basic auth check using environment variable
                            if let Ok(expected_token) = std::env::var("PULSE_ACCESS_TOKEN") {
                                if !expected_token.is_empty() {
                                    if let Some(received_token) = token {
                                        if received_token != expected_token {
                                            warn!(
                                                "Authentication failed for {}: Invalid token",
                                                user_id
                                            );
                                            return None;
                                        }
                                    } else {
                                        warn!(
                                            "Authentication failed for {}: No token provided",
                                            user_id
                                        );
                                        return None;
                                    }
                                }
                            }
                            return Some(user_id);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse Connect message: {}", e);
                    }
                }
            }
        }
        None
    });

    match timeout.await {
        Ok(result) => result,
        Err(_) => {
            warn!("Authentication timeout");
            None
        }
    }
}

/// Handle an incoming message from a connected client
pub fn handle_message(text: &str, sender_id: &str, state: &ServerState) {
    let mut msg: WsMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to parse message from {}: {}", sender_id, e);
            return;
        }
    };

    // Enforce sender identity to prevent spoofing
    match &mut msg {
        WsMessage::ChatMessage {
            sender_id: sid,
            sender_name: _,
            ..
        } => *sid = sender_id.to_string(),
        WsMessage::Typing { user_id, .. } => *user_id = sender_id.to_string(),
        WsMessage::Presence { user_id, .. } => *user_id = sender_id.to_string(),

        // CORRECTION: In receipts, 'sender_id' is the DESTINATION (original sender of the Msg).
        // We must enforce that the 'delivered_to' / 'user_id' matches the current connection.
        WsMessage::DeliveryReceipt {
            delivered_to: dt, ..
        } => *dt = sender_id.to_string(),

        WsMessage::ReadReceipt { user_id: uid, .. } => *uid = sender_id.to_string(),

        WsMessage::ProfileUpdate { user_id, .. } => *user_id = sender_id.to_string(),

        // Video Call & WebRTC - Sender Enforcement
        WsMessage::CallInvite { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::CallRinging { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::CallAccept { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::CallReject { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::CallHangup { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::RtcOffer { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::RtcAnswer { from_user_id, .. } => *from_user_id = sender_id.to_string(),
        WsMessage::RtcIceCandidate { from_user_id, .. } => *from_user_id = sender_id.to_string(),

        // Messages that shouldn't be sent by client or don't generally carry spoofable sender_id in this context
        WsMessage::Connect { .. } | WsMessage::AuthResponse { .. } | WsMessage::Error { .. } => {}
    }

    // Re-serialize the secure message
    let safe_text = match serde_json::to_string(&msg) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to re-serialize message from {}: {}", sender_id, e);
            return;
        }
    };

    match &msg {
        WsMessage::ChatMessage { recipient_id, .. } => {
            // Route to specific recipient (queues if offline)
            state.send_or_queue(recipient_id, &safe_text);
        }
        WsMessage::Typing { .. } => {
            // Typing indicators are ephemeral - don't queue, just broadcast
            state.broadcast(&safe_text, Some(sender_id));
        }
        WsMessage::Presence { .. } => {
            // Presence is broadcast to all online users
            state.broadcast(&safe_text, Some(sender_id));
        }
        WsMessage::DeliveryReceipt {
            sender_id: original_sender,
            ..
        } => {
            // Route receipt BACK to the original sender of the message.
            state.send_or_queue(original_sender, &safe_text);
        }
        WsMessage::ReadReceipt {
            sender_id: original_sender,
            ..
        } => {
            // Route receipt BACK to the original sender of the message.
            state.send_or_queue(original_sender, &safe_text);
        }
        WsMessage::ProfileUpdate { .. } => {
            // Profile updates broadcast to all online users
            state.broadcast(&safe_text, Some(sender_id));
        }
        // === Video Call Control - route directly to recipient (no queue, time-sensitive) ===
        WsMessage::CallInvite { to_user_id, .. }
        | WsMessage::CallRinging { to_user_id, .. }
        | WsMessage::CallAccept { to_user_id, .. }
        | WsMessage::CallReject { to_user_id, .. }
        | WsMessage::CallHangup { to_user_id, .. }
        | WsMessage::RtcOffer { to_user_id, .. }
        | WsMessage::RtcAnswer { to_user_id, .. }
        | WsMessage::RtcIceCandidate { to_user_id, .. } => {
            // Call signaling is time-sensitive - send directly, don't queue
            state.send_to_user(to_user_id, &safe_text);
        }
        WsMessage::Connect { .. } => {
            // Already authenticated, ignore
        }
        WsMessage::AuthResponse { .. } | WsMessage::Error { .. } => {
            // Server-only messages, ignore from client
        }
    }
}
