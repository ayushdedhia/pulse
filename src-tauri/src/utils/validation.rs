/// Input validation constants and utilities

/// Maximum lengths for various fields
pub const MAX_USER_NAME_LENGTH: usize = 100;
pub const MAX_USER_ID_LENGTH: usize = 128;
pub const MAX_PHONE_LENGTH: usize = 20;
pub const MAX_ABOUT_LENGTH: usize = 500;
pub const MAX_MESSAGE_LENGTH: usize = 10000;
pub const MAX_CHAT_ID_LENGTH: usize = 256;
pub const MAX_AVATAR_URL_LENGTH: usize = 2048;

/// Validate a user name
pub fn validate_user_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.len() > MAX_USER_NAME_LENGTH {
        return Err(format!(
            "Name too long (max {} characters)",
            MAX_USER_NAME_LENGTH
        ));
    }
    // Check for control characters
    if name.chars().any(|c| c.is_control()) {
        return Err("Name contains invalid characters".to_string());
    }
    Ok(())
}

/// Validate a user ID
pub fn validate_user_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("User ID cannot be empty".to_string());
    }
    if id.len() > MAX_USER_ID_LENGTH {
        return Err(format!(
            "User ID too long (max {} characters)",
            MAX_USER_ID_LENGTH
        ));
    }
    // Check for control characters
    if id.chars().any(|c| c.is_control()) {
        return Err("User ID contains invalid characters".to_string());
    }
    Ok(())
}

/// Validate a phone number
pub fn validate_phone(phone: Option<&str>) -> Result<(), String> {
    if let Some(p) = phone {
        if p.len() > MAX_PHONE_LENGTH {
            return Err(format!(
                "Phone number too long (max {} characters)",
                MAX_PHONE_LENGTH
            ));
        }
        // Only allow digits, plus, spaces, parentheses, and hyphens
        if !p.chars().all(|c| c.is_ascii_digit() || " +()-".contains(c)) {
            return Err("Phone number contains invalid characters".to_string());
        }
    }
    Ok(())
}

/// Validate message content
pub fn validate_message(content: &str) -> Result<(), String> {
    if content.is_empty() {
        return Err("Message cannot be empty".to_string());
    }
    if content.len() > MAX_MESSAGE_LENGTH {
        return Err(format!(
            "Message too long (max {} characters)",
            MAX_MESSAGE_LENGTH
        ));
    }
    Ok(())
}

/// Validate a chat ID
pub fn validate_chat_id(chat_id: &str) -> Result<(), String> {
    if chat_id.is_empty() {
        return Err("Chat ID cannot be empty".to_string());
    }
    if chat_id.len() > MAX_CHAT_ID_LENGTH {
        return Err(format!(
            "Chat ID too long (max {} characters)",
            MAX_CHAT_ID_LENGTH
        ));
    }
    Ok(())
}

/// Validate about/status text
pub fn validate_about(about: Option<&str>) -> Result<(), String> {
    if let Some(a) = about {
        if a.len() > MAX_ABOUT_LENGTH {
            return Err(format!(
                "About text too long (max {} characters)",
                MAX_ABOUT_LENGTH
            ));
        }
    }
    Ok(())
}

/// Validate URL
pub fn validate_url(url: Option<&str>) -> Result<(), String> {
    if let Some(u) = url {
        if u.len() > MAX_AVATAR_URL_LENGTH {
            return Err(format!(
                "URL too long (max {} characters)",
                MAX_AVATAR_URL_LENGTH
            ));
        }
    }
    Ok(())
}
