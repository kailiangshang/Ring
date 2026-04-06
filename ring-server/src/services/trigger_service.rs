use crate::services::llm_provider::LlmEvent;

pub fn check_archive_suggestion(last_assistant_text: &str) -> Option<LlmEvent> {
    let indicators = ["总结", "归档", "记录", "要点", "会议纪要"];
    let lower = last_assistant_text.to_lowercase();
    if indicators.iter().any(|i| lower.contains(i)) {
        Some(LlmEvent::ArchiveSuggestion {
            data: serde_json::json!({
                "reason": "对话内容包含可归档的知识",
                "suggested_title": "AI suggested archive"
            }),
        })
    } else {
        None
    }
}

pub fn check_empty_graph_guidance(node_count: usize) -> Option<LlmEvent> {
    if node_count < 3 {
        Some(LlmEvent::ArchiveSuggestion {
            data: serde_json::json!({
                "reason": "知识图谱节点较少，建议添加更多知识节点",
                "suggested_title": "empty_graph_guidance"
            }),
        })
    } else {
        None
    }
}
