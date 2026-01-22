//! BOM File Parser
//! 
//! Multi-format parser supporting CSV, Excel, and XML bill of materials files.

use anyhow::{Context, Result};
use std::path::Path;
use uuid::Uuid;

/// Supported BOM file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomFormat {
    Csv,
    Excel,  // XLSX/XLS
    Xml,
}

impl BomFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "csv" => Some(Self::Csv),
            "xlsx" | "xls" => Some(Self::Excel),
            "xml" => Some(Self::Xml),
            _ => None,
        }
    }
    
    /// Detect format from content type header
    pub fn from_content_type(content_type: &str) -> Option<Self> {
        match content_type {
            "text/csv" | "application/csv" => Some(Self::Csv),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some(Self::Excel),
            "application/vnd.ms-excel" => Some(Self::Excel),
            "application/xml" | "text/xml" => Some(Self::Xml),
            _ => None,
        }
    }
}

/// Parsed BOM row representing a single component/supplier entry
#[derive(Debug, Clone)]
pub struct BomRow {
    pub row_number: usize,
    pub supplier_name: Option<String>,
    pub supplier_email: Option<String>,
    pub contact_person: Option<String>,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub material_type: Option<String>,
    pub cas_numbers: Vec<String>,
    pub raw_data: std::collections::HashMap<String, String>,
}

/// Complete parsed BOM with metadata
#[derive(Debug, Clone)]
pub struct ParsedBom {
    pub id: Uuid,
    pub filename: String,
    pub format: BomFormat,
    pub rows: Vec<BomRow>,
    pub column_headers: Vec<String>,
    pub total_rows: usize,
    pub parse_warnings: Vec<String>,
}

/// Main BOM parser
pub struct BomParser {
    /// Column name mappings for different BOM formats
    supplier_name_columns: Vec<String>,
    supplier_email_columns: Vec<String>,
    contact_columns: Vec<String>,
    part_number_columns: Vec<String>,
    description_columns: Vec<String>,
    material_columns: Vec<String>,
    cas_columns: Vec<String>,
}

impl Default for BomParser {
    fn default() -> Self {
        Self {
            supplier_name_columns: vec![
                "supplier".to_string(),
                "supplier_name".to_string(),
                "vendor".to_string(),
                "vendor_name".to_string(),
                "manufacturer".to_string(),
            ],
            supplier_email_columns: vec![
                "email".to_string(),
                "supplier_email".to_string(),
                "vendor_email".to_string(),
                "contact_email".to_string(),
            ],
            contact_columns: vec![
                "contact".to_string(),
                "contact_person".to_string(),
                "contact_name".to_string(),
            ],
            part_number_columns: vec![
                "part_number".to_string(),
                "part_no".to_string(),
                "pn".to_string(),
                "sku".to_string(),
                "item_number".to_string(),
            ],
            description_columns: vec![
                "description".to_string(),
                "desc".to_string(),
                "item_description".to_string(),
                "part_description".to_string(),
            ],
            material_columns: vec![
                "material".to_string(),
                "material_type".to_string(),
                "material_class".to_string(),
            ],
            cas_columns: vec![
                "cas".to_string(),
                "cas_number".to_string(),
                "cas_numbers".to_string(),
                "chemical_cas".to_string(),
            ],
        }
    }
}

impl BomParser {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Parse BOM file from bytes
    pub fn parse_bytes(&self, filename: &str, data: &[u8], format: Option<BomFormat>) -> Result<ParsedBom> {
        let format = format.or_else(|| BomFormat::from_extension(Path::new(filename)))
            .context("Could not determine file format")?;
        
        match format {
            BomFormat::Csv => self.parse_csv(filename, data),
            BomFormat::Excel => self.parse_excel(filename, data),
            BomFormat::Xml => self.parse_xml(filename, data),
        }
    }
    
    /// Parse CSV format
    fn parse_csv(&self, filename: &str, data: &[u8]) -> Result<ParsedBom> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(data);
        
