//! Integration tests for the Pulse WebSocket server
//!
//! These tests spin up a real server and connect clients to verify
//! message routing, presence, and broadcasting work correctly.

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Start a test server on a random available port
async fn start_test_server() -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let state = std::sync::Arc::new(pulse_server::ServerState::new());

    let handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let state = state.clone();
            tokio::spawn(async move {
                pulse_server::handle_connection(ws_stream, state).await;
            });
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    (port, handle)
}

/// Connect a client to the server and authenticate
async fn connect_client(
    port: u16,
    user_id: &str,
) -> tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
> {
    let url = format!("ws://127.0.0.1:{}", port);
    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

    let (mut write, mut read) = ws_stream.split();

    // Send connect message
    let connect_msg = json!({
        "type": "connect",
        "user_id": user_id
    });
    write
        .send(Message::Text(connect_msg.to_string().into()))
        .await
        .unwrap();

    // Wait for auth response
    let response = timeout(Duration::from_secs(5), read.next())
        .await
        .expect("Timeout waiting for auth")
        .expect("Stream closed")
        .expect("Read error");

    if let Message::Text(text) = response {
        let msg: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(msg["type"], "auth_response");
        assert_eq!(msg["success"], true);
    } else {
        panic!("Expected text message");
    }

    // Reunite the stream
    write.reunite(read).unwrap()
}

#[tokio::test]
async fn test_client_connects_and_authenticates() {
    let (port, server_handle) = start_test_server().await;

    let _client = connect_client(port, "user1").await;

    server_handle.abort();
}

