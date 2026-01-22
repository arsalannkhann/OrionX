//! Component domain models for the Elementa compliance system.
//! 
//! This module defines component-related data structures including
//! component specifications, material types, and CAS number management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

/// Represents a component or part in the supply chain with associated CAS numbers
/// and detailed specifications.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate, PartialEq)]
pub struct Component {
    pub id: Uuid,
    #[validate(length(min = 1, max = 100, message = "Part number must be between 1 and 100 characters"))]
    pub part_number: String,
    #[validate(length(min = 1, max = 500, message = "Description must be between 1 and 500 characters"))]
    pub description: String,
    #[validate(custom = "validate_cas_numbers")]
    pub cas_numbers: Vec<String>,
    pub material_type: MaterialType,
    pub supplier_id: Uuid,
    #[validate]
    pub specifications: ComponentSpecifications,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaterialType {
    Metal,
    Polymer,
    Ceramic,
    Composite,
    Chemical,
    Electronic,
    Textile,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ComponentSpecifications {
    #[validate(range(min = 0.0, message = "Weight must be positive"))]
    pub weight_grams: Option<f64>,
    #[validate]
    pub dimensions: Option<Dimensions>,
    #[validate(length(max = 50))]
    pub color: Option<String>,
    #[validate(length(max = 100))]
    pub finish: Option<String>,
    #[validate(length(max = 50))]
    pub grade: Option<String>,
    pub certifications: Vec<String>,
    pub custom_properties: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct Dimensions {
    #[validate(range(min = 0.0, message = "Length must be positive"))]
    pub length_mm: f64,
    #[validate(range(min = 0.0, message = "Width must be positive"))]
    pub width_mm: f64,
    #[validate(range(min = 0.0, message = "Height must be positive"))]
    pub height_mm: f64,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            part_number: String::new(),
            description: String::new(),
            cas_numbers: Vec::new(),
            material_type: MaterialType::Other("Unknown".to_string()),
            supplier_id: Uuid::new_v4(),
            specifications: ComponentSpecifications::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for ComponentSpecifications {
    fn default() -> Self {
        Self {
            weight_grams: None,
            dimensions: None,
            color: None,
            finish: None,
            grade: None,
            certifications: Vec::new(),
            custom_properties: std::collections::HashMap::new(),
        }
    }
}

// Custom validation functions
fn validate_cas_numbers(cas_numbers: &[String]) -> Result<(), ValidationError> {
    for cas_number in cas_numbers {
        if !is_valid_cas_format(cas_number) {
            return Err(ValidationError::new("invalid_cas_format"));
        }
    }
    Ok(())
}

fn is_valid_cas_format(cas_number: &str) -> bool {
    // CAS number format: XXXXXX-XX-X where X is a digit
    let parts: Vec<&str> = cas_number.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    
    // Check format: 2-7 digits, 2 digits, 1 digit
    if parts[0].len() < 2 || parts[0].len() > 7 || parts[1].len() != 2 || parts[2].len() != 1 {
        return false;
    }
    
    // Check all parts are numeric
    parts.iter().all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

// Utility methods for Component
impl Component {
    /// Creates a new component with the given part number and description
    pub fn new(part_number: String, description: String, supplier_id: Uuid) -> Self {
        let mut component = Self::default();
        component.part_number = part_number;
        component.description = description;
        component.supplier_id = supplier_id;
        component
    }
    
    /// Adds a CAS number to the component if it's valid
    pub fn add_cas_number(&mut self, cas_number: String) -> Result<(), String> {
        if !is_valid_cas_format(&cas_number) {
            return Err(format!("Invalid CAS number format: {}", cas_number));
        }
        
        if !self.cas_numbers.contains(&cas_number) {
            self.cas_numbers.push(cas_number);
            self.updated_at = Utc::now();
        }
        
        Ok(())
    }
    
    /// Removes a CAS number from the component
    pub fn remove_cas_number(&mut self, cas_number: &str) {
        if let Some(pos) = self.cas_numbers.iter().position(|x| x == cas_number) {
            self.cas_numbers.remove(pos);
            self.updated_at = Utc::now();
        }
    }
    
    /// Checks if the component has any CAS numbers
    pub fn has_cas_numbers(&self) -> bool {
        !self.cas_numbers.is_empty()
    }
    
    /// Gets the total volume in cubic millimeters if dimensions are available
    pub fn volume_mm3(&self) -> Option<f64> {
        self.specifications.dimensions.as_ref()
            .map(|d| d.length_mm * d.width_mm * d.height_mm)
    }
    
    /// Adds a certification to the component
    pub fn add_certification(&mut self, certification: String) {
        if !self.specifications.certifications.contains(&certification) {
            self.specifications.certifications.push(certification);
            self.updated_at = Utc::now();
        }
    }
    
    /// Sets a custom property
    pub fn set_custom_property(&mut self, key: String, value: String) {
        self.specifications.custom_properties.insert(key, value);
        self.updated_at = Utc::now();
    }
    
    /// Gets a custom property value
    pub fn get_custom_property(&self, key: &str) -> Option<&String> {
        self.specifications.custom_properties.get(key)
    }
}