//! End-to-End tests for the Pulse WebSocket server
//!
//! These tests spawn the actual server binary and connect real WebSocket clients
//! to verify the complete message flow works as expected.
//!
//! Note: These tests must run sequentially (--test-threads=1) to avoid port conflicts
//! when spawning multiple server processes on Windows.

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Wrapper to manage server process lifecycle
struct ServerProcess {
    child: Child,
    port: u16,
}

impl ServerProcess {
    /// Start the server on a specific port
    fn start(port: u16) -> Result<Self, String> {
        // Build the server first to ensure it's up to date
        let build_status = Command::new("cargo")
            .args(["build", "--bin", "pulse-server"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to build server: {}", e))?;

        if !build_status.success() {
            return Err("Server build failed".to_string());
        }

        // Start the server process
        let child = Command::new("cargo")
            .args(["run", "--bin", "pulse-server"])
            .env("PULSE_SERVER_ADDR", format!("127.0.0.1:{}", port))
            .env("RUST_LOG", "info")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start server: {}", e))?;

        Ok(Self { child, port })
    }

    /// Wait for the server to be ready to accept connections
    async fn wait_until_ready(&self) -> Result<(), String> {
        let url = format!("ws://127.0.0.1:{}", self.port);

        for attempt in 0..30 {
            match connect_async(&url).await {
                Ok(_) => return Ok(()),
                Err(_) => {
                    if attempt < 29 {
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }

        Err("Server did not become ready within 3 seconds".to_string())
    }

    fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        // Kill the server process when test ends
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Helper to connect and authenticate a client
async fn connect_and_auth(
    url: &str,
    user_id: &str,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    String,
> {
    let (ws_stream, _) = connect_async(url)
        .await
        .map_err(|e| format!("Connect failed: {}", e))?;

    let (mut write, mut read) = ws_stream.split();

    // Send connect message
    let connect_msg = json!({
        "type": "connect",
        "user_id": user_id
    });
    write
        .send(Message::Text(connect_msg.to_string().into()))
        .await
        .map_err(|e| format!("Send failed: {}", e))?;

    // Wait for auth response
    let response = timeout(Duration::from_secs(5), read.next())
        .await
        .map_err(|_| "Timeout waiting for auth")?
        .ok_or("Stream closed")?
        .map_err(|e| format!("Read error: {}", e))?;

    if let Message::Text(text) = response {
        let msg: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("Parse error: {}", e))?;

        if msg["type"] != "auth_response" || msg["success"] != true {
            return Err(format!("Auth failed: {:?}", msg));
        }
    } else {
        return Err("Expected text message".to_string());
    }

    Ok(write.reunite(read).unwrap())
}

/// Read next message with timeout, skipping presence notifications if needed
async fn read_message_of_type(
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    expected_type: &str,
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);

    while tokio::time::Instant::now() < deadline {
        let remaining = deadline - tokio::time::Instant::now();
        match timeout(remaining, read.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let msg: serde_json::Value =
                    serde_json::from_str(&text).map_err(|e| format!("Parse error: {}", e))?;

                if msg["type"] == expected_type {
                    return Ok(msg);
                }
                // Skip other message types (like presence)
            }
            Ok(Some(Ok(_))) => continue,
            Ok(Some(Err(e))) => return Err(format!("Read error: {}", e)),
            Ok(None) => return Err("Stream closed".to_string()),
            Err(_) => return Err(format!("Timeout waiting for {} message", expected_type)),
        }
    }

    Err(format!("Timeout waiting for {} message", expected_type))
}

// Use a static port counter with larger spacing to avoid conflicts on Windows
// where ports may take time to be released after process termination
use std::sync::atomic::{AtomicU16, Ordering};
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19001);

fn get_unique_port() -> u16 {
    // Increment by 10 to leave room for any lingering connections
    PORT_COUNTER.fetch_add(10, Ordering::SeqCst)
}

#[tokio::test]
async fn e2e_server_starts_and_accepts_connections() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");

    server
        .wait_until_ready()
        .await
        .expect("Server not ready");

    let client = connect_and_auth(&server.url(), "test_user")
        .await
        .expect("Failed to connect");

    drop(client);
}

#[tokio::test]
async fn e2e_presence_broadcast_on_connect() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect first client
    let client1 = connect_and_auth(&server.url(), "user1")
        .await
        .expect("Failed to connect user1");
    let (_, mut read1) = client1.split();

    // Connect second client
    let _client2 = connect_and_auth(&server.url(), "user2")
        .await
        .expect("Failed to connect user2");

    // Client 1 should receive presence notification for user2
    let msg = read_message_of_type(&mut read1, "presence", 5)
        .await
        .expect("Should receive presence");

    assert_eq!(msg["user_id"], "user2");
    assert_eq!(msg["is_online"], true);
}

#[tokio::test]
async fn e2e_message_delivery_between_clients() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect two clients
    let client1 = connect_and_auth(&server.url(), "alice")
        .await
        .expect("Failed to connect alice");
    let client2 = connect_and_auth(&server.url(), "bob")
        .await
        .expect("Failed to connect bob");

