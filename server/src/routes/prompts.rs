use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct PromptEntry {
    pub module: String,
    pub name: String,
    pub content: String,
}

pub async fn list_prompts() -> Json<Vec<PromptEntry>> {
    let entries = vec![
        PromptEntry {
            module: "super_ring".into(),
            name: "default_system".into(),
            content: crate::prompts::super_ring::DEFAULT_SYSTEM.into(),
        },
        PromptEntry {
            module: "self_chat".into(),
            name: "system".into(),
            content: crate::prompts::self_chat::system(None, None, None),
        },
        PromptEntry {
            module: "group_ring".into(),
            name: "system".into(),
            content: crate::prompts::group_ring::system("RING", None),
        },
        PromptEntry {
            module: "archive".into(),
            name: "extract_system".into(),
            content: crate::prompts::archive::EXTRACT_SYSTEM.into(),
        },
        PromptEntry {
            module: "archive".into(),
            name: "judge_system".into(),
            content: crate::prompts::archive::JUDGE_SYSTEM.into(),
        },
        PromptEntry {
            module: "blueprint".into(),
            name: "system".into(),
            content: crate::prompts::blueprint::system("RING", None, None),
        },
        PromptEntry {
            module: "search".into(),
            name: "cross_ring_instruction".into(),
            content: crate::prompts::search::cross_ring_context_instruction(),
        },
        PromptEntry {
            module: "workflow".into(),
            name: "file_parse".into(),
            content: crate::prompts::workflow::file_parse_extraction(None),
        },
        PromptEntry {
            module: "workflow".into(),
            name: "knowledge_extract".into(),
            content: crate::prompts::workflow::knowledge_extraction_prompt(None),
        },
        PromptEntry {
            module: "compact".into(),
            name: "system".into(),
            content: crate::prompts::compact::SYSTEM.into(),
        },
        PromptEntry {
            module: "super_ring".into(),
            name: "cross_ring_query".into(),
            content: crate::prompts::super_ring::cross_ring_query("RING_A", "RING_B"),
        },
        PromptEntry {
            module: "super_ring".into(),
            name: "cross_ring_compare".into(),
            content: crate::prompts::super_ring::cross_ring_analysis("compare", "RING_A"),
        },
        PromptEntry {
            module: "super_ring".into(),
            name: "cross_ring_merge".into(),
            content: crate::prompts::super_ring::cross_ring_analysis("merge", "RING_A"),
        },
        PromptEntry {
            module: "super_ring".into(),
            name: "cross_ring_summary".into(),
            content: crate::prompts::super_ring::cross_ring_analysis("summary", "RING_A"),
        },
        PromptEntry {
            module: "self_chat".into(),
            name: "metrics_context".into(),
            content: crate::prompts::self_chat::metrics_context(&serde_json::json!({})),
        },
        PromptEntry {
            module: "group_docs".into(),
            name: "active_context_system".into(),
            content: crate::prompts::group_docs::ACTIVE_CONTEXT_SYSTEM.into(),
        },
        PromptEntry {
            module: "group_docs".into(),
            name: "active_context_user".into(),
            content: crate::prompts::group_docs::ACTIVE_CONTEXT_USER.into(),
        },
        PromptEntry {
            module: "group_docs".into(),
            name: "archive_patterns_system".into(),
            content: crate::prompts::group_docs::ARCHIVE_PATTERNS_SYSTEM.into(),
        },
        PromptEntry {
            module: "group_docs".into(),
            name: "archive_patterns_user".into(),
            content: crate::prompts::group_docs::ARCHIVE_PATTERNS_USER.into(),
        },
        PromptEntry {
            module: "export".into(),
            name: "ai_report_system".into(),
            content: crate::prompts::export::AI_REPORT_SYSTEM.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "decision_material".into(),
            content: crate::prompts::session::skill::DECISION_MATERIAL.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "decision_summary".into(),
            content: crate::prompts::session::skill::DECISION_SUMMARY.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "research_material".into(),
            content: crate::prompts::session::skill::RESEARCH_MATERIAL.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "research_summary".into(),
            content: crate::prompts::session::skill::RESEARCH_SUMMARY.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "review_material".into(),
            content: crate::prompts::session::skill::REVIEW_MATERIAL.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "review_summary".into(),
            content: crate::prompts::session::skill::REVIEW_SUMMARY.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "retrospective_material".into(),
            content: crate::prompts::session::skill::RETROSPECTIVE_MATERIAL.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "retrospective_summary".into(),
            content: crate::prompts::session::skill::RETROSPECTIVE_SUMMARY.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "knowledge_sharing_material".into(),
            content: crate::prompts::session::skill::KNOWLEDGE_SHARING_MATERIAL.into(),
        },
        PromptEntry {
            module: "session_skill".into(),
            name: "knowledge_sharing_summary".into(),
            content: crate::prompts::session::skill::KNOWLEDGE_SHARING_SUMMARY.into(),
        },
    ];

    Json(entries)
}
