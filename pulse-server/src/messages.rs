use serde::{Deserialize, Serialize};

/// WebSocket message types (shared between server and client)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "message")]
    ChatMessage {
        id: String,
        chat_id: String,
        sender_id: String,
        sender_name: String,
        recipient_id: String,
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
    #[serde(rename = "delivery_receipt")]
    DeliveryReceipt {
        message_id: String,
        chat_id: String,
        sender_id: String,
        delivered_to: String,
    },
    #[serde(rename = "read_receipt")]
    ReadReceipt {
        chat_id: String,
        sender_id: String,
        user_id: String,
        message_ids: Vec<String>,
    },
    #[serde(rename = "connect")]
    Connect { user_id: String },
    #[serde(rename = "auth_response")]
    AuthResponse { success: bool, message: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "profile_update")]
    ProfileUpdate {
        user_id: String,
        name: String,
        phone: Option<String>,
        avatar_url: Option<String>,
        about: Option<String>,
        avatar_data: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_message_serialization() {
        let msg = WsMessage::Connect {
            user_id: "user123".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"connect\""));
        assert!(json.contains("\"user_id\":\"user123\""));

        // Deserialize back
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        if let WsMessage::Connect { user_id } = parsed {
            assert_eq!(user_id, "user123");
        } else {
            panic!("Expected Connect message");
        }
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = WsMessage::ChatMessage {
            id: "msg1".to_string(),
            chat_id: "chat1".to_string(),
            sender_id: "user1".to_string(),
            sender_name: "Alice".to_string(),
            recipient_id: "user2".to_string(),
            content: "Hello, world!".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"message\""));
        assert!(json.contains("\"id\":\"msg1\""));
        assert!(json.contains("\"content\":\"Hello, world!\""));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        if let WsMessage::ChatMessage { id, content, .. } = parsed {
            assert_eq!(id, "msg1");
            assert_eq!(content, "Hello, world!");
        } else {
            panic!("Expected ChatMessage");
        }
    }

    #[test]
    fn test_typing_message_serialization() {
        let msg = WsMessage::Typing {
            chat_id: "chat1".to_string(),
            user_id: "user1".to_string(),
            is_typing: true,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"typing\""));
        assert!(json.contains("\"is_typing\":true"));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        if let WsMessage::Typing { is_typing, .. } = parsed {
            assert!(is_typing);
        } else {
            panic!("Expected Typing message");
        }
    }

    #[test]
    fn test_presence_message_serialization() {
        // Online presence
        let msg = WsMessage::Presence {
            user_id: "user1".to_string(),
            is_online: true,
            last_seen: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"presence\""));
        assert!(json.contains("\"is_online\":true"));

        // Offline presence with last_seen
        let msg = WsMessage::Presence {
            user_id: "user1".to_string(),
            is_online: false,
            last_seen: Some(1234567890),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"is_online\":false"));
        assert!(json.contains("\"last_seen\":1234567890"));
    }

    #[test]
    fn test_delivery_receipt_serialization() {
        let msg = WsMessage::DeliveryReceipt {
            message_id: "msg1".to_string(),
            chat_id: "chat1".to_string(),
            sender_id: "user1".to_string(),
            delivered_to: "user2".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"delivery_receipt\""));
        assert!(json.contains("\"message_id\":\"msg1\""));
        assert!(json.contains("\"delivered_to\":\"user2\""));
    }

    #[test]
    fn test_read_receipt_serialization() {
        let msg = WsMessage::ReadReceipt {
            chat_id: "chat1".to_string(),
            sender_id: "user2".to_string(),
            user_id: "user1".to_string(),
            message_ids: vec!["msg1".to_string(), "msg2".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"read_receipt\""));
        assert!(json.contains("\"message_ids\":[\"msg1\",\"msg2\"]"));
    }

    #[test]
    fn test_auth_response_serialization() {
        let msg = WsMessage::AuthResponse {
            success: true,
            message: "Connected".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"auth_response\""));
        assert!(json.contains("\"success\":true"));

        let msg = WsMessage::AuthResponse {
            success: false,
            message: "Invalid user".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"success\":false"));
    }

    #[test]
    fn test_error_message_serialization() {
        let msg = WsMessage::Error {
            message: "Something went wrong".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"message\":\"Something went wrong\""));
    }

    #[test]
    fn test_profile_update_serialization() {
        let msg = WsMessage::ProfileUpdate {
            user_id: "user1".to_string(),
            name: "Alice".to_string(),
            phone: Some("+1234567890".to_string()),
            avatar_url: None,
            about: Some("Hello!".to_string()),
            avatar_data: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"profile_update\""));
        assert!(json.contains("\"name\":\"Alice\""));
        assert!(json.contains("\"phone\":\"+1234567890\""));
        assert!(json.contains("\"about\":\"Hello!\""));
    }

    #[test]
    fn test_deserialize_from_frontend_format() {
        // Test parsing JSON in the format the frontend sends
        let json = r#"{"type":"connect","user_id":"abc-123"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        if let WsMessage::Connect { user_id } = msg {
            assert_eq!(user_id, "abc-123");
        } else {
            panic!("Expected Connect");
        }

        let json = r#"{"type":"typing","chat_id":"chat1","user_id":"user1","is_typing":true}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        if let WsMessage::Typing {
            chat_id,
            user_id,
            is_typing,
        } = msg
        {
            assert_eq!(chat_id, "chat1");
            assert_eq!(user_id, "user1");
            assert!(is_typing);
        } else {
            panic!("Expected Typing");
        }
    }
}
