use pulse_server::{ServerState, WsMessage, handle_message};
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::test]
async fn test_sender_spoofing_protection() {
    let state = Arc::new(ServerState::new());
    let (tx, mut rx) = mpsc::unbounded_channel();
    state.add_client("victim".to_string(), tx);

    // Attacker "attacker" tries to send a message as "admin"
    let spoofed_json = r#"{
        "type": "message",
        "id": "1",
        "chat_id": "c1",
        "sender_id": "admin", 
        "sender_name": "Admin",
        "recipient_id": "victim",
        "content": "Click this link",
        "timestamp": 123
    }"#;

    // "attacker" is the authenticated connection
    handle_message(spoofed_json, "attacker", &state);

    // Check what "victim" received
    if let Some(msg_str) = rx.recv().await {
        let msg: WsMessage = serde_json::from_str(&msg_str).unwrap();
        if let WsMessage::ChatMessage { sender_id, .. } = msg {
            assert_eq!(sender_id, "attacker", "Sender ID should have been overwritten to 'attacker'");
            assert_ne!(sender_id, "admin", "Spoofed sender ID 'admin' persisted!");
        } else {
            panic!("Expected ChatMessage");
        }
    } else {
        panic!("Victim received nothing");
    }
}

#[tokio::test]
async fn test_receipt_routing_protection() {
     let state = Arc::new(ServerState::new());
    let (tx, mut rx) = mpsc::unbounded_channel();
    // The original sender of the message who expects a receipt
    state.add_client("user_origin".to_string(), tx);

    // User "reader" sends a read receipt.
    // They claim they are "reader" but they put "user_origin" as sender_id (destination)
    // And they try to spoof "user_id" (who read it) as "someone_else"
    let spoofed_receipt = r#"{
        "type": "read_receipt",
        "chat_id": "c1",
        "sender_id": "user_origin",
        "user_id": "someone_else",
        "message_ids": ["m1"]
    }"#;
    
    // "reader" is the authenticated connection
    handle_message(spoofed_receipt, "reader", &state);
    
    // Check what "user_origin" received
    if let Some(msg_str) = rx.recv().await {
         let msg: WsMessage = serde_json::from_str(&msg_str).unwrap();
         if let WsMessage::ReadReceipt { user_id, .. } = msg {
             assert_eq!(user_id, "reader", "Reader USER_ID should be overwritten to 'reader'");
             assert_ne!(user_id, "someone_else", "Spoofed reader ID persisted!");
         } else {
             panic!("Expected ReadReceipt");
         }
    } else {
         panic!("Origin received nothing");
    }
}
