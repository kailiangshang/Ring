use std::path::{Path, PathBuf};

use crate::error::{Result, RingError};
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;

const MEMORY_DIR: &str = "memory";
const MAX_FILE_CHARS: usize = 2000;
const MEMORY_FILES: &[&str] = &["user_profile", "preferences", "active_goals", "growth"];

fn ensure_memory_dir(self_dir: &Path) -> PathBuf {
    let dir = self_dir.join(MEMORY_DIR);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn read_memory_file_sync(self_dir: &Path, name: &str) -> Result<(String, bool)> {
    validate_memory_name(name)?;
    let path = ensure_memory_dir(self_dir).join(format!("{name}.md"));
    if !path.exists() {
        return Ok((String::new(), false));
    }
    Ok((std::fs::read_to_string(&path)?, true))
}

fn write_memory_file_sync(self_dir: &Path, name: &str, content: &str) -> Result<()> {
    validate_memory_name(name)?;
    let dir = ensure_memory_dir(self_dir);
    std::fs::write(dir.join(format!("{name}.md")), content)?;
    Ok(())
}

fn list_memory_files_sync(self_dir: &Path) -> Result<Vec<serde_json::Value>> {
    let _dir = ensure_memory_dir(self_dir);
    let mut files = Vec::new();
    for name in MEMORY_FILES {
        let (content, exists) = read_memory_file_sync(self_dir, name)?;
        let line_count = if exists {
            content.lines().filter(|l| !l.trim().is_empty()).count()
        } else {
            0
        };
        files.push(serde_json::json!({
            "name": name,
            "exists": exists,
            "line_count": line_count,
            "size": content.len(),
        }));
    }
    Ok(files)
}

fn delete_memory_file_sync(self_dir: &Path, name: &str) -> Result<()> {
    validate_memory_name(name)?;
    let path = ensure_memory_dir(self_dir).join(format!("{name}.md"));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub async fn read_memory_file(self_dir: &Path, name: &str) -> Result<(String, bool)> {
    let self_dir = self_dir.to_path_buf();
    let name = name.to_string();
    tokio::task::spawn_blocking(move || read_memory_file_sync(&self_dir, &name))
        .await
        .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?
}

pub async fn write_memory_file(self_dir: &Path, name: &str, content: &str) -> Result<()> {
    let self_dir = self_dir.to_path_buf();
    let name = name.to_string();
    let content = content.to_string();
    tokio::task::spawn_blocking(move || write_memory_file_sync(&self_dir, &name, &content))
        .await
        .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?
}

pub async fn list_memory_files(self_dir: &Path) -> Result<Vec<serde_json::Value>> {
    let self_dir = self_dir.to_path_buf();
    tokio::task::spawn_blocking(move || list_memory_files_sync(&self_dir))
        .await
        .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?
}

pub async fn delete_memory_file(self_dir: &Path, name: &str) -> Result<()> {
    let self_dir = self_dir.to_path_buf();
    let name = name.to_string();
    tokio::task::spawn_blocking(move || delete_memory_file_sync(&self_dir, &name))
        .await
        .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?
}

fn validate_memory_name(name: &str) -> Result<()> {
    if !MEMORY_FILES.contains(&name) {
        return Err(RingError::BadRequest(format!(
            "invalid memory file: {name}"
        )));
    }
    Ok(())
}

fn build_memory_context_sync(self_dir: &Path) -> String {
    let mut ctx = String::new();
    for name in MEMORY_FILES {
        if let Ok((content, exists)) = read_memory_file_sync(self_dir, name) {
            if exists && !content.trim().is_empty() {
                let label = match *name {
                    "user_profile" => "用户画像",
                    "preferences" => "偏好",
                    "active_goals" => "当前目标",
                    "growth" => "成长轨迹",
                    _ => name,
                };
                ctx.push_str(&format!("### {label}\n{content}\n\n"));
            }
        }
    }
    if ctx.is_empty() {
        return String::new();
    }
    format!("## 长期记忆\n\n{}", ctx)
}

pub async fn build_memory_context(self_dir: &Path) -> String {
    let self_dir = self_dir.to_path_buf();
    tokio::task::spawn_blocking(move || build_memory_context_sync(&self_dir))
        .await
        .unwrap_or_default()
}

#[derive(serde::Deserialize)]
struct ExtractedFact {
    fact: String,
    category: String,
}

pub async fn extract_memories(
    user: &UserRow,
    user_id: &str,
    user_message: &str,
    ai_response: &str,
) -> Result<()> {
    let llm = LlmClient::from_user(user)?;

    let prompt = format!(
        "你是记忆提取系统。从以下对话中提取新的或更新的用户事实。\n\n\
         输出 JSON 数组：\n\
         [{{\"fact\": \"...\", \"category\": \"user_profile|preferences|goals|growth\"}}]\n\n\
         规则：\n\
         - 只提取明确的用户事实，不提取 AI 的内容\n\
         - 跳过问候、闲聊、没有答案的问题\n\
         - category 必须是 user_profile（用户身份/背景/技能）、preferences（偏好/习惯）、goals（目标/任务）、growth（成就/学习/进步/里程碑）之一\n\
         - 如果没有新事实，输出空数组 []\n\
         - 每个事实用一句话概括\n\n\
         用户说：{}\n\n\
         AI 回复：{}",
        user_message, ai_response
    );

    let result = llm
        .chat_complete(
            "你是记忆提取系统，只输出 JSON 数组，不要其他内容。".into(),
            prompt,
        )
        .await?;

    let facts: Vec<ExtractedFact> = match serde_json::from_str::<Vec<ExtractedFact>>(result.trim())
    {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };

    let self_dir = crate::services::self_data::get_self_dir(user_id);
    for fact in facts {
        let file_name = match fact.category.as_str() {
            "user_profile" => "user_profile",
            "preferences" => "preferences",
            "goals" => "active_goals",
            "growth" => "growth",
            _ => continue,
        };

        if let Ok((mut content, _)) = read_memory_file(&self_dir, file_name).await {
            let fact_line = format!("- {}", fact.fact);
            if !content.contains(&fact.fact) {
                if !content.ends_with('\n') && !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(&fact_line);
                content.push('\n');
                if let Err(e) = write_memory_file(&self_dir, file_name, &content).await {
                    tracing::warn!("failed to write memory file: {e}");
                }
            }
        }
    }

    let self_dir = crate::services::self_data::get_self_dir(user_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "memory_extract") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(())
}

pub async fn check_and_compress(user: &UserRow, user_id: &str) -> Result<()> {
    let self_dir = crate::services::self_data::get_self_dir(user_id);
    for name in MEMORY_FILES {
        if let Ok((content, exists)) = read_memory_file(&self_dir, name).await {
            if exists && content.len() > MAX_FILE_CHARS {
                if let Ok(llm) = LlmClient::from_user(user) {
                    compress_memory_file(llm, &self_dir, name, &content).await;
                }
            }
        }
    }
    Ok(())
}

async fn compress_memory_file(llm: LlmClient, self_dir: &Path, name: &str, content: &str) {
    let label = match name {
        "user_profile" => "用户画像",
        "preferences" => "偏好",
        "active_goals" => "当前目标",
        "growth" => "成长轨迹",
        _ => name,
    };

    let prompt = format!(
        "以下是 Self AI 的「{}」记忆文件，内容过长需要压缩。\n\
         请重写为简洁的要点列表，保留最重要的信息，删除冗余和过时内容。\n\
         只输出 markdown 要点，不要其他内容。\n\n\
         原始内容：\n{}",
        label, content
    );

    if let Ok(compressed) = llm
        .chat_complete(
            "你是记忆压缩系统，只输出精简后的 markdown 要点。".into(),
            prompt,
        )
        .await
    {
        if let Err(e) = write_memory_file(self_dir, name, &compressed).await {
            tracing::warn!("failed to write compressed memory: {e}");
        }
    }
}