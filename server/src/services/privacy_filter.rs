use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static PHONE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)(?:\+?86[-\s]?)?1[3-9]\d{9}(?:\s|$)").unwrap());
static ID_CARD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:^|\s)\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx](?:\s|$)",
    )
    .unwrap()
});
static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());
static BANK_CARD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}(?:[-\s]?\d{3})?(?:\s|$)").unwrap()
});

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
        result = PHONE_RE.replace_all(&result, " [PHONE] ").to_string();
    }

    if filters.id_card {
        result = ID_CARD_RE.replace_all(&result, " [ID_CARD] ").to_string();
    }

    if filters.email {
        result = EMAIL_RE.replace_all(&result, "[EMAIL]").to_string();
    }

    if filters.bank_card {
        result = BANK_CARD_RE
            .replace_all(&result, " [BANK_CARD] ")
            .to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_filter() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("我的手机号是 13812345678 请联系我", &filters);
        assert!(result.contains("[PHONE]"));
        assert!(!result.contains("13812345678"));
    }

    #[test]
    fn test_phone_with_country_code() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("call +8613900001234 ok", &filters);
        assert!(result.contains("[PHONE]"));
    }

    #[test]
    fn test_id_card_18_digits() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("身份证号 110101199001011234 在这里", &filters);
        assert!(result.contains("[ID_CARD]"));
        assert!(!result.contains("110101199001011234"));
    }

    #[test]
    fn test_id_card_with_x() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("证件 11010119900101123X 请查收", &filters);
        assert!(result.contains("[ID_CARD]"));
    }

    #[test]
    fn test_email_filter() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("联系邮箱 test@example.com 谢谢", &filters);
        assert!(result.contains("[EMAIL]"));
        assert!(!result.contains("test@example.com"));
    }

    #[test]
    fn test_email_various_domains() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("user.name+tag@sub.domain.co.uk", &filters);
        assert!(result.contains("[EMAIL]"));
    }

    #[test]
    fn test_bank_card_16_digits() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("卡号 6222021234567890 到期", &filters);
        assert!(result.contains("[BANK_CARD]"));
        assert!(!result.contains("6222021234567890"));
    }

    #[test]
    fn test_bank_card_19_digits() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("银行卡 6222021234567890123 请查收", &filters);
        assert!(result.contains("[BANK_CARD]"));
    }

    #[test]
    fn test_empty_string() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("", &filters);
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_match() {
        let filters = PrivacyFilters::default();
        let result = apply_filters("这是一段普通文本，没有敏感信息。", &filters);
        assert_eq!(result, "这是一段普通文本，没有敏感信息。");
    }

    #[test]
    fn test_mixed_content() {
        let filters = PrivacyFilters::default();
        let input =
            "用户邮箱 a@b.com 手机 13900001111 身份证 110101199001011234 卡号 6222021234567890";
        let result = apply_filters(input, &filters);
        assert!(result.contains("[EMAIL]"));
        assert!(result.contains("[PHONE]"));
        assert!(result.contains("[ID_CARD]"));
        assert!(result.contains("[BANK_CARD]"));
    }

    #[test]
    fn test_filter_disabled() {
        let filters = PrivacyFilters {
            phone: false,
            id_card: false,
            email: false,
            bank_card: false,
        };
        let input = "手机 13900001111 邮箱 a@b.com";
        let result = apply_filters(input, &filters);
        assert_eq!(result, input);
    }

    #[test]
    fn test_partial_filter_disabled() {
        let filters = PrivacyFilters {
            phone: true,
            id_card: false,
            email: false,
            bank_card: false,
        };
        let input = "手机 13900001111 邮箱 a@b.com";
        let result = apply_filters(input, &filters);
        assert!(result.contains("[PHONE]"));
        assert!(result.contains("a@b.com"));
    }
}
