//! Input validation constants and utilities
//!
//! Note: Most validation is now handled by garde in models/input.rs.
//! These functions remain for use in commands that don't use DTOs (e.g., receive_message).

/// Maximum lengths for various fields
pub const MAX_USER_NAME_LENGTH: usize = 100;
pub const MAX_PHONE_LENGTH: usize = 20;
pub const MIN_PHONE_DIGITS: usize = 7;
pub const MAX_PHONE_DIGITS: usize = 15;
pub const MAX_ABOUT_LENGTH: usize = 500;
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

/// Validate a phone number for use as a unique user ID
/// Format: optional + followed by 7-15 digits
/// Strips spaces, dashes, and parentheses before validation
pub fn validate_phone_id(phone: &str) -> Result<String, String> {
    if phone.is_empty() {
        return Err("Phone number cannot be empty".to_string());
    }

    // Normalize: remove spaces, dashes, parentheses
    let normalized: String = phone
        .chars()
        .filter(|c| !matches!(*c, ' ' | '-' | '(' | ')'))
        .collect();

    if normalized.is_empty() {
        return Err("Phone number cannot be empty".to_string());
    }

    // Check for valid format: optional + followed by digits only
    let (has_plus, digits_part) = if normalized.starts_with('+') {
        (true, &normalized[1..])
    } else {
        (false, normalized.as_str())
    };

    // All remaining characters must be digits
    if !digits_part.chars().all(|c| c.is_ascii_digit()) {
        return Err("Phone number can only contain digits (and optional leading +)".to_string());
    }

    // Check digit count
    let digit_count = digits_part.len();
    if digit_count < MIN_PHONE_DIGITS {
        return Err(format!(
            "Phone number too short (min {} digits)",
            MIN_PHONE_DIGITS
        ));
    }
    if digit_count > MAX_PHONE_DIGITS {
        return Err(format!(
            "Phone number too long (max {} digits)",
            MAX_PHONE_DIGITS
        ));
    }

    // Return normalized form (with + if originally present)
    Ok(if has_plus {
        format!("+{}", digits_part)
    } else {
        digits_part.to_string()
    })
}

/// Validate a phone number (optional field, less strict)
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== validate_phone_id tests ====================

    #[test]
    fn test_valid_phone_ids() {
        // Standard formats
        assert!(validate_phone_id("+1234567890").is_ok());
        assert!(validate_phone_id("1234567890").is_ok());
        assert!(validate_phone_id("+919876543210").is_ok());
        assert!(validate_phone_id("9876543210").is_ok());

        // International formats
        assert!(validate_phone_id("+14155552671").is_ok()); // US
        assert!(validate_phone_id("+442071234567").is_ok()); // UK
        assert!(validate_phone_id("+81312345678").is_ok()); // Japan
    }

    #[test]
    fn test_phone_id_normalization() {
        // Spaces should be stripped
        let result = validate_phone_id("+1 234 567 8901");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "+12345678901");

        // Dashes should be stripped
        let result = validate_phone_id("+1-234-567-8901");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "+12345678901");

        // Parentheses should be stripped
        let result = validate_phone_id("+1 (234) 567-8901");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "+12345678901");

        // Without plus
        let result = validate_phone_id("(234) 567-8901");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2345678901");
    }

    #[test]
    fn test_phone_id_empty() {
        let result = validate_phone_id("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));

        // Only whitespace/formatting chars
        let result = validate_phone_id("   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));

        let result = validate_phone_id("---");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_phone_id_too_short() {
        // Less than 7 digits
        assert!(validate_phone_id("123456").is_err());
        assert!(validate_phone_id("+123456").is_err());

        let result = validate_phone_id("12345");
        assert!(result.unwrap_err().contains("too short"));

        // Exactly 7 digits should be ok
        assert!(validate_phone_id("1234567").is_ok());
        assert!(validate_phone_id("+1234567").is_ok());
    }

    #[test]
    fn test_phone_id_too_long() {
        // More than 15 digits
        assert!(validate_phone_id("1234567890123456").is_err());
        assert!(validate_phone_id("+1234567890123456").is_err());

        let result = validate_phone_id("1234567890123456");
        assert!(result.unwrap_err().contains("too long"));

        // Exactly 15 digits should be ok
        assert!(validate_phone_id("123456789012345").is_ok());
        assert!(validate_phone_id("+123456789012345").is_ok());
    }

    #[test]
    fn test_phone_id_invalid_characters() {
        // Letters not allowed
        let result = validate_phone_id("+1234567890a");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only contain digits"));

        assert!(validate_phone_id("abc1234567").is_err());
        assert!(validate_phone_id("123abc4567").is_err());

        // Special characters not allowed (except formatting)
        assert!(validate_phone_id("+123#456*7890").is_err());
        assert!(validate_phone_id("123.456.7890").is_err());
        assert!(validate_phone_id("123/456/7890").is_err());
    }

    #[test]
    fn test_phone_id_plus_position() {
        // Plus only at start
        assert!(validate_phone_id("+1234567890").is_ok());

        // Plus in middle - not allowed
        let result = validate_phone_id("123+4567890");
        assert!(result.is_err());

        // Multiple pluses - not allowed
        let result = validate_phone_id("++1234567890");
        assert!(result.is_err());
    }

    #[test]
    fn test_phone_id_boundary_cases() {
        // Exactly min digits (7)
        assert!(validate_phone_id("1234567").is_ok());
        assert_eq!(validate_phone_id("1234567").unwrap(), "1234567");

        // One below min
        assert!(validate_phone_id("123456").is_err());

        // Exactly max digits (15)
        assert!(validate_phone_id("123456789012345").is_ok());
        assert_eq!(
            validate_phone_id("123456789012345").unwrap(),
            "123456789012345"
        );

        // One above max
        assert!(validate_phone_id("1234567890123456").is_err());

        // With plus at boundaries
        assert!(validate_phone_id("+1234567").is_ok());
        assert!(validate_phone_id("+123456789012345").is_ok());
    }

    #[test]
    fn test_phone_id_real_world_formats() {
        // US formats
        assert!(validate_phone_id("+1 (555) 123-4567").is_ok());
        assert!(validate_phone_id("(555) 123-4567").is_ok());
        assert!(validate_phone_id("555-123-4567").is_ok());

        // Indian format
        assert!(validate_phone_id("+91 98765 43210").is_ok());

        // UK format
        assert!(validate_phone_id("+44 20 7123 4567").is_ok());

        // German format
        assert!(validate_phone_id("+49 30 12345678").is_ok());
    }

    #[test]
    fn test_phone_id_preserves_plus() {
        // With plus should keep plus
        let result = validate_phone_id("+1234567890");
        assert_eq!(result.unwrap(), "+1234567890");

        // Without plus should not add plus
        let result = validate_phone_id("1234567890");
        assert_eq!(result.unwrap(), "1234567890");
    }
}
