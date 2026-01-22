//! Email Template Engine
//! 
//! Handlebars-based template rendering for compliance emails.

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Email template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub subject_template: String,
    pub body_html_template: String,
    pub body_text_template: String,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Template rendering result
#[derive(Debug, Clone)]
pub struct RenderedEmail {
    pub subject: String,
    pub body_html: String,
    #[allow(dead_code)]
    pub body_text: String,
}

/// Template engine
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    templates: HashMap<String, EmailTemplate>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            handlebars: Handlebars::new(),
            templates: HashMap::new(),
        };
        
        // Register built-in templates
        engine.register_builtin_templates();
        
        engine
    }
    
    /// Register built-in compliance email templates
    fn register_builtin_templates(&mut self) {
        // Initial outreach template
        let initial_outreach = EmailTemplate {
            id: "initial_outreach".to_string(),
            name: "Initial Compliance Request".to_string(),
            description: "First contact with supplier requesting PFAS compliance data".to_string(),
            subject_template: "PFAS Compliance Data Request - {{company_name}}".to_string(),
            body_html_template: r#"
<!DOCTYPE html>
<html>
<head><style>body{font-family:Arial,sans-serif;line-height:1.6;color:#333;}.header{background:#2563eb;color:white;padding:20px;}.content{padding:20px;}.footer{background:#f3f4f6;padding:20px;font-size:12px;}</style></head>
<body>
<div class="header"><h2>PFAS Compliance Data Request</h2></div>
<div class="content">
<p>Dear {{contact_name}},</p>
<p>As part of our ongoing compliance efforts with EPA TSCA PFAS reporting requirements (effective 2026), we are reaching out to request chemical composition data for the following components you supply to {{company_name}}:</p>
<ul>
{{#each components}}<li>{{this}}</li>{{/each}}
</ul>
<p>Specifically, we need:</p>
<ol>
<li>Complete CAS number listings for all substances used in manufacturing</li>
<li>Any existing PFAS testing or certification documentation</li>
<li>Material Safety Data Sheets (MSDS/SDS) for relevant materials</li>
</ol>
<p>Please respond by {{deadline}} to ensure we meet our regulatory reporting deadlines.</p>
<p>If you have any questions about this request, please don't hesitate to reach out.</p>
<p>Best regards,<br>{{sender_name}}<br>{{sender_title}}</p>
</div>
<div class="footer">This is an automated message from the Elementa Compliance System. Reference: {{reference_id}}</div>
</body>
</html>
"#.to_string(),
            body_text_template: r#"
PFAS Compliance Data Request

Dear {{contact_name}},

As part of our ongoing compliance efforts with EPA TSCA PFAS reporting requirements (effective 2026), we are reaching out to request chemical composition data for the following components you supply to {{company_name}}:

{{#each components}}- {{this}}
{{/each}}

Specifically, we need:
1. Complete CAS number listings for all substances used in manufacturing
2. Any existing PFAS testing or certification documentation
3. Material Safety Data Sheets (MSDS/SDS) for relevant materials

Please respond by {{deadline}} to ensure we meet our regulatory reporting deadlines.

If you have any questions about this request, please don't hesitate to reach out.

Best regards,
{{sender_name}}
{{sender_title}}

---
This is an automated message from the Elementa Compliance System.
Reference: {{reference_id}}
"#.to_string(),
            variables: vec![
                TemplateVariable { name: "contact_name".to_string(), description: "Supplier contact name".to_string(), required: true, default_value: None },
                TemplateVariable { name: "company_name".to_string(), description: "Your company name".to_string(), required: true, default_value: None },
                TemplateVariable { name: "components".to_string(), description: "List of components".to_string(), required: true, default_value: None },
                TemplateVariable { name: "deadline".to_string(), description: "Response deadline".to_string(), required: true, default_value: None },
                TemplateVariable { name: "sender_name".to_string(), description: "Sender name".to_string(), required: true, default_value: None },
                TemplateVariable { name: "sender_title".to_string(), description: "Sender title".to_string(), required: true, default_value: None },
                TemplateVariable { name: "reference_id".to_string(), description: "Reference ID".to_string(), required: false, default_value: Some("AUTO".to_string()) },
            ],
        };
        
        self.templates.insert(initial_outreach.id.clone(), initial_outreach);
        
        // Follow-up template
        let follow_up = EmailTemplate {
            id: "follow_up".to_string(),
            name: "Follow-up Request".to_string(),
            description: "Follow-up email for outstanding compliance data".to_string(),
            subject_template: "Reminder: PFAS Compliance Data Request - {{company_name}}".to_string(),
            body_html_template: r#"
<!DOCTYPE html>
<html>
<body style="font-family:Arial,sans-serif;line-height:1.6;color:#333;">
<p>Dear {{contact_name}},</p>
<p>This is a friendly reminder regarding our previous request for PFAS compliance data (Reference: {{reference_id}}).</p>
<p>We have not yet received the requested documentation for the following components:</p>
<ul>{{#each pending_components}}<li>{{this}}</li>{{/each}}</ul>
<p>The deadline for submission is <strong>{{deadline}}</strong>. Please prioritize this request to avoid any disruption to our business relationship.</p>
<p>If you need assistance or have questions, please contact us.</p>
<p>Best regards,<br>{{sender_name}}</p>
</body>
</html>
"#.to_string(),
            body_text_template: "Dear {{contact_name}},\n\nThis is a friendly reminder regarding our PFAS compliance data request (Reference: {{reference_id}}).\n\nDeadline: {{deadline}}\n\nBest regards,\n{{sender_name}}".to_string(),
            variables: vec![
                TemplateVariable { name: "contact_name".to_string(), description: "Supplier contact name".to_string(), required: true, default_value: None },
                TemplateVariable { name: "pending_components".to_string(), description: "Components still pending".to_string(), required: true, default_value: None },
                TemplateVariable { name: "deadline".to_string(), description: "Response deadline".to_string(), required: true, default_value: None },
            ],
        };
        
        self.templates.insert(follow_up.id.clone(), follow_up);
    }
    
    /// Get template by ID
    #[allow(dead_code)]
    pub fn get_template(&self, template_id: &str) -> Option<&EmailTemplate> {
        self.templates.get(template_id)
    }
    
    /// List all templates
    pub fn list_templates(&self) -> Vec<&EmailTemplate> {
        self.templates.values().collect()
    }
    
    /// Render template with variables
    pub fn render(&self, template_id: &str, variables: &HashMap<String, serde_json::Value>) -> Result<RenderedEmail> {
        let template = self.templates.get(template_id)
            .context("Template not found")?;
        
        let subject = self.handlebars.render_template(&template.subject_template, variables)
            .context("Failed to render subject")?;
        
        let body_html = self.handlebars.render_template(&template.body_html_template, variables)
            .context("Failed to render HTML body")?;
        
        let body_text = self.handlebars.render_template(&template.body_text_template, variables)
            .context("Failed to render text body")?;
        
        Ok(RenderedEmail {
            subject,
            body_html,
            body_text,
        })
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