    let (mut write1, _read1) = client1.split();
    let (_, mut read2) = client2.split();

    // Drain any pending presence notifications
    sleep(Duration::from_millis(100)).await;
    while timeout(Duration::from_millis(50), read2.next())
        .await
        .is_ok()
    {}

    // Alice sends a message
    let chat_msg = json!({
        "type": "message",
        "id": "msg-e2e-1",
        "chat_id": "chat-alice-bob",
        "sender_id": "alice",
        "sender_name": "Alice",
        "content": "Hello Bob! This is an E2E test message.",
        "timestamp": 1234567890
    });
    write1
        .send(Message::Text(chat_msg.to_string().into()))
        .await
        .expect("Send failed");

    // Bob should receive the message
    let received = read_message_of_type(&mut read2, "message", 5)
        .await
        .expect("Bob should receive message");

    assert_eq!(received["id"], "msg-e2e-1");
    assert_eq!(received["sender_id"], "alice");
    assert_eq!(received["content"], "Hello Bob! This is an E2E test message.");
}

#[tokio::test]
async fn e2e_typing_indicator_broadcast() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    let client1 = connect_and_auth(&server.url(), "typer")
        .await
        .expect("Failed to connect");
    let client2 = connect_and_auth(&server.url(), "watcher")
        .await
        .expect("Failed to connect");

    let (mut write1, _) = client1.split();
    let (_, mut read2) = client2.split();

    // Drain presence
    sleep(Duration::from_millis(100)).await;
    while timeout(Duration::from_millis(50), read2.next())
        .await
        .is_ok()
    {}

    // Send typing indicator
    let typing_msg = json!({
        "type": "typing",
        "chat_id": "chat1",
        "user_id": "typer",
        "is_typing": true
    });
    write1
        .send(Message::Text(typing_msg.to_string().into()))
        .await
        .expect("Send failed");

    // Watcher should receive typing indicator
    let received = read_message_of_type(&mut read2, "typing", 5)
        .await
        .expect("Should receive typing");

    assert_eq!(received["user_id"], "typer");
    assert_eq!(received["is_typing"], true);
}

#[tokio::test]
async fn e2e_delivery_receipt_flow() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    let client1 = connect_and_auth(&server.url(), "sender")
        .await
        .expect("Failed to connect");
    let client2 = connect_and_auth(&server.url(), "receiver")
        .await
        .expect("Failed to connect");

    let (_, mut read1) = client1.split();
    let (mut write2, _) = client2.split();

    // Drain presence
    sleep(Duration::from_millis(100)).await;
    while timeout(Duration::from_millis(50), read1.next())
        .await
        .is_ok()
    {}

    // Receiver sends delivery receipt
    let receipt = json!({
        "type": "delivery_receipt",
        "message_id": "msg-123",
        "chat_id": "chat1",
        "delivered_to": "receiver"
    });
    write2
        .send(Message::Text(receipt.to_string().into()))
        .await
        .expect("Send failed");

    // Sender should receive the delivery receipt
    let received = read_message_of_type(&mut read1, "delivery_receipt", 5)
        .await
        .expect("Should receive delivery receipt");

    assert_eq!(received["message_id"], "msg-123");
    assert_eq!(received["delivered_to"], "receiver");
}

#[tokio::test]
async fn e2e_read_receipt_flow() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    let client1 = connect_and_auth(&server.url(), "author")
        .await
        .expect("Failed to connect");
    let client2 = connect_and_auth(&server.url(), "reader")
        .await
        .expect("Failed to connect");

    let (_, mut read1) = client1.split();
    let (mut write2, _) = client2.split();

    // Drain presence
    sleep(Duration::from_millis(100)).await;
    while timeout(Duration::from_millis(50), read1.next())
        .await
        .is_ok()
    {}

    // Reader sends read receipt
    let receipt = json!({
        "type": "read_receipt",
        "chat_id": "chat1",
        "user_id": "reader",
        "message_ids": ["msg-1", "msg-2", "msg-3"]
    });
    write2
        .send(Message::Text(receipt.to_string().into()))
        .await
        .expect("Send failed");

    // Author should receive the read receipt
    let received = read_message_of_type(&mut read1, "read_receipt", 5)
        .await
        .expect("Should receive read receipt");

    assert_eq!(received["user_id"], "reader");
    let msg_ids: Vec<String> = serde_json::from_value(received["message_ids"].clone()).unwrap();
    assert_eq!(msg_ids, vec!["msg-1", "msg-2", "msg-3"]);
}

