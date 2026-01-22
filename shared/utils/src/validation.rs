use crate::error::{ElementaError, ElementaResult};
use regex::Regex;
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

pub fn validate_model<T: Validate>(model: &T) -> ElementaResult<()> {
    match model.validate() {
        Ok(()) => Ok(()),
        Err(errors) => {
            let error_messages = format_validation_errors(&errors);
            Err(ElementaError::validation("model", error_messages))
        }
    }
}

pub fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = match &error.code {
                std::borrow::Cow::Borrowed("email") => "Invalid email format".to_string(),
                std::borrow::Cow::Borrowed("length") => {
                    format!("Length validation failed for field '{}'", field)
                }
                std::borrow::Cow::Borrowed("range") => {
                    format!("Value out of range for field '{}'", field)
                }
                std::borrow::Cow::Borrowed("required") => {
                    format!("Field '{}' is required", field)
                }
                _ => format!("Validation failed for field '{}': {}", field, error.code),
            };
            messages.push(message);
        }
    }
    
    messages.join(", ")
}

pub fn validate_cas_number(cas_number: &str) -> ElementaResult<()> {
    let cas_regex = Regex::new(r"^\d{2,7}-\d{2}-\d$").unwrap();
    
    if !cas_regex.is_match(cas_number) {
        return Err(ElementaError::validation(
            "cas_number",
            "Invalid CAS number format. Expected format: XXXXXX-XX-X",
        ));
    }
    
    // Validate check digit
    let digits: String = cas_number.replace('-', "");
    let check_digit = digits.chars().last().unwrap().to_digit(10).unwrap() as usize;
    
    let mut sum = 0;
    for (i, digit_char) in digits[..digits.len()-1].chars().rev().enumerate() {
        if let Some(digit) = digit_char.to_digit(10) {
            sum += (digit as usize) * (i + 1);
        }
    }
    
    if sum % 10 != check_digit {
        return Err(ElementaError::validation(
            "cas_number",
            "Invalid CAS number check digit",
        ));
    }
    
    Ok(())
}

pub fn validate_email_address(email: &str) -> ElementaResult<()> {
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
    
    if !email_regex.is_match(email) {
        return Err(ElementaError::validation(
            "email",
            "Invalid email address format",
        ));
    }
    
    Ok(())
}

pub fn validate_file_type(file_name: &str, allowed_types: &[&str]) -> ElementaResult<()> {
    let extension = std::path::Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    if !allowed_types.contains(&extension.to_lowercase().as_str()) {
        return Err(ElementaError::validation(
            "file_type",
            format!("File type '{}' not allowed. Allowed types: {}", extension, allowed_types.join(", ")),
        ));
    }
    
    Ok(())
}

pub fn validate_file_size(file_size: u64, max_size: u64) -> ElementaResult<()> {
    if file_size > max_size {
        return Err(ElementaError::validation(
            "file_size",
            format!("File size {} bytes exceeds maximum allowed size {} bytes", file_size, max_size),
        ));
    }
    
    Ok(())
}

pub fn validate_uuid(uuid_str: &str) -> ElementaResult<uuid::Uuid> {
    uuid::Uuid::parse_str(uuid_str)
        .map_err(|_| ElementaError::validation("uuid", "Invalid UUID format"))
}

pub fn validate_date_range(start_date: chrono::DateTime<chrono::Utc>, end_date: chrono::DateTime<chrono::Utc>) -> ElementaResult<()> {
    if start_date >= end_date {
        return Err(ElementaError::validation(
            "date_range",
            "Start date must be before end date",
        ));
    }
    
    Ok(())
}

pub fn validate_required_fields<T>(data: &HashMap<String, T>, required_fields: &[&str]) -> ElementaResult<()> {
    let missing_fields: Vec<&str> = required_fields
        .iter()
        .filter(|field| !data.contains_key(**field))
        .copied()
        .collect();
    
    if !missing_fields.is_empty() {
        return Err(ElementaError::validation(
            "required_fields",
            format!("Missing required fields: {}", missing_fields.join(", ")),
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cas_number_valid() {
        assert!(validate_cas_number("7732-18-5").is_ok()); // Water
        assert!(validate_cas_number("64-17-5").is_ok());   // Ethanol
    }

    #[test]
    fn test_validate_cas_number_invalid_format() {
        assert!(validate_cas_number("123-45").is_err());
        assert!(validate_cas_number("abc-de-f").is_err());
    }

    #[test]
    fn test_validate_cas_number_invalid_check_digit() {
        assert!(validate_cas_number("7732-18-6").is_err()); // Wrong check digit
    }

    #[test]
    fn test_validate_email_address() {
        assert!(validate_email_address("test@example.com").is_ok());
        assert!(validate_email_address("invalid-email").is_err());
        assert!(validate_email_address("@example.com").is_err());
    }

    #[test]
    fn test_validate_file_type() {
        let allowed_types = &["pdf", "xlsx", "csv"];
        assert!(validate_file_type("document.pdf", allowed_types).is_ok());
        assert!(validate_file_type("document.txt", allowed_types).is_err());
    }
}