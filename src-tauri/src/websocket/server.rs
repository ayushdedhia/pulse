use super::handlers::handle_connection;
use super::messages::WsMessage;
use super::get_session_token;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

#[derive(Clone, serde::Serialize)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
    pub connected: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct NetworkStatus {
    pub is_server: bool,
    pub local_ip: Option<String>,
    pub port: u16,
    pub connected_peers: Vec<PeerInfo>,
}

pub struct WebSocketServer {
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    broadcast_tx: broadcast::Sender<String>,
    relay_tx: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
    is_server: Arc<Mutex<bool>>,
    connected_peers: Arc<Mutex<Vec<PeerInfo>>>,
    local_ip: Arc<Mutex<Option<String>>>,
    port: Arc<Mutex<u16>>,
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketServer {
    pub fn new() -> Self {
        // Increased buffer size to handle bursts, with 1000 message capacity
        let (broadcast_tx, _) = broadcast::channel(1000);

        // Get local IP address
        let local_ip = local_ip_address::local_ip()
            .ok()
            .map(|ip| ip.to_string());

        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            relay_tx: Arc::new(Mutex::new(None)),
            is_server: Arc::new(Mutex::new(false)),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
            local_ip: Arc::new(Mutex::new(local_ip)),
            port: Arc::new(Mutex::new(9001)),
        }
    }

    pub async fn start(&self, addr: &str) -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
        // First, try to connect to an existing LOCAL server to check if one is already running
        debug!("Checking if WebSocket server already exists on localhost");