#[tokio::test]
async fn e2e_profile_update_broadcast() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    let client1 = connect_and_auth(&server.url(), "updater")
        .await
        .expect("Failed to connect");
    let client2 = connect_and_auth(&server.url(), "observer")
        .await
        .expect("Failed to connect");

    let (mut write1, _) = client1.split();
    let (_, mut read2) = client2.split();

    // Drain presence
    sleep(Duration::from_millis(100)).await;
    while timeout(Duration::from_millis(50), read2.next())
        .await
        .is_ok()
    {}

    // Send profile update
    let profile_update = json!({
        "type": "profile_update",
        "user_id": "updater",
        "name": "New Name",
        "phone": "+1234567890",
        "avatar_url": null,
        "about": "Updated status!",
        "avatar_data": null
    });
    write1
        .send(Message::Text(profile_update.to_string().into()))
        .await
        .expect("Send failed");

    // Observer should receive the profile update
    let received = read_message_of_type(&mut read2, "profile_update", 5)
        .await
        .expect("Should receive profile update");

    assert_eq!(received["user_id"], "updater");
    assert_eq!(received["name"], "New Name");
    assert_eq!(received["about"], "Updated status!");
}

#[tokio::test]
async fn e2e_offline_presence_on_disconnect() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect first client
    let client1 = connect_and_auth(&server.url(), "stayer")
        .await
        .expect("Failed to connect");
    let (_, mut read1) = client1.split();

    // Connect and then disconnect second client
    {
        let client2 = connect_and_auth(&server.url(), "leaver")
            .await
            .expect("Failed to connect");

        // Drain online presence
        let _ = read_message_of_type(&mut read1, "presence", 2).await;

        // client2 disconnects when it goes out of scope
        drop(client2);
    }

    // Should receive offline presence
    let received = read_message_of_type(&mut read1, "presence", 5)
        .await
        .expect("Should receive offline presence");

    assert_eq!(received["user_id"], "leaver");
    assert_eq!(received["is_online"], false);
    assert!(received["last_seen"].is_number());
}

#[tokio::test]
async fn e2e_multiple_clients_broadcast() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect three clients
    let client1 = connect_and_auth(&server.url(), "broadcaster")
        .await
        .expect("Failed to connect");
    let client2 = connect_and_auth(&server.url(), "listener1")
        .await
        .expect("Failed to connect");
    let client3 = connect_and_auth(&server.url(), "listener2")
        .await
        .expect("Failed to connect");

    let (mut write1, _) = client1.split();
    let (_, mut read2) = client2.split();
    let (_, mut read3) = client3.split();

    // Drain presence notifications
    sleep(Duration::from_millis(200)).await;
    while timeout(Duration::from_millis(50), read2.next())
        .await
        .is_ok()
    {}
    while timeout(Duration::from_millis(50), read3.next())
        .await
        .is_ok()
    {}

    // Broadcaster sends a message
    let chat_msg = json!({
        "type": "message",
        "id": "broadcast-msg",
        "chat_id": "group-chat",
        "sender_id": "broadcaster",
        "sender_name": "Broadcaster",
        "content": "Hello everyone!",
        "timestamp": 1234567890
    });
    write1
        .send(Message::Text(chat_msg.to_string().into()))
        .await
        .expect("Send failed");

    // Both listeners should receive the message
    let msg2 = read_message_of_type(&mut read2, "message", 5)
        .await
        .expect("listener1 should receive");
    let msg3 = read_message_of_type(&mut read3, "message", 5)
        .await
        .expect("listener2 should receive");

    assert_eq!(msg2["content"], "Hello everyone!");
    assert_eq!(msg3["content"], "Hello everyone!");
}

#[tokio::test]
async fn e2e_sender_does_not_receive_own_message() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    let client = connect_and_auth(&server.url(), "self_sender")
        .await
        .expect("Failed to connect");
    let (mut write, mut read) = client.split();

    // Send a message
    let chat_msg = json!({
        "type": "message",
        "id": "self-msg",
        "chat_id": "chat1",
        "sender_id": "self_sender",
        "sender_name": "Self",
        "content": "Talking to myself",
        "timestamp": 1234567890
    });
    write
        .send(Message::Text(chat_msg.to_string().into()))
        .await
        .expect("Send failed");

    // Should NOT receive our own message back
    let result = timeout(Duration::from_millis(500), read.next()).await;

    // Either timeout (no message) or not a "message" type
    match result {
        Err(_) => {} // Timeout - good, no message received
        Ok(Some(Ok(Message::Text(text)))) => {
            let msg: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_ne!(
                msg["type"], "message",
                "Should not receive own message back"
            );
        }
        _ => {}
    }
}

