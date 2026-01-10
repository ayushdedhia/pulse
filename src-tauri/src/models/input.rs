//! Input DTOs with garde validation for Tauri commands.
//!
//! These structs validate frontend data before processing.

use garde::Validate;
use serde::Deserialize;

/// Validation constants
const MAX_USER_ID_LENGTH: usize = 128;
const MAX_CHAT_ID_LENGTH: usize = 256;
const MAX_MESSAGE_LENGTH: usize = 10000;
const MAX_SEARCH_QUERY_LENGTH: usize = 200;

/// Custom validation for message type
fn validate_message_type(value: &str, _ctx: &()) -> garde::Result {
    match value {
        "text" | "image" | "audio" | "video" | "document" => Ok(()),
        _ => Err(garde::Error::new("Invalid message type")),
    }
}

/// Custom validation for message status
fn validate_message_status(value: &str, _ctx: &()) -> garde::Result {
    match value {
        "sent" | "delivered" | "read" => Ok(()),
        _ => Err(garde::Error::new("Invalid message status")),
    }
}

/// Input for creating a new chat
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreateChatInput {
    #[garde(length(min = 1, max = MAX_USER_ID_LENGTH))]
    pub user_id: String,
}

/// Input for getting messages from a chat
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct GetMessagesInput {
    #[garde(length(min = 1, max = MAX_CHAT_ID_LENGTH))]
    pub chat_id: String,
    #[garde(range(min = 1, max = 1000))]
    pub limit: i32,
    #[garde(range(min = 0))]
    pub offset: i32,
}

/// Input for sending a message
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct SendMessageInput {
    #[garde(length(min = 1, max = MAX_CHAT_ID_LENGTH))]
    pub chat_id: String,
    #[garde(length(min = 1, max = MAX_MESSAGE_LENGTH))]
    pub content: String,
    #[garde(custom(validate_message_type))]
    pub message_type: String,
}

/// Input for marking messages as read
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct MarkAsReadInput {
    #[garde(length(min = 1, max = MAX_CHAT_ID_LENGTH))]
    pub chat_id: String,
}

/// Input for updating message status
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdateMessageStatusInput {
    #[garde(length(min = 1, max = MAX_USER_ID_LENGTH))]
    pub message_id: String,
    #[garde(custom(validate_message_status))]
    pub status: String,
}

/// Input for searching messages
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct SearchMessagesInput {
    #[garde(length(min = 1, max = MAX_SEARCH_QUERY_LENGTH))]
    pub query: String,
}

/// Helper trait to convert garde validation errors to String
pub trait ValidateExt {
    fn validate_input(&self) -> Result<(), String>;
}

impl<T: Validate<Context = ()>> ValidateExt for T {
    fn validate_input(&self) -> Result<(), String> {
        self.validate().map_err(|e| e.to_string())
    }
}
