use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "message")]
    ChatMessage {
        id: String,
        chat_id: String,
        sender_id: String,
        sender_name: String,
        content: String,
        timestamp: i64,
    },
    #[serde(rename = "typing")]
    Typing {
        chat_id: String,
        user_id: String,
        is_typing: bool,
    },
    #[serde(rename = "presence")]
    Presence {
        user_id: String,
        is_online: bool,
        last_seen: Option<i64>,
    },
    #[serde(rename = "read_receipt")]
    ReadReceipt {
        chat_id: String,
        user_id: String,
        message_id: String,
    },
    #[serde(rename = "connect")]
    Connect { user_id: String },
    #[serde(rename = "error")]
    Error { message: String },
}

pub struct WebSocketServer {
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    broadcast_tx: broadcast::Sender<String>,
    // Channel to send messages to the relay client (when we're not the server)
    relay_tx: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
    is_server: Arc<Mutex<bool>>,
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketServer {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            relay_tx: Arc::new(Mutex::new(None)),
            is_server: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self, addr: &str) -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
        // First, try to connect to an existing server to check if one is already running
        // This is more reliable than just trying to bind
        println!("[WS] Checking if WebSocket server already exists...");

        match tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            TcpStream::connect("127.0.0.1:9001")
        ).await {
            Ok(Ok(_stream)) => {
                // Successfully connected - a server exists, we should be a client
                println!("[WS] Found existing WebSocket server - connecting as client");
                self.connect_as_client("ws://127.0.0.1:9001").await;
                return Ok(9001);
            }
            Ok(Err(e)) => {
                println!("[WS] No existing server found (connection error: {}), we will be the server", e);
            }
            Err(_) => {
                println!("[WS] No existing server found (timeout), we will be the server");
            }
        }

        // No server exists - try to bind as the server
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                // Bind failed but connection also failed earlier - race condition
                // Try connecting as client one more time
                println!("[WS] Bind failed ({}), trying to connect as client...", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                self.connect_as_client("ws://127.0.0.1:9001").await;
                return Ok(9001);
            }
        };

        *self.is_server.lock().await = true;
        let port = listener.local_addr()?.port();
        println!("WebSocket server listening on port {}", port);

        let clients = self.clients.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let clients = clients.clone();
                let broadcast_tx = broadcast_tx.clone();
                tokio::spawn(handle_connection(stream, addr, clients, broadcast_tx));
            }
        });

        Ok(port)
    }

    async fn connect_as_client(&self, url: &str) {
        let relay_tx = self.relay_tx.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let url = url.to_string();

        tokio::spawn(async move {
            let mut attempt = 0;
            loop {
                attempt += 1;
                println!("[WS Client] Connection attempt {} to {}", attempt, url);
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        println!("[WS Client] Successfully connected to WebSocket server!");
                        let (mut write, mut read) = ws_stream.split();

                        // Create channel for sending messages
                        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
                        *relay_tx.lock().await = Some(tx);

                        loop {
                            tokio::select! {
                                // Forward messages from our app to the server
                                Some(msg) = rx.recv() => {
                                    if write.send(Message::Text(msg)).await.is_err() {
                                        break;
                                    }
                                }
                                // Handle messages from server - forward to local broadcast for frontend
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            // Forward server messages to local broadcast channel
                                            // so our frontend WebSocket connection receives them
                                            println!("[WS Client] Received from server: {}", &text[..100.min(text.len())]);
                                            let _ = broadcast_tx.send(text);
                                        }
                                        Some(Ok(_)) => {} // Ignore non-text messages
                                        _ => break,
                                    }
                                }
                            }
                        }
                        println!("[WS Client] Disconnected from WebSocket server");
                        *relay_tx.lock().await = None;
                    }
                    Err(e) => {
                        eprintln!("[WS Client] Failed to connect to WebSocket server: {}", e);
                    }
                }
                // Reconnect after delay
                println!("[WS Client] Reconnecting in 3 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        });
    }

    pub fn broadcast(&self, message: WsMessage) -> Result<(), String> {
        let json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
        println!("[WS] broadcast() called with message: {}", &json[..100.min(json.len())]);

        // Always send to local broadcast (for local frontend)
        let local_result = self.broadcast_tx.send(json.clone());
        println!("[WS] Local broadcast result: {:?}", local_result.is_ok());

        // If we're not the server, also relay to the server
        // Use try_lock to avoid blocking and check if relay is available
        if let Ok(guard) = self.relay_tx.try_lock() {
            if let Some(tx) = guard.as_ref() {
                println!("[WS] Relaying to server via relay_tx");
                let _ = tx.send(json);
            } else {
                println!("[WS] No relay connection (we are the server)");
            }
        } else {
            println!("[WS] Could not acquire relay lock");
        }

        Ok(())
    }
}

async fn handle_connection(
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

// Global WebSocket server instance
use std::sync::OnceLock;
static WS_SERVER: OnceLock<WebSocketServer> = OnceLock::new();

pub fn get_ws_server() -> &'static WebSocketServer {
    WS_SERVER.get_or_init(WebSocketServer::new)
}

pub async fn init_websocket_server() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    let server = get_ws_server();
    server.start("127.0.0.1:9001").await
}