        let headers: Vec<String> = reader.headers()
            .context("Failed to read CSV headers")?
            .iter()
            .map(|h| h.to_lowercase().trim().to_string())
            .collect();
        
        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        
        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    let raw_data: std::collections::HashMap<String, String> = headers.iter()
                        .enumerate()
                        .filter_map(|(i, h)| {
                            record.get(i).map(|v| (h.clone(), v.to_string()))
                        })
                        .collect();
                    
                    let row = self.map_row(idx + 2, &headers, &raw_data);
                    rows.push(row);
                }
                Err(e) => {
                    warnings.push(format!("Row {}: Parse error - {}", idx + 2, e));
                }
            }
        }
        
        Ok(ParsedBom {
            id: Uuid::new_v4(),
            filename: filename.to_string(),
            format: BomFormat::Csv,
            total_rows: rows.len(),
            rows,
            column_headers: headers,
            parse_warnings: warnings,
        })
    }
    
    /// Parse Excel format
    fn parse_excel(&self, filename: &str, data: &[u8]) -> Result<ParsedBom> {
        use calamine::{Reader, open_workbook_from_rs, Xlsx, DataType};
        
        let cursor = std::io::Cursor::new(data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .context("Failed to open Excel workbook")?;
        
        let sheet_name = workbook.sheet_names()
            .first()
            .cloned()
            .context("No sheets found in workbook")?;
        
        let range = workbook.worksheet_range(&sheet_name)
            .context("Failed to read worksheet")??;
        
        let mut rows_iter = range.rows();
        
        // First row is headers
        let headers: Vec<String> = rows_iter.next()
            .context("Empty worksheet")?
            .iter()
            .map(|cell: &DataType| cell.to_string().to_lowercase().trim().to_string())
            .collect();
        
        let mut rows = Vec::new();
        let warnings = Vec::new();
        
        for (idx, row) in rows_iter.enumerate() {
            let raw_data: std::collections::HashMap<String, String> = headers.iter()
                .enumerate()
                .filter_map(|(i, h): (usize, &String)| {
                    row.get(i).map(|v: &DataType| (h.clone(), v.to_string()))
                })
                .collect();
            
            let parsed_row = self.map_row(idx + 2, &headers, &raw_data);
            rows.push(parsed_row);
        }
        
        Ok(ParsedBom {
            id: Uuid::new_v4(),
            filename: filename.to_string(),
            format: BomFormat::Excel,
            total_rows: rows.len(),
            rows,
            column_headers: headers,
            parse_warnings: warnings,
        })
    }
    
    /// Parse XML format
    fn parse_xml(&self, filename: &str, data: &[u8]) -> Result<ParsedBom> {
        use quick_xml::Reader;
        use quick_xml::events::Event;
        
        let mut reader = Reader::from_reader(data);
        reader.trim_text(true);
        
        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut current_row: Option<std::collections::HashMap<String, String>> = None;
        let mut current_element = String::new();
        let mut row_number = 0;
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    // Common XML BOM element names
                    if matches!(tag_name.as_str(), "row" | "item" | "component" | "entry" | "record") {
                        current_row = Some(std::collections::HashMap::new());
                        row_number += 1;
                    } else if current_row.is_some() {
                        current_element = tag_name.to_lowercase();
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some(ref mut row) = current_row {
                        if !current_element.is_empty() {
                            row.insert(current_element.clone(), e.unescape().unwrap_or_default().to_string());
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    if matches!(tag_name.as_str(), "row" | "item" | "component" | "entry" | "record") {
                        if let Some(raw_data) = current_row.take() {
                            let headers: Vec<String> = raw_data.keys().cloned().collect();
                            let parsed_row = self.map_row(row_number, &headers, &raw_data);
                            rows.push(parsed_row);
                        }
                    }
                    current_element.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warnings.push(format!("XML parse error: {}", e));
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        
        let headers = if let Some(first) = rows.first() {
            first.raw_data.keys().cloned().collect()
        } else {
            Vec::new()
        };
        
        Ok(ParsedBom {
            id: Uuid::new_v4(),
            filename: filename.to_string(),
            format: BomFormat::Xml,
            total_rows: rows.len(),
            rows,
            column_headers: headers,
            parse_warnings: warnings,
        })
    }
    
    /// Map raw data to structured BomRow
    fn map_row(&self, row_number: usize, _headers: &[String], raw_data: &std::collections::HashMap<String, String>) -> BomRow {
        BomRow {
            row_number,
            supplier_name: self.find_value(&self.supplier_name_columns, raw_data),
            supplier_email: self.find_value(&self.supplier_email_columns, raw_data),
            contact_person: self.find_value(&self.contact_columns, raw_data),
            part_number: self.find_value(&self.part_number_columns, raw_data),
            description: self.find_value(&self.description_columns, raw_data),
            material_type: self.find_value(&self.material_columns, raw_data),
            cas_numbers: self.extract_cas_numbers(raw_data),
            raw_data: raw_data.clone(),
        }
    }
    
    /// Find value by checking multiple possible column names
    fn find_value(&self, candidates: &[String], data: &std::collections::HashMap<String, String>) -> Option<String> {
        for candidate in candidates {
            if let Some(value) = data.get(candidate) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }
    
    /// Extract and normalize CAS numbers from row
    fn extract_cas_numbers(&self, data: &std::collections::HashMap<String, String>) -> Vec<String> {
        let mut cas_numbers = Vec::new();
        
        for candidate in &self.cas_columns {
            if let Some(value) = data.get(candidate) {
                // Split by common delimiters and normalize
                for cas in value.split(&[',', ';', '|', '\n'][..]) {
                    let normalized = self.normalize_cas(cas.trim());
                    if !normalized.is_empty() && !cas_numbers.contains(&normalized) {
                        cas_numbers.push(normalized);
                    }
                }
            }
        }
        
        cas_numbers
    }
    
    /// Normalize CAS number format (XXXXXXX-XX-X)
    fn normalize_cas(&self, cas: &str) -> String {
        // Remove non-numeric and non-dash characters
        let cleaned: String = cas.chars()
            .filter(|c| c.is_numeric() || *c == '-')
            .collect();
        
        // Validate CAS format
        let parts: Vec<&str> = cleaned.split('-').collect();
        if parts.len() == 3 {
            cleaned
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_format_detection() {
        assert_eq!(BomFormat::from_extension(Path::new("test.csv")), Some(BomFormat::Csv));
        assert_eq!(BomFormat::from_extension(Path::new("test.xlsx")), Some(BomFormat::Excel));
        assert_eq!(BomFormat::from_extension(Path::new("test.xml")), Some(BomFormat::Xml));
        assert_eq!(BomFormat::from_extension(Path::new("test.txt")), None);
    }
    
    #[test]
    fn test_csv_parsing() {
        let csv_data = b"supplier,part_number,description,cas_number\nAcme Corp,PN-001,Widget,7732-18-5\nGlobex,PN-002,Gadget,7647-14-5";
        
        let parser = BomParser::new();
        let result = parser.parse_csv("test.csv", csv_data).unwrap();
        
        assert_eq!(result.total_rows, 2);
        assert_eq!(result.rows[0].supplier_name, Some("Acme Corp".to_string()));
        assert_eq!(result.rows[0].part_number, Some("PN-001".to_string()));
        assert_eq!(result.rows[0].cas_numbers, vec!["7732-18-5".to_string()]);
    }
    
    proptest! {
        /// Property 1: BOM Processing Completeness
        /// For any valid BOM, processed + flagged = total entries
        #[test]
        fn prop_bom_processing_completeness(
            supplier in "[A-Za-z ]{3,20}",
            part_no in "[A-Z]{2}-[0-9]{3}",
        ) {
            let csv = format!("supplier,part_number\n{},{}", supplier, part_no);
            let parser = BomParser::new();
            let result = parser.parse_csv("test.csv", csv.as_bytes()).unwrap();
            
            // Total parsed rows should equal input rows
            prop_assert_eq!(result.total_rows, 1);
            prop_assert!(result.rows[0].supplier_name.is_some());
        }
    }
}
