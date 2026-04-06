use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::tool_model::ToolDefinition;
use crate::services::tool_engine::Tool;

pub struct PrivacyFilterTool {
    email_re: Regex,
    phone_re: Regex,
    id_card_re: Regex,
}

#[derive(Deserialize)]
struct PrivacyFilterInput {
    text: String,
}

impl Default for PrivacyFilterTool {
    fn default() -> Self {
        PrivacyFilterTool {
            email_re: Regex::new(r"(?i)\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
                .unwrap(),
            phone_re: Regex::new(r"\b1[3-9]\d{9}\b").unwrap(),
            id_card_re: Regex::new(r"\b\d{17}[\dXx]\b").unwrap(),
        }
    }
}

impl PrivacyFilterTool {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Tool for PrivacyFilterTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "privacy_filter".to_string(),
            description: "Redact PII (email, phone, ID card) from text".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to filter" }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        let parsed: PrivacyFilterInput =
            serde_json::from_value(input).map_err(RingError::Serialization)?;

        let mut count = 0;
        let text = self
            .email_re
            .replace_all(&parsed.text, "[REDACTED]")
            .into_owned();
        let (text, c) = self.count_replace(&text, &self.phone_re);
        count += c;
        let (text, c) = self.count_replace(&text, &self.id_card_re);
        count += c;

        let email_count = self.email_re.find_iter(&parsed.text).count();
        count += email_count;

        Ok(serde_json::json!({
            "filtered_text": text,
            "redactions_count": count
        }))
    }
}

impl PrivacyFilterTool {
    fn count_replace<'a>(&self, text: &'a str, re: &Regex) -> (std::borrow::Cow<'a, str>, usize) {
        let count = re.find_iter(text).count();
        let replaced = re.replace_all(text, "[REDACTED]");
        (replaced, count)
    }
}
