use super::messages::WsMessage;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn handle_connection(
   stream: TcpStream,
   addr: SocketAddr,
   clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
   broadcast_tx: broadcast::Sender<String>,
) {
   println!("New WebSocket connection from: {}", addr);

   let ws_stream = match accept_async(stream).await {
      Ok(ws) => ws,
      Err(e) => {
         eprintln!("WebSocket handshake failed: {}", e);
         return;
      }
   };

   let (mut write, mut read) = ws_stream.split();
   let mut broadcast_rx = broadcast_tx.subscribe();
   let mut user_id: Option<String> = None;

   loop {
      tokio::select! {
          // Handle incoming messages from client
          msg = read.next() => {
              match msg {
                  Some(Ok(Message::Text(text))) => {
                      println!("[WS Server] Received message: {}", &text[..100.min(text.len())]);
                      match serde_json::from_str::<WsMessage>(&text) {
                          Ok(ws_msg) => {
                              match ws_msg {
                                  WsMessage::Connect { user_id: uid } => {
                                      user_id = Some(uid.clone());
                                      println!("[WS Server] User {} connected", uid);

                                      // Broadcast presence
                                      let presence = WsMessage::Presence {
                                          user_id: uid,
                                          is_online: true,
                                          last_seen: None,
                                      };
                                      let _ = broadcast_tx.send(serde_json::to_string(&presence).unwrap());
                                  }
                                  _ => {
                                      // Broadcast the message to all clients
                                      println!("[WS Server] Broadcasting message to all clients");
                                      let _ = broadcast_tx.send(text);
                                  }
                              }
                          }
                          Err(e) => {
                              eprintln!("[WS Server] Failed to parse message: {}", e);
                              // Still try to broadcast raw message
                              let _ = broadcast_tx.send(text);
                          }
                      }
                  }
                  Some(Ok(Message::Close(_))) | None => {
                      break;
                  }
                  Some(Err(e)) => {
                      eprintln!("WebSocket error: {}", e);
                      break;
                  }
                  _ => {}
              }
          }

          // Handle broadcast messages to send to this client
          msg = broadcast_rx.recv() => {
              if let Ok(text) = msg {
                  if write.send(Message::Text(text)).await.is_err() {
                      break;
                  }
              }
          }
      }
   }

   // User disconnected
   if let Some(uid) = user_id {
      println!("User {} disconnected", uid);
      clients.lock().await.remove(&uid);

      let presence = WsMessage::Presence {
         user_id: uid,
         is_online: false,
         last_seen: Some(chrono::Utc::now().timestamp_millis()),
      };
      let _ = broadcast_tx.send(serde_json::to_string(&presence).unwrap());
   }
}
