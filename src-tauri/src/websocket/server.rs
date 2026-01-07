use super::handlers::handle_connection;
use super::messages::WsMessage;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

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
        let (broadcast_tx, _) = broadcast::channel(100);

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
        println!("[WS] Checking if WebSocket server already exists on localhost...");

        match tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            TcpStream::connect("127.0.0.1:9001"),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                // Successfully connected - a local server exists, we should be a client
                println!("[WS] Found existing WebSocket server - connecting as client");
                self.connect_as_client("ws://127.0.0.1:9001", "127.0.0.1").await;
                return Ok(9001);
            }
            Ok(Err(e)) => {
                println!(
                    "[WS] No existing server found (connection error: {}), we will be the server",
                    e
                );
            }
            Err(_) => {
                println!("[WS] No existing server found (timeout), we will be the server");
            }
        }

        // No server exists - try to bind as the server on ALL interfaces (0.0.0.0)
        // This allows LAN connections from other machines
        let bind_addr = addr.replace("127.0.0.1", "0.0.0.0");
        println!("[WS] Binding to {} for LAN access", bind_addr);

        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(l) => l,
            Err(e) => {
                // Bind failed but connection also failed earlier - race condition
                println!("[WS] Bind failed ({}), trying to connect as client...", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                self.connect_as_client("ws://127.0.0.1:9001", "127.0.0.1").await;
                return Ok(9001);
            }
        };

        *self.is_server.lock().await = true;
        let port = listener.local_addr()?.port();
        *self.port.lock().await = port;

        if let Some(ip) = self.local_ip.lock().await.as_ref() {
            println!("[WS] Server listening on LAN at {}:{}", ip, port);
        }
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
                println!("[WS Client] Connection attempt {} to {}", attempt, url);
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        println!("[WS Client] Successfully connected to WebSocket server!");

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

                        // Mark peer as disconnected
                        {
                            let mut peers = connected_peers.lock().await;
                            if let Some(peer) = peers.iter_mut().find(|p| p.ip == peer_ip) {
                                peer.connected = false;
                            }
                        }
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

    /// Connect to a peer on the LAN by IP address
    pub async fn connect_to_peer(&self, ip: &str, port: u16) -> Result<(), String> {
        let url = format!("ws://{}:{}", ip, port);
        println!("[WS] Attempting to connect to peer at {}", url);

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
        println!(
            "[WS] broadcast() called with message: {}",
            &json[..100.min(json.len())]
        );

        // Always send to local broadcast (for local frontend)
        let local_result = self.broadcast_tx.send(json.clone());
        println!("[WS] Local broadcast result: {:?}", local_result.is_ok());

        // If we're not the server, also relay to the server
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
