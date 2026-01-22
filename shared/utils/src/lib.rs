pub mod config;
pub mod logging;
pub mod error;
pub mod validation;
pub mod bom;

pub use config::*;
pub use logging::*;
pub use error::*;
pub use validation::*;
pub use bom::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_error_handling() {
        let error = ElementaError::validation("test_field", "test message");
        assert_eq!(error.error_code(), "VALIDATION_ERROR");
        assert_eq!(error.http_status_code(), 400);
    }
}