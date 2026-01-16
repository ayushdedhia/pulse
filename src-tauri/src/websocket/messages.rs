use serde::{Deserialize, Serialize};

/// URL preview data for WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsUrlPreview {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
}

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
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url_preview: Option<WsUrlPreview>,
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
    /// Profile update broadcast to peers
    #[serde(rename = "profile_update")]
    ProfileUpdate {
        user_id: String,
        name: String,
        phone: Option<String>,
        avatar_url: Option<String>,
        about: Option<String>,
        /// Base64-encoded avatar image bytes (only when avatar changes)
        avatar_data: Option<String>,
    },

    #[serde(rename = "call_invite")]
    CallInvite {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
        kind: String, // "video" or "audio"
    },
    #[serde(rename = "call_ringing")]
    CallRinging {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
    },
    #[serde(rename = "call_accept")]
    CallAccept {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
    },
    #[serde(rename = "call_reject")]
    CallReject {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
        reason: String,
    },
    #[serde(rename = "call_hangup")]
    CallHangup {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
    },

    #[serde(rename = "rtc_offer")]
    RtcOffer {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
        sdp: String,
    },
    #[serde(rename = "rtc_answer")]
    RtcAnswer {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
        sdp: String,
    },
    #[serde(rename = "rtc_ice_candidate")]
    RtcIceCandidate {
        call_id: String,
        from_user_id: String,
        to_user_id: String,
        candidate: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_message_roundtrip() {
        let msg = WsMessage::Connect {
            user_id: "user123".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::Connect { user_id } = parsed {
            assert_eq!(user_id, "user123");
        } else {
            panic!("Expected Connect");
        }
    }

    #[test]
    fn test_chat_message_roundtrip() {
        let msg = WsMessage::ChatMessage {
            id: "msg1".to_string(),
            chat_id: "chat1".to_string(),
            sender_id: "user1".to_string(),
            sender_name: "Alice".to_string(),
            recipient_id: "user2".to_string(),
            content: "Hello!".to_string(),
            timestamp: 1234567890,
            reply_to_id: None,
            url_preview: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::ChatMessage {
            id,
            content,
            sender_name,
            ..
        } = parsed
        {
            assert_eq!(id, "msg1");
            assert_eq!(content, "Hello!");
            assert_eq!(sender_name, "Alice");
        } else {
            panic!("Expected ChatMessage");
        }
    }

    #[test]
    fn test_typing_message_roundtrip() {
        let msg = WsMessage::Typing {
            chat_id: "chat1".to_string(),
            user_id: "user1".to_string(),
            is_typing: true,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::Typing {
            chat_id,
            user_id,
            is_typing,
        } = parsed
        {
            assert_eq!(chat_id, "chat1");
            assert_eq!(user_id, "user1");
            assert!(is_typing);
        } else {
            panic!("Expected Typing");
        }
    }

    #[test]
    fn test_presence_message_roundtrip() {
        let msg = WsMessage::Presence {
            user_id: "user1".to_string(),
            is_online: true,
            last_seen: Some(1234567890),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::Presence {
            user_id,
            is_online,
            last_seen,
        } = parsed
        {
            assert_eq!(user_id, "user1");
            assert!(is_online);
            assert_eq!(last_seen, Some(1234567890));
        } else {
            panic!("Expected Presence");
        }
    }

    #[test]
    fn test_delivery_receipt_roundtrip() {
        let msg = WsMessage::DeliveryReceipt {
            message_id: "msg1".to_string(),
            chat_id: "chat1".to_string(),
            sender_id: "user1".to_string(),
            delivered_to: "user2".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::DeliveryReceipt {
            message_id,
            chat_id,
            sender_id,
            delivered_to,
        } = parsed
        {
            assert_eq!(message_id, "msg1");
            assert_eq!(chat_id, "chat1");
            assert_eq!(sender_id, "user1");
            assert_eq!(delivered_to, "user2");
        } else {
            panic!("Expected DeliveryReceipt");
        }
    }

    #[test]
    fn test_read_receipt_roundtrip() {
        let msg = WsMessage::ReadReceipt {
            chat_id: "chat1".to_string(),
            sender_id: "user2".to_string(),
            user_id: "user1".to_string(),
            message_ids: vec!["msg1".to_string(), "msg2".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::ReadReceipt {
            chat_id,
            sender_id,
            user_id,
            message_ids,
        } = parsed
        {
            assert_eq!(chat_id, "chat1");
            assert_eq!(sender_id, "user2");
            assert_eq!(user_id, "user1");
            assert_eq!(message_ids, vec!["msg1", "msg2"]);
        } else {
            panic!("Expected ReadReceipt");
        }
    }

    #[test]
    fn test_auth_response_roundtrip() {
        let msg = WsMessage::AuthResponse {
            success: true,
            message: "Connected".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::AuthResponse { success, message } = parsed {
            assert!(success);
            assert_eq!(message, "Connected");
        } else {
            panic!("Expected AuthResponse");
        }
    }

    #[test]
    fn test_profile_update_roundtrip() {
        let msg = WsMessage::ProfileUpdate {
            user_id: "user1".to_string(),
            name: "Alice".to_string(),
            phone: Some("+1234567890".to_string()),
            avatar_url: None,
            about: Some("Hello!".to_string()),
            avatar_data: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        if let WsMessage::ProfileUpdate {
            user_id,
            name,
            phone,
            avatar_url,
            about,
            avatar_data,
        } = parsed
        {
            assert_eq!(user_id, "user1");
            assert_eq!(name, "Alice");
            assert_eq!(phone, Some("+1234567890".to_string()));
            assert!(avatar_url.is_none());
            assert_eq!(about, Some("Hello!".to_string()));
            assert!(avatar_data.is_none());
        } else {
            panic!("Expected ProfileUpdate");
        }
    }

    #[test]
    fn test_server_client_message_compatibility() {
        // Test that messages serialized by server can be parsed by client
        // Simulate server sending auth_response
        let server_json = r#"{"type":"auth_response","success":true,"message":"Connected to server"}"#;
        let parsed: WsMessage = serde_json::from_str(server_json).unwrap();

        if let WsMessage::AuthResponse { success, .. } = parsed {
            assert!(success);
        } else {
            panic!("Expected AuthResponse");
        }

        // Simulate server sending presence
        let server_json = r#"{"type":"presence","user_id":"user2","is_online":true,"last_seen":null}"#;
        let parsed: WsMessage = serde_json::from_str(server_json).unwrap();

        if let WsMessage::Presence {
            user_id, is_online, ..
        } = parsed
        {
            assert_eq!(user_id, "user2");
            assert!(is_online);
        } else {
            panic!("Expected Presence");
        }
    }
}