        match tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            TcpStream::connect("127.0.0.1:9001"),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                // Successfully connected - a local server exists, we should be a client
                info!("Found existing WebSocket server - connecting as client");
                self.connect_as_client("ws://127.0.0.1:9001", "127.0.0.1").await;
                return Ok(9001);
            }
            Ok(Err(e)) => {
                debug!(error = %e, "No existing server found (connection error), we will be the server");
            }
            Err(_) => {
                debug!("No existing server found (timeout), we will be the server");
            }
        }

        // No server exists - try to bind as the server on ALL interfaces (0.0.0.0)
        // This allows LAN connections from other machines
        let bind_addr = addr.replace("127.0.0.1", "0.0.0.0");
        debug!(addr = %bind_addr, "Binding for LAN access");

        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(l) => l,
            Err(e) => {
                // Bind failed but connection also failed earlier - race condition
                warn!(error = %e, "Bind failed, trying to connect as client");
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                self.connect_as_client("ws://127.0.0.1:9001", "127.0.0.1").await;
                return Ok(9001);
            }
        };

        *self.is_server.lock().await = true;
        let port = listener.local_addr()?.port();
        *self.port.lock().await = port;

        if let Some(ip) = self.local_ip.lock().await.as_ref() {
            info!(ip = %ip, port = port, "Server listening on LAN");
        }
        info!(port = port, "WebSocket server listening");

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

    async fn connect_as_client(&self, url: &str, peer_ip: &str) {
        let relay_tx = self.relay_tx.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let connected_peers = self.connected_peers.clone();
        let url = url.to_string();
        let peer_ip = peer_ip.to_string();

        tokio::spawn(async move {
            let mut attempt = 0;
            loop {
                attempt += 1;
                debug!(attempt = attempt, url = %url, "Client connection attempt");
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        info!(url = %url, "Connected to WebSocket server");

                        // Track this peer as connected
                        {
                            let mut peers = connected_peers.lock().await;
                            if let Some(peer) = peers.iter_mut().find(|p| p.ip == peer_ip) {
                                peer.connected = true;
                            } else {
                                peers.push(PeerInfo {
                                    ip: peer_ip.clone(),
                                    port: 9001,
                                    connected: true,
                                });
                            }
                        }

                        let (mut write, mut read) = ws_stream.split();

                        // Authenticate with the server using our session token
                        let auth_token = get_session_token().to_string();
                        let connect_msg = WsMessage::Connect {
                            user_id: format!("relay_{}", peer_ip),
                            auth_token: Some(auth_token.clone()),
                        };
                        let auth_json = serde_json::to_string(&connect_msg).unwrap();
                        debug!("Sending authentication to server");
                        if write.send(Message::Text(auth_json)).await.is_err() {
                            error!("Failed to send authentication");
                            continue;
                        }

                        // Wait for auth response
                        let mut authenticated = false;
                        if let Some(Ok(Message::Text(response))) = read.next().await {
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&response) {
                                match msg {
                                    WsMessage::AuthResponse { success, message } => {
                                        if success {
                                            info!("Relay authenticated successfully");
                                            authenticated = true;
                                        } else {
                                            // Server doesn't know our token yet - this is expected
                                            // for first connection. We'll send a token exchange.
                                            warn!(message = %message, "Auth failed, attempting token exchange");
                                        }
                                    }
                                    _ => {
                                        debug!(preview = %&response[..100.min(response.len())], "Unexpected response");
                                    }
                                }
                            }
                        }

                        // If auth failed, try sending a PeerConnect with token for exchange
                        if !authenticated {
                            let peer_connect = WsMessage::PeerConnect {
                                peer_token: auth_token,
                            };
                            let peer_json = serde_json::to_string(&peer_connect).unwrap();
                            debug!("Sending peer token exchange");
                            if write.send(Message::Text(peer_json)).await.is_err() {
                                error!("Failed to send peer connect");
                                continue;
                            }

                            // Wait for auth response after token exchange
                            if let Some(Ok(Message::Text(response))) = read.next().await {
                                if let Ok(WsMessage::AuthResponse { success, message }) =
                                    serde_json::from_str(&response)
                                {
                                    if success {
                                        info!("Peer token exchange successful");
                                        authenticated = true;
                                    } else {
                                        error!(message = %message, "Peer token exchange failed");
                                        continue;
                                    }
                                }
                            }
                        }

                        if !authenticated {
                            error!("Failed to authenticate with server");
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            continue;
                        }

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
                                            debug!(preview = %&text[..100.min(text.len())], "Received from server");
                                            let _ = broadcast_tx.send(text);
                                        }
                                        Some(Ok(_)) => {} // Ignore non-text messages
                                        _ => break,
                                    }
                                }
                            }
                        }
                        info!("Disconnected from WebSocket server");
                        *relay_tx.lock().await = None;

                        // Mark peer as disconnected
                        {
                            let mut peers = connected_peers.lock().await;
                            if let Some(peer) = peers.iter_mut().find(|p| p.ip == peer_ip) {
                                peer.connected = false;
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to connect to WebSocket server");
                    }
                }
                // Reconnect after delay
                debug!("Reconnecting in 3 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        });
    }

    /// Connect to a peer on the LAN by IP address
    pub async fn connect_to_peer(&self, ip: &str, port: u16) -> Result<(), String> {
        let url = format!("ws://{}:{}", ip, port);
        info!(url = %url, "Attempting to connect to peer");

        // Check if already connected to this peer
        {
            let peers = self.connected_peers.lock().await;
            if peers.iter().any(|p| p.ip == ip && p.connected) {
                return Err(format!("Already connected to peer {}", ip));
            }
        }

        // Add peer to list (will be marked connected when connection succeeds)
        {
            let mut peers = self.connected_peers.lock().await;
            if !peers.iter().any(|p| p.ip == ip) {
                peers.push(PeerInfo {
                    ip: ip.to_string(),
                    port,
                    connected: false,
                });
            }
        }

        self.connect_as_client(&url, ip).await;
        Ok(())
    }

    /// Get current network status
    pub async fn get_network_status(&self) -> NetworkStatus {
        NetworkStatus {
            is_server: *self.is_server.lock().await,
            local_ip: self.local_ip.lock().await.clone(),
            port: *self.port.lock().await,
            connected_peers: self.connected_peers.lock().await.clone(),
        }
    }

    /// Get local IP address
    pub async fn get_local_ip(&self) -> Option<String> {
        self.local_ip.lock().await.clone()
    }

    pub fn broadcast(&self, message: WsMessage) -> Result<(), String> {
        let json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
        debug!(preview = %&json[..100.min(json.len())], "Broadcasting message");

        // Always send to local broadcast (for local frontend)
        let local_result = self.broadcast_tx.send(json.clone());
        debug!(success = local_result.is_ok(), "Local broadcast result");

        // If we're not the server, also relay to the server
        if let Ok(guard) = self.relay_tx.try_lock() {
            if let Some(tx) = guard.as_ref() {
                debug!("Relaying to server");
                let _ = tx.send(json);
            }
        }

        Ok(())
    }
}