#[tokio::test]
async fn test_presence_broadcast_on_connect() {
    let (port, server_handle) = start_test_server().await;

    // Connect first client
    let client1 = connect_client(port, "user1").await;
    let (_, mut read1) = client1.split();

    // Connect second client
    let _client2 = connect_client(port, "user2").await;

    // Client 1 should receive presence notification for client 2
    let msg = timeout(Duration::from_secs(5), read1.next())
        .await
        .expect("Timeout waiting for presence")
        .expect("Stream closed")
        .expect("Read error");

    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "presence");
        assert_eq!(parsed["user_id"], "user2");
        assert_eq!(parsed["is_online"], true);
    } else {
        panic!("Expected text message");
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_message_broadcast() {
    let (port, server_handle) = start_test_server().await;

    // Connect two clients
    let client1 = connect_client(port, "user1").await;
    let client2 = connect_client(port, "user2").await;

    let (mut write1, _read1) = client1.split();
    let (_write2, mut read2) = client2.split();

    // Drain presence notification from client2's perspective
    let _ = timeout(Duration::from_millis(200), read2.next()).await;

    // User1 sends a message to user2
    let chat_msg = json!({
        "type": "message",
        "id": "msg1",
        "chat_id": "chat1",
        "sender_id": "user1",
        "sender_name": "Alice",
        "recipient_id": "user2",
        "content": "Hello from user1!",
        "timestamp": 1234567890
    });
    write1
        .send(Message::Text(chat_msg.to_string().into()))
        .await
        .unwrap();

    // User2 should receive the message
    let msg = timeout(Duration::from_secs(5), read2.next())
        .await
        .expect("Timeout waiting for message")
        .expect("Stream closed")
        .expect("Read error");

    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "message");
        assert_eq!(parsed["sender_id"], "user1");
        assert_eq!(parsed["content"], "Hello from user1!");
    } else {
        panic!("Expected text message");
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_sender_does_not_receive_own_message() {
    let (port, server_handle) = start_test_server().await;

    let client1 = connect_client(port, "user1").await;
    let (mut write1, mut read1) = client1.split();

    // User1 sends a message to user2 (who is offline)
    let chat_msg = json!({
        "type": "message",
        "id": "msg1",
        "chat_id": "chat1",
        "sender_id": "user1",
        "sender_name": "Alice",
        "recipient_id": "user2",
        "content": "Test message",
        "timestamp": 1234567890
    });
    write1
        .send(Message::Text(chat_msg.to_string().into()))
        .await
        .unwrap();

    // User1 should NOT receive their own message back
    let result = timeout(Duration::from_millis(500), read1.next()).await;

    // Should timeout because no message should be received
    assert!(result.is_err(), "Sender should not receive their own message");

    server_handle.abort();
}

#[tokio::test]
async fn test_typing_indicator_broadcast() {
    let (port, server_handle) = start_test_server().await;

    let client1 = connect_client(port, "user1").await;
    let client2 = connect_client(port, "user2").await;

    let (mut write1, _) = client1.split();
    let (_, mut read2) = client2.split();

    // Drain presence notification
    let _ = timeout(Duration::from_millis(200), read2.next()).await;

    // User1 starts typing
    let typing_msg = json!({
        "type": "typing",
        "chat_id": "chat1",
        "user_id": "user1",
        "is_typing": true
    });
    write1
        .send(Message::Text(typing_msg.to_string().into()))
        .await
        .unwrap();

    // User2 should receive typing indicator
    let msg = timeout(Duration::from_secs(5), read2.next())
        .await
        .expect("Timeout")
        .expect("Closed")
        .expect("Error");

    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "typing");
        assert_eq!(parsed["user_id"], "user1");
        assert_eq!(parsed["is_typing"], true);
    } else {
        panic!("Expected text message");
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_delivery_receipt_routed_to_sender() {
    let (port, server_handle) = start_test_server().await;

    let client1 = connect_client(port, "user1").await;
    let client2 = connect_client(port, "user2").await;

    let (_, mut read1) = client1.split();
    let (mut write2, _) = client2.split();

    // Drain presence notification from user1's perspective
    let _ = timeout(Duration::from_millis(200), read1.next()).await;

    // User2 sends delivery receipt back to user1 (the original sender)
    let receipt = json!({
        "type": "delivery_receipt",
        "message_id": "msg1",
        "chat_id": "chat1",
        "sender_id": "user1",
        "delivered_to": "user2"
    });
    write2
        .send(Message::Text(receipt.to_string().into()))
        .await
        .unwrap();

    // User1 should receive the delivery receipt
    let msg = timeout(Duration::from_secs(5), read1.next())
        .await
        .expect("Timeout")
        .expect("Closed")
        .expect("Error");

    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "delivery_receipt");
        assert_eq!(parsed["message_id"], "msg1");
        assert_eq!(parsed["delivered_to"], "user2");
    } else {
        panic!("Expected text message");
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_offline_presence_on_disconnect() {
    let (port, server_handle) = start_test_server().await;

    // Connect first client
    let client1 = connect_client(port, "user1").await;
    let (_, mut read1) = client1.split();

    // Connect and then disconnect second client
    let client2 = connect_client(port, "user2").await;

    // Drain online presence notification
    let _ = timeout(Duration::from_millis(200), read1.next()).await;

    // Disconnect client2
    drop(client2);

    // Client 1 should receive offline presence for client 2
    let msg = timeout(Duration::from_secs(5), read1.next())
        .await
        .expect("Timeout waiting for offline presence")
        .expect("Stream closed")
        .expect("Read error");

    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "presence");
        assert_eq!(parsed["user_id"], "user2");
        assert_eq!(parsed["is_online"], false);
        assert!(parsed["last_seen"].is_number());
    } else {
        panic!("Expected text message");
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_message_routed_to_specific_recipient() {
    let (port, server_handle) = start_test_server().await;

    // Connect three clients
    let client1 = connect_client(port, "user1").await;
    let client2 = connect_client(port, "user2").await;
    let client3 = connect_client(port, "user3").await;

    let (mut write1, _) = client1.split();
    let (_, mut read2) = client2.split();
    let (_, mut read3) = client3.split();

    // Drain presence notifications
    tokio::time::sleep(Duration::from_millis(200)).await;
    while timeout(Duration::from_millis(50), read2.next()).await.is_ok() {}
    while timeout(Duration::from_millis(50), read3.next()).await.is_ok() {}

    // User1 sends a message specifically to user2
    let chat_msg = json!({
        "type": "message",
        "id": "msg1",
        "chat_id": "chat1",
        "sender_id": "user1",
        "sender_name": "Alice",
        "recipient_id": "user2",
        "content": "Only for user2",
        "timestamp": 1234567890
    });
    write1
        .send(Message::Text(chat_msg.to_string().into()))
        .await
        .unwrap();

    // User2 should receive the message
    let msg2 = timeout(Duration::from_secs(5), read2.next())
        .await
        .expect("Timeout")
        .expect("Closed")
        .expect("Error");

    if let Message::Text(text) = msg2 {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "message");
        assert_eq!(parsed["content"], "Only for user2");
    } else {
        panic!("Expected text message");
    }

    // User3 should NOT receive the message (it was routed to user2 only)
    let result = timeout(Duration::from_millis(500), read3.next()).await;
    assert!(
        result.is_err(),
        "User3 should not receive message intended for user2"
    );

    server_handle.abort();
}

#[tokio::test]
async fn test_profile_update_broadcast() {
    let (port, server_handle) = start_test_server().await;

    let client1 = connect_client(port, "user1").await;
    let client2 = connect_client(port, "user2").await;

    let (mut write1, _) = client1.split();
    let (_, mut read2) = client2.split();

    // Drain presence notification
    let _ = timeout(Duration::from_millis(200), read2.next()).await;

    // User1 updates their profile
    let profile_update = json!({
        "type": "profile_update",
        "user_id": "user1",
        "name": "Alice Updated",
        "phone": "+1234567890",
        "avatar_url": null,
        "about": "New status",
        "avatar_data": null
    });
    write1
        .send(Message::Text(profile_update.to_string().into()))
        .await
        .unwrap();

    // User2 should receive the profile update
    let msg = timeout(Duration::from_secs(5), read2.next())
        .await
        .expect("Timeout")
        .expect("Closed")
        .expect("Error");

    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "profile_update");
        assert_eq!(parsed["name"], "Alice Updated");
        assert_eq!(parsed["about"], "New status");
    } else {
        panic!("Expected text message");
    }

    server_handle.abort();
}
