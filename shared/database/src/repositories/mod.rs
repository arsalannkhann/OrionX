//! Repository module for database CRUD operations
//! 
//! Provides typed repository implementations for all domain entities.

pub mod supplier;
pub mod compliance;
pub mod component;
pub mod chemical;
pub mod workflow;
pub mod audit;
pub mod email;

pub use supplier::SupplierRepository;
pub use compliance::ComplianceRepository;
pub use component::ComponentRepository;
pub use chemical::ChemicalRepository;
pub use workflow::WorkflowRepository;
pub use audit::AuditRepository;
pub use email::EmailRepository;
