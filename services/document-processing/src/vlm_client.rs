//! VLM (Vision-Language Model) Client
//! 
//! Integrates with OpenAI GPT-4V or similar for document understanding.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// VLM client for document processing
#[allow(dead_code)]
pub struct VlmClient {
    client: Client,
    api_key: String,
    model: String,
}

#[allow(dead_code)]
impl VlmClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            api_key,
            model: "gpt-4o".to_string(),
        }
    }
    
    /// Extract compliance data from document image
    pub async fn extract_compliance_data(&self, image_data: &[u8], prompt: &str) -> Result<VlmExtractionResult> {
        let base64_image = BASE64.encode(image_data);
        
        let request = VlmRequest {
            model: self.model.clone(),
            messages: vec![
                VlmMessage {
                    role: "system".to_string(),
                    content: vec![VlmContent::Text {
                        text: COMPLIANCE_EXTRACTION_PROMPT.to_string(),
                    }],
                },
                VlmMessage {
                    role: "user".to_string(),
                    content: vec![
                        VlmContent::Image {
                            image_url: ImageUrl {
                                url: format!("data:image/png;base64,{}", base64_image),
                            },
                        },
                        VlmContent::Text {
                            text: prompt.to_string(),
                        },
                    ],
                },
            ],
            max_tokens: 4096,
            temperature: 0.1, // Low temperature for consistent extraction
        };
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call VLM API")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("VLM API error: {}", error_text);
        }
        
        let result: VlmResponse = response.json().await
            .context("Failed to parse VLM response")?;
        
        let content = result.choices.first()
            .map(|c| c.message.content.as_str())
            .context("No response content")?;
        
        // Parse structured extraction from response
        let extraction: VlmExtractionResult = serde_json::from_str(content)
            .context("Failed to parse extraction JSON")?;
        
        Ok(extraction)
    }
}

/// VLM API request
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct VlmRequest {
    model: String,
    messages: Vec<VlmMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct VlmMessage {
    role: String,
    content: Vec<VlmContent>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
#[serde(tag = "type")]
enum VlmContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    Image { image_url: ImageUrl },
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ImageUrl {
    url: String,
}

/// VLM API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VlmResponse {
    choices: Vec<VlmChoice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VlmChoice {
    message: VlmChoiceMessage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VlmChoiceMessage {
    content: String,
}

/// Structured extraction result
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VlmExtractionResult {
    pub cas_numbers: Vec<CasExtraction>,
    pub test_results: Vec<TestResultExtraction>,
    pub certifications: Vec<CertificationExtraction>,
    pub uncertainties: Vec<Uncertainty>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CasExtraction {
    pub cas_number: String,
    pub confidence: f64,
    pub context: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TestResultExtraction {
    pub test_name: String,
    pub result: String,
    pub unit: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CertificationExtraction {
    pub name: String,
    pub issuer: Option<String>,
    pub valid_until: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Uncertainty {
    pub field: String,
    pub reason: String,
    pub alternatives: Vec<String>,
}

#[allow(dead_code)]
const COMPLIANCE_EXTRACTION_PROMPT: &str = r#"
You are a compliance document extraction specialist. Extract structured data from the provided document image.

Return a JSON object with the following structure:
{
  "cas_numbers": [
    {"cas_number": "XXXXX-XX-X", "confidence": 0.0-1.0, "context": "surrounding text", "location": "page/section"}
  ],
  "test_results": [
    {"test_name": "...", "result": "...", "unit": "...", "confidence": 0.0-1.0}
  ],
  "certifications": [
    {"name": "...", "issuer": "...", "valid_until": "YYYY-MM-DD"}
  ],
  "overall_confidence": 0.0-1.0,
  "uncertainties": [
    {"field": "...", "reason": "...", "alternatives": ["...", "..."]}
  ]
}

Focus on:
1. CAS numbers (format: XXXXXXX-XX-X)
2. Chemical test results and measurements
3. Compliance certifications (RoHS, REACH, etc.)
4. Mark any uncertain extractions with low confidence

Return ONLY valid JSON, no additional text.
"#;
