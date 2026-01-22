use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ElementaError {
    #[error("Database error: {message}")]
    Database { message: String },
    
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },
    
    #[error("Document processing error: {message}")]
    DocumentProcessing { message: String },
    
    #[error("Email communication error: {message}")]
    EmailCommunication { message: String },
    
    #[error("Chemical database error: {message}")]
    ChemicalDatabase { message: String },
    
    #[error("Workflow orchestration error: {message}")]
    WorkflowOrchestration { message: String },
    
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    
    #[error("Authorization error: {message}")]
    Authorization { message: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Conflict: {message}")]
    Conflict { message: String },
    
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },
    
    #[error("Internal server error: {message}")]
    Internal { message: String },
}

impl ElementaError {
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }
    
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn document_processing(message: impl Into<String>) -> Self {
        Self::DocumentProcessing {
            message: message.into(),
        }
    }
    
    pub fn email_communication(message: impl Into<String>) -> Self {
        Self::EmailCommunication {
            message: message.into(),
        }
    }
    
    pub fn chemical_database(message: impl Into<String>) -> Self {
        Self::ChemicalDatabase {
            message: message.into(),
        }
    }
    
    pub fn workflow_orchestration(message: impl Into<String>) -> Self {
        Self::WorkflowOrchestration {
            message: message.into(),
        }
    }
    
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }
    
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }
    
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Database { .. } => "DATABASE_ERROR",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::DocumentProcessing { .. } => "DOCUMENT_PROCESSING_ERROR",
            Self::EmailCommunication { .. } => "EMAIL_COMMUNICATION_ERROR",
            Self::ChemicalDatabase { .. } => "CHEMICAL_DATABASE_ERROR",
            Self::WorkflowOrchestration { .. } => "WORKFLOW_ORCHESTRATION_ERROR",
            Self::Authentication { .. } => "AUTHENTICATION_ERROR",
            Self::Authorization { .. } => "AUTHORIZATION_ERROR",
            Self::Configuration { .. } => "CONFIGURATION_ERROR",
            Self::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::RateLimit { .. } => "RATE_LIMIT_EXCEEDED",
            Self::Internal { .. } => "INTERNAL_SERVER_ERROR",
        }
    }
    
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::Database { .. } => 500,
            Self::Validation { .. } => 400,
            Self::DocumentProcessing { .. } => 422,
            Self::EmailCommunication { .. } => 502,
            Self::ChemicalDatabase { .. } => 502,
            Self::WorkflowOrchestration { .. } => 500,
            Self::Authentication { .. } => 401,
            Self::Authorization { .. } => 403,
            Self::Configuration { .. } => 500,
            Self::ExternalService { .. } => 502,
            Self::NotFound { .. } => 404,
            Self::Conflict { .. } => 409,
            Self::RateLimit { .. } => 429,
            Self::Internal { .. } => 500,
        }
    }
}

pub type ElementaResult<T> = Result<T, ElementaError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl From<ElementaError> for ErrorResponse {
    fn from(error: ElementaError) -> Self {
        Self {
            error: error.to_string(),
            code: error.error_code().to_string(),
            message: error.to_string(),
            details: None,
        }
    }
}

// Conversion from common error types
impl From<sqlx::Error> for ElementaError {
    fn from(error: sqlx::Error) -> Self {
        Self::database(error.to_string())
    }
}

impl From<mongodb::error::Error> for ElementaError {
    fn from(error: mongodb::error::Error) -> Self {
        Self::database(error.to_string())
    }
}

impl From<redis::RedisError> for ElementaError {
    fn from(error: redis::RedisError) -> Self {
        Self::database(error.to_string())
    }
}

impl From<reqwest::Error> for ElementaError {
    fn from(error: reqwest::Error) -> Self {
        Self::external_service("HTTP Client", error.to_string())
    }
}

impl From<serde_json::Error> for ElementaError {
    fn from(error: serde_json::Error) -> Self {
        Self::validation("JSON", error.to_string())
    }
}