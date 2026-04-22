use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyFilters {
    #[serde(default = "default_true")]
    pub phone: bool,
    #[serde(default = "default_true")]
    pub id_card: bool,
    #[serde(default = "default_true")]
    pub email: bool,
    #[serde(default = "default_true")]
    pub bank_card: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PrivacyFilters {
    fn default() -> Self {
        Self {
            phone: true,
            id_card: true,
            email: true,
            bank_card: true,
        }
    }
}

impl PrivacyFilters {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

pub fn apply_filters(text: &str, filters: &PrivacyFilters) -> String {
    let mut result = text.to_string();

    if filters.phone {
        let re = Regex::new(r"(?<![0-9a-zA-Z])(?:\+?86[-\s]?)?1[3-9]\d{9}(?![0-9a-zA-Z])").unwrap();
        result = re.replace_all(&result, "[PHONE]").to_string();
    }

    if filters.id_card {
        let re = Regex::new(r"(?<![0-9a-zA-Z])\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx](?![0-9a-zA-Z])").unwrap();
        result = re.replace_all(&result, "[ID_CARD]").to_string();
    }

    if filters.email {
        let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        result = re.replace_all(&result, "[EMAIL]").to_string();
    }

    if filters.bank_card {
        let re = Regex::new(r"(?<![0-9a-zA-Z])\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}(?:[-\s]?\d{3})?(?![0-9a-zA-Z])").unwrap();
        result = re.replace_all(&result, "[BANK_CARD]").to_string();
    }

    result
}
