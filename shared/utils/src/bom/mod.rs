//! BOM (Bill of Materials) Processing Module
//! 
//! Multi-format parser for extracting suppliers and components from BOM files.
//! Supports CSV, Excel (XLSX/XLS), and XML formats.
//! 
//! Requirements: 1.1, 1.2, 1.3, 1.4, 1.5

pub mod parser;
pub mod extractor;
pub mod validator;

pub use parser::{BomParser, BomFormat, ParsedBom};
pub use extractor::{SupplierExtractor, ExtractedSupplier};
pub use validator::{BomValidator, ValidationResult};
