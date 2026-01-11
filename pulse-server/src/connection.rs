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
    if let Ok(json) = serde_json::to_string(&auth_response) {
        let _ = ws_sender.send(Message::Text(json.into())).await;
    }

    // Broadcast presence to all other clients
    let presence = WsMessage::Presence {
        user_id: user_id.clone(),
        is_online: true,
        last_seen: None,
    };
    if let Ok(json) = serde_json::to_string(&presence) {
        state.broadcast(&json, Some(&user_id));
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
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages
    let user_id_clone = user_id.clone();
    let state_clone = state.clone();
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                handle_message(&text, &user_id_clone, &state_clone);
            }
            Ok(Message::Close(_)) => {
                info!("User {} sent close frame", user_id_clone);
                break;
            }
            Ok(Message::Ping(data)) => {
                // Pong is handled automatically by tungstenite
                let _ = data;
            }
            Err(e) => {
                error!("WebSocket error for user {}: {}", user_id_clone, e);
                break;
            }
            _ => {}
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
                if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                    if let WsMessage::Connect { user_id } = msg {
                        return Some(user_id);
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
fn handle_message(text: &str, sender_id: &str, state: &ServerState) {
    let msg: WsMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to parse message from {}: {}", sender_id, e);
            return;
        }
    };

    match &msg {
        WsMessage::ChatMessage { recipient_id, .. } => {
            // Route to specific recipient (queues if offline)
            state.send_or_queue(recipient_id, text);
        }
        WsMessage::Typing { .. } => {
            // Typing indicators are ephemeral - don't queue, just broadcast
            state.broadcast(text, Some(sender_id));
        }
        WsMessage::Presence { .. } => {
            // Presence is broadcast to all online users
            state.broadcast(text, Some(sender_id));
        }
        WsMessage::DeliveryReceipt { sender_id: original_sender, .. } => {
            // Route receipt back to original message sender (queues if offline)
            state.send_or_queue(original_sender, text);
        }
        WsMessage::ReadReceipt { sender_id: original_sender, .. } => {
            // Route receipt back to original message sender (queues if offline)
            state.send_or_queue(original_sender, text);
        }
        WsMessage::ProfileUpdate { .. } => {
            // Profile updates broadcast to all online users
            state.broadcast(text, Some(sender_id));
        }
        WsMessage::Connect { .. } => {
            // Already authenticated, ignore
        }
        WsMessage::AuthResponse { .. } | WsMessage::Error { .. } => {
            // Server-only messages, ignore from client
        }
    }
}