#[tokio::test]
async fn e2e_server_handles_invalid_json() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    let (ws_stream, _) = connect_async(&server.url())
        .await
        .expect("Connect failed");
    let (mut write, mut read) = ws_stream.split();

    // First authenticate properly
    let connect_msg = json!({
        "type": "connect",
        "user_id": "invalid_json_tester"
    });
    write
        .send(Message::Text(connect_msg.to_string().into()))
        .await
        .unwrap();

    // Wait for auth response
    let _ = timeout(Duration::from_secs(2), read.next()).await;

    // Send invalid JSON
    write
        .send(Message::Text("this is not valid json".into()))
        .await
        .unwrap();

    // Connection should still be alive - send a valid message
    let typing_msg = json!({
        "type": "typing",
        "chat_id": "chat1",
        "user_id": "invalid_json_tester",
        "is_typing": true
    });

    // Should be able to send without error
    let send_result = write
        .send(Message::Text(typing_msg.to_string().into()))
        .await;

    assert!(
        send_result.is_ok(),
        "Connection should survive invalid JSON"
    );
}

#[tokio::test]
async fn e2e_reconnection_replaces_old_session() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect observer to watch presence changes
    let observer = connect_and_auth(&server.url(), "observer")
        .await
        .expect("Failed to connect");
    let (_, mut observer_read) = observer.split();

    // First connection for reconnecting_user
    let _client1 = connect_and_auth(&server.url(), "reconnecting_user")
        .await
        .expect("Failed to connect");

    // Drain online presence
    let _ = read_message_of_type(&mut observer_read, "presence", 2).await;

    // Second connection with same user_id (simulates reconnect)
    let client2 = connect_and_auth(&server.url(), "reconnecting_user")
        .await
        .expect("Failed to connect second time");

    let (mut write2, _) = client2.split();

    // Drain any presence updates from reconnection
    sleep(Duration::from_millis(200)).await;
    while timeout(Duration::from_millis(50), observer_read.next())
        .await
        .is_ok()
    {}

    // New connection should be able to send messages
    let msg = json!({
        "type": "message",
        "id": "reconnect-msg",
        "chat_id": "chat1",
        "sender_id": "reconnecting_user",
        "sender_name": "Reconnector",
        "content": "I reconnected!",
        "timestamp": 1234567890
    });
    write2
        .send(Message::Text(msg.to_string().into()))
        .await
        .expect("Should be able to send from new connection");

    // Observer should receive the message
    let received = read_message_of_type(&mut observer_read, "message", 5)
        .await
        .expect("Observer should receive message from reconnected user");

    assert_eq!(received["sender_id"], "reconnecting_user");
}

#[tokio::test]
async fn e2e_new_client_receives_existing_online_users() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect first client (Alice)
    let _client1 = connect_and_auth(&server.url(), "alice")
        .await
        .expect("Failed to connect alice");

    // Small delay to ensure alice is registered
    sleep(Duration::from_millis(100)).await;

    // Connect second client (Bob) - should receive alice's online presence
    let client2 = connect_and_auth(&server.url(), "bob")
        .await
        .expect("Failed to connect bob");
    let (_, mut read2) = client2.split();

    // Bob should receive alice's presence (existing online user)
    let received = read_message_of_type(&mut read2, "presence", 5)
        .await
        .expect("Bob should receive alice's presence");

    assert_eq!(received["user_id"], "alice");
    assert_eq!(received["is_online"], true);
}

#[tokio::test]
async fn e2e_bidirectional_presence_sync() {
    let port = get_unique_port();
    let server = ServerProcess::start(port).expect("Failed to start server");
    server.wait_until_ready().await.expect("Server not ready");

    // Connect first client (Alice)
    let client1 = connect_and_auth(&server.url(), "alice")
        .await
        .expect("Failed to connect alice");
    let (_, mut read1) = client1.split();

    // Connect second client (Bob)
    let client2 = connect_and_auth(&server.url(), "bob")
        .await
        .expect("Failed to connect bob");
    let (_, mut read2) = client2.split();

    // Alice should receive bob's online presence (bob just connected)
    let alice_sees_bob = read_message_of_type(&mut read1, "presence", 5)
        .await
        .expect("Alice should receive bob's presence");
    assert_eq!(alice_sees_bob["user_id"], "bob");
    assert_eq!(alice_sees_bob["is_online"], true);

    // Bob should receive alice's presence (existing online user sent on connect)
    let bob_sees_alice = read_message_of_type(&mut read2, "presence", 5)
        .await
        .expect("Bob should receive alice's presence");
    assert_eq!(bob_sees_alice["user_id"], "alice");
    assert_eq!(bob_sees_alice["is_online"], true);
}
