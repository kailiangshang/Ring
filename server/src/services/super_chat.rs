use std::path::Path;

use async_openai::types::ChatCompletionTool;
use serde::Deserialize;

use crate::error::{Result, RingError};
use crate::models::message;
use crate::services::chat;
use crate::services::llm::{LlmClient, SseEvent};
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
use crate::state::AppState;

const SUPER_RING_ID: &str = "super";

const DEFAULT_SUPER_SYSTEM_PROMPT: &str = crate::prompts::super_ring::DEFAULT_SYSTEM;

const DEFAULT_PREFERENCES: &str = "## 语言\n- default: zh-CN\n\n## LLM\n- default_provider: openai\n\n## 输出格式\n- style: concise\n\n## 默认模式\n- mode: normal";

fn get_system_prompt_sync(hub_dir: &Path) -> String {
    let prompt_file = hub_dir.join("system_prompt.md");
    match std::fs::read_to_string(&prompt_file) {
        Ok(content) if !content.trim().is_empty() => content,
        _ => DEFAULT_SUPER_SYSTEM_PROMPT.to_string(),
    }
}

fn get_system_prompt_info_sync(hub_dir: &Path) -> (String, bool) {
    let prompt_file = hub_dir.join("system_prompt.md");
    match std::fs::read_to_string(&prompt_file) {
        Ok(ref content) if !content.trim().is_empty() => (content.clone(), true),
        _ => (DEFAULT_SUPER_SYSTEM_PROMPT.to_string(), false),
    }
}

fn update_system_prompt_sync(hub_dir: &Path, prompt: &str) -> Result<()> {
    let prompt_file = hub_dir.join("system_prompt.md");
    if prompt.trim().is_empty() {
        let _ = std::fs::remove_file(&prompt_file);
    } else {
        std::fs::write(&prompt_file, prompt)?;
    }
    Ok(())
}

fn get_user_preferences_sync(hub_dir: &Path) -> String {
    let prefs_file = hub_dir.join("user_preferences.md");
    match std::fs::read_to_string(&prefs_file) {
        Ok(content) if !content.trim().is_empty() => content,
        _ => DEFAULT_PREFERENCES.to_string(),
    }
}

fn get_user_preferences_info_sync(hub_dir: &Path) -> (String, bool) {
    let prefs_file = hub_dir.join("user_preferences.md");
    match std::fs::read_to_string(&prefs_file) {
        Ok(ref content) if !content.trim().is_empty() => (content.clone(), true),
        _ => (DEFAULT_PREFERENCES.to_string(), false),
    }
}

fn update_user_preferences_sync(hub_dir: &Path, content: &str) -> Result<()> {
    let prefs_file = hub_dir.join("user_preferences.md");
    if content.trim().is_empty() {
        let _ = std::fs::remove_file(&prefs_file);
    } else {
        std::fs::write(&prefs_file, content)?;
    }
    Ok(())
}

pub async fn get_system_prompt(hub_dir: &Path) -> String {
    let hub_dir = hub_dir.to_path_buf();
    tokio::task::spawn_blocking(move || get_system_prompt_sync(&hub_dir))
        .await
        .unwrap_or_else(|_| DEFAULT_SUPER_SYSTEM_PROMPT.to_string())
}

pub async fn get_system_prompt_info(hub_dir: &Path) -> (String, bool) {
    let hub_dir = hub_dir.to_path_buf();
    tokio::task::spawn_blocking(move || get_system_prompt_info_sync(&hub_dir))
        .await
        .unwrap_or_else(|_| (DEFAULT_SUPER_SYSTEM_PROMPT.to_string(), false))
}

pub async fn update_system_prompt(hub_dir: &Path, prompt: &str) -> Result<()> {
    let hub_dir = hub_dir.to_path_buf();
    let prompt = prompt.to_string();
    tokio::task::spawn_blocking(move || update_system_prompt_sync(&hub_dir, &prompt))
        .await
        .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?
}

pub async fn get_user_preferences(hub_dir: &Path) -> String {
    let hub_dir = hub_dir.to_path_buf();
    tokio::task::spawn_blocking(move || get_user_preferences_sync(&hub_dir))
        .await
        .unwrap_or_else(|_| DEFAULT_PREFERENCES.to_string())
}

pub async fn get_user_preferences_info(hub_dir: &Path) -> (String, bool) {
    let hub_dir = hub_dir.to_path_buf();
    tokio::task::spawn_blocking(move || get_user_preferences_info_sync(&hub_dir))
        .await
        .unwrap_or_else(|_| (DEFAULT_PREFERENCES.to_string(), false))
}

pub async fn update_user_preferences(hub_dir: &Path, content: &str) -> Result<()> {
    let hub_dir = hub_dir.to_path_buf();
    let content = content.to_string();
    tokio::task::spawn_blocking(move || update_user_preferences_sync(&hub_dir, &content))
        .await
        .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?
}
#[derive(Debug, Deserialize)]
struct QueryRingDetailArgs {
    ring_name: String,
}

#[derive(Debug, Deserialize)]
struct UpdatePreferencesArgs {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ManageSkillsArgs {
    action: String,
    name: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateRingArgs {
    name: String,
    role_description: Option<String>,
    storage_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchRingsArgs {
    query: String,
}

#[derive(Debug, Deserialize)]
struct ManageRingMembersArgs {
    action: String,
    ring_name: String,
    target_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GetRingGraphArgs {
    ring_name: String,
}

pub fn get_super_tools() -> Vec<ChatCompletionTool> {
    vec![
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "query_rings".to_string(),
                description: Some(
                    "列出用户所有 Ring 的摘要信息，包括名称、成员数和最近归档标题。当用户询问关于 Ring 的概况时使用。"
                        .to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "query_ring_detail".to_string(),
                description: Some(
                    "读取指定 Ring 的详细数据，包括图谱节点和最近归档内容。当用户想了解某个 Ring 的具体内容时使用。"
                        .to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "ring_name": {
                                "type": "string",
                                "description": "要查询的 Ring 名称"
                            }
                        },
                        "required": ["ring_name"]
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "query_user_preferences".to_string(),
                description: Some(
                    "读取用户的全局偏好设置，包括语言、默认 LLM、输出格式、默认模式等。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "update_user_preferences".to_string(),
                description: Some(
                    "更新用户的全局偏好设置。接收完整的 Markdown 内容覆盖写入户好文件。修改前应先用 query_user_preferences 读取当前内容。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "完整的偏好设置 Markdown 内容"
                            }
                        },
                        "required": ["content"]
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "manage_skills".to_string(),
                description: Some(
                    "管理 Skill 插件。支持三个操作：list（列出所有 Skill）、install（从 URL 安装 Skill）、remove（卸载 Skill）。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": ["list", "install", "remove"],
                                "description": "操作类型"
                            },
                            "name": {
                                "type": "string",
                                "description": "Skill 名称（install/remove 时必填）"
                            },
                            "source_url": {
                                "type": "string",
                                "description": "远程 Skill URL（install 时必填）"
                            }
                        },
                        "required": ["action"]
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "create_ring".to_string(),
                description: Some(
                    "创建一个新的 Ring（群组知识空间）。用户说出想创建的 Ring 名称和用途时使用。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Ring 名称，简洁有意义"
                            },
                            "role_description": {
                                "type": "string",
                                "description": "Group Ring 的角色描述，如「你是技术架构组长，擅长架构决策记录」"
                            },
                            "storage_mode": {
                                "type": "string",
                                "enum": ["local", "gitlab"],
                                "description": "存储模式，默认 local"
                            }
                        },
                        "required": ["name"]
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "search_rings".to_string(),
                description: Some(
                    "Search across all Rings for relevant knowledge. Returns top 10 matching results.".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            }
                        },
                        "required": ["query"]
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "manage_ring_members".to_string(),
                description: Some(
                    "管理 Ring 成员。支持 list（列出成员）、invite（邀请成员）、remove（移除成员）。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": ["list", "invite", "remove"],
                                "description": "操作类型"
                            },
                            "ring_name": {
                                "type": "string",
                                "description": "Ring 名称"
                            },
                            "target_user_id": {
                                "type": "string",
                                "description": "目标用户 ID（invite/remove 时必填）"
                            }
                        },
                        "required": ["action", "ring_name"]
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "get_ring_graph".to_string(),
                description: Some(
                    "获取 Ring 的图谱结构概要，包括所有节点和边。用于了解 Ring 的知识图谱。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "ring_name": {
                                "type": "string",
                                "description": "Ring 名称"
                            }
                        },
                        "required": ["ring_name"]
                    }),
                ),
                strict: None,
            },
        },
    ]
}

pub async fn build_ring_summary(pool: &sqlx::SqlitePool, user_id: &str) -> String {
    let rows = sqlx::query_as::<_, (String, String, i64, Option<String>)>(
        "SELECT r.id, r.name, COUNT(m2.user_id) as member_count,
                GROUP_CONCAT(ar.title, '|') as archive_titles
         FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         LEFT JOIN members m2 ON m2.ring_id = r.id
         LEFT JOIN (
             SELECT ring_id, title
             FROM archive_records
             WHERE status IN ('pushed', 'committed')
             ORDER BY created_at DESC
         ) ar ON ar.ring_id = r.id
         GROUP BY r.id, r.name
         ORDER BY r.created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return "用户目前没有任何 Ring。".to_string();
    }

    let mut summary = String::from("## 用户的所有 Ring\n\n");

    for (_ring_id, ring_name, member_count, archive_titles) in &rows {
        let titles: Vec<&str> = archive_titles
            .as_ref()
            .map(|s| s.split('|').collect())
            .unwrap_or_default();
        let titles: Vec<&str> = titles.into_iter().take(3).collect();

        summary.push_str(&format!("### {ring_name} ({member_count} 成员)\n"));
        if titles.is_empty() {
            summary.push_str("- 暂无归档\n\n");
        } else {
            summary.push_str(&format!("- 最近归档: {}\n\n", titles.join(", ")));
        }
    }

    summary
}

pub async fn execute_tool(
    pool: &sqlx::SqlitePool,
    rings_dir: &Path,
    hub_dir: &Path,
    user_id: &str,
    tool_name: &str,
    arguments: &str,
    state: Option<&AppState>,
) -> Result<String> {
    match tool_name {
        "query_rings" => execute_query_rings(pool, user_id).await,
        "query_ring_detail" => {
            let args: QueryRingDetailArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_query_ring_detail(pool, rings_dir, user_id, &args.ring_name).await
        }
        "query_user_preferences" => execute_query_user_preferences(hub_dir).await,
        "update_user_preferences" => {
            let args: UpdatePreferencesArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_update_user_preferences(hub_dir, &args.content).await
        }
        "manage_skills" => {
            let args: ManageSkillsArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_manage_skills(hub_dir, args).await
        }
        "create_ring" => {
            let args: CreateRingArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            let state = state.ok_or_else(|| RingError::BadRequest("state not available".into()))?;
            execute_create_ring(state, user_id, args).await
        }
        "search_rings" => {
            let args: SearchRingsArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_search_rings(pool, user_id, &args.query).await
        }
        "manage_ring_members" => {
            let args: ManageRingMembersArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            let state = state.ok_or_else(|| RingError::BadRequest("state not available".into()))?;
            execute_manage_ring_members(state, user_id, args).await
        }
        "get_ring_graph" => {
            let args: GetRingGraphArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_get_ring_graph(pool, user_id, &args.ring_name).await
        }
        _ => Err(RingError::BadRequest(format!("unknown tool: {tool_name}"))),
    }
}

async fn execute_query_user_preferences(hub_dir: &Path) -> Result<String> {
    Ok(get_user_preferences(hub_dir).await)
}

async fn execute_update_user_preferences(hub_dir: &Path, content: &str) -> Result<String> {
    update_user_preferences(hub_dir, content).await?;
    Ok("偏好设置已更新。".to_string())
}

async fn execute_manage_skills(hub_dir: &Path, args: ManageSkillsArgs) -> Result<String> {
    let skills_dir = hub_dir
        .parent()
        .map(|p| p.join("skills"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.ring/skills"));

    match args.action.as_str() {
        "list" => {
            let skills = crate::services::skill::list_skills(&skills_dir);
            if skills.is_empty() {
                return Ok("目前没有安装任何 Skill。".to_string());
            }
            let mut result = String::from("## 已安装的 Skill\n\n");
            for s in &skills {
                let source_label = if s.source == "builtin" {
                    "内置"
                } else {
                    "用户"
                };
                result.push_str(&format!(
                    "### {} [{}]\n{}\n\n",
                    s.name, source_label, s.description
                ));
            }
            Ok(result)
        }
        "install" => {
            let name = args.name.unwrap_or_default();
            let url = args.source_url.unwrap_or_default();
            if name.is_empty() || url.is_empty() {
                return Ok("安装 Skill 需要 name 和 source_url 参数。".to_string());
            }
            match crate::services::skill::install_skill(&skills_dir, &name, &url).await {
                Ok(info) => Ok(format!(
                    "Skill '{}' 安装成功：{}",
                    info.name, info.description
                )),
                Err(e) => Ok(format!("Skill 安装失败：{e}")),
            }
        }
        "remove" => {
            let name = args.name.unwrap_or_default();
            if name.is_empty() {
                return Ok("卸载 Skill 需要 name 参数。".to_string());
            }
            match crate::services::skill::remove_skill(&skills_dir, &name) {
                Ok(()) => Ok(format!("Skill '{name}' 已卸载。")),
                Err(e) => Ok(format!("卸载失败：{e}")),
            }
        }
        _ => Ok(format!(
            "未知操作 '{}'。支持: list, install, remove",
            args.action
        )),
    }
}

async fn execute_query_rings(pool: &sqlx::SqlitePool, user_id: &str) -> Result<String> {
    Ok(build_ring_summary(pool, user_id).await)
}

async fn execute_search_rings(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    query: &str,
) -> Result<String> {
    let ring_ids = crate::services::search::get_user_ring_ids(pool, user_id)
        .await
        .unwrap_or_default();
    if ring_ids.is_empty() {
        return Ok("用户目前没有任何 Ring。".to_string());
    }
    let results = crate::services::search::search_cross_ring(pool, &ring_ids, query, 10)
        .await
        .unwrap_or_default();
    if results.is_empty() {
        return Ok("未找到相关结果。".to_string());
    }
    Ok(crate::services::search::format_search_context(&results))
}

async fn resolve_ring_id_by_name(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ring_name: &str,
) -> Result<Option<String>> {
    let ring_id: Option<String> = sqlx::query_scalar(
        "SELECT r.id FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         WHERE r.name LIKE ?2",
    )
    .bind(user_id)
    .bind(format!("%{ring_name}%"))
    .fetch_optional(pool)
    .await?
    .flatten();
    Ok(ring_id)
}

async fn execute_manage_ring_members(
    state: &AppState,
    user_id: &str,
    args: ManageRingMembersArgs,
) -> Result<String> {
    let ring_id = resolve_ring_id_by_name(&state.db, user_id, &args.ring_name).await?;
    let ring_id = match ring_id {
        Some(id) => id,
        None => return Ok(format!("未找到名为「{}」的 Ring。", args.ring_name)),
    };

    match args.action.as_str() {
        "list" => {
            let members = crate::services::member::list_members(state, &ring_id, user_id).await?;
            let mut result = format!("## Ring「{}」的成员\n\n", args.ring_name);
            for m in &members {
                result.push_str(&format!(
                    "- {} ({}): {}\n",
                    m.display_name, m.role, m.token_id
                ));
            }
            Ok(result)
        }
        "invite" => {
            let target_id = match args.target_user_id {
                Some(ref id) if !id.is_empty() => id.clone(),
                _ => return Ok("invite 操作需要 target_user_id 参数。".to_string()),
            };
            match crate::services::member::add_member_service(state, &ring_id, user_id, &target_id)
                .await
            {
                Ok(_) => Ok(format!(
                    "已将用户 {} 添加到 Ring「{}」。",
                    target_id, args.ring_name
                )),
                Err(e) => Ok(format!("添加成员失败：{e}")),
            }
        }
        "remove" => {
            let target_id = match args.target_user_id {
                Some(ref id) if !id.is_empty() => id.clone(),
                _ => return Ok("remove 操作需要 target_user_id 参数。".to_string()),
            };
            match crate::services::member::remove_member(state, &ring_id, user_id, &target_id).await
            {
                Ok(_) => Ok(format!(
                    "已将用户 {} 从 Ring「{}」移除。",
                    target_id, args.ring_name
                )),
                Err(e) => Ok(format!("移除成员失败：{e}")),
            }
        }
        _ => Ok(format!(
            "未知操作 '{}'。支持: list, invite, remove",
            args.action
        )),
    }
}

async fn execute_get_ring_graph(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ring_name: &str,
) -> Result<String> {
    let ring_id = resolve_ring_id_by_name(pool, user_id, ring_name).await?;
    let ring_id = match ring_id {
        Some(id) => id,
        None => return Ok(format!("未找到名为「{ring_name}」的 Ring。")),
    };

    let g = crate::models::graph::ensure_default_graph(pool, &ring_id).await?;
    let nodes = crate::models::graph::list_nodes(pool, &g.id).await?;
    let edges = crate::models::graph::list_edges(pool, &g.id).await?;

    let mut result = format!("## Ring「{ring_name}」图谱概要\n\n");
    result.push_str(&format!("### 节点（共 {} 个）\n\n", nodes.len()));
    for node in &nodes {
        result.push_str(&format!(
            "- {} [{}] {}\n",
            node.label,
            node.node_type,
            if node.tags == "[]" {
                String::new()
            } else {
                format!("tags: {}", node.tags)
            }
        ));
    }
    result.push_str(&format!("\n### 边（共 {} 个）\n\n", edges.len()));
    for edge in &edges {
        let source_label = nodes
            .iter()
            .find(|n| n.id == edge.source_id)
            .map(|n| n.label.as_str())
            .unwrap_or(&edge.source_id);
        let target_label = nodes
            .iter()
            .find(|n| n.id == edge.target_id)
            .map(|n| n.label.as_str())
            .unwrap_or(&edge.target_id);
        result.push_str(&format!(
            "- {} → {} ({})\n",
            source_label, target_label, edge.relation
        ));
    }
    Ok(result)
}

async fn execute_create_ring(
    state: &AppState,
    user_id: &str,
    args: CreateRingArgs,
) -> Result<String> {
    let input = crate::models::ring::CreateRing {
        name: args.name.clone(),
        role_description: args
            .role_description
            .unwrap_or_else(|| format!("You are a {} assistant", args.name)),
        storage_mode: args.storage_mode.unwrap_or_else(|| "local".into()),
        gitlab_repo_url: None,
        gitlab_namespace: None,
    };

    match crate::services::ring::create_ring(state, user_id, input).await {
        Ok(resp) => Ok(format!(
            "Ring「{}」已创建成功！ID: {}。用户可以进入该 Ring 开始使用。",
            resp.name, resp.id
        )),
        Err(e) => Ok(format!("创建 Ring 失败：{e}")),
    }
}

pub async fn execute_query_ring_detail(
    pool: &sqlx::SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    ring_name: &str,
) -> Result<String> {
    let ring_id: Option<String> = sqlx::query_scalar(
        "SELECT r.id FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         WHERE r.name LIKE ?2",
    )
    .bind(user_id)
    .bind(format!("%{ring_name}%"))
    .fetch_optional(pool)
    .await?
    .flatten();

    let ring_id = match ring_id {
        Some(id) => id,
        None => return Ok(format!("未找到名为「{ring_name}」的 Ring。")),
    };

    let rings_dir = rings_dir.to_path_buf();
    let ring_id_clone = ring_id.clone();
    let result =
        tokio::task::spawn_blocking(move || read_ring_detail_sync(&rings_dir, &ring_id_clone))
            .await
            .map_err(|e| RingError::Internal(format!("blocking task failed: {e}")))?;

    if result.is_empty() {
        Ok(format!("Ring「{ring_name}」暂无图谱和归档数据。"))
    } else {
        Ok(result)
    }
}

fn read_ring_detail_sync(rings_dir: &Path, ring_id: &str) -> String {
    let mut result = String::new();

    let graph_path = rings_dir.join(ring_id).join("graph.json");
    if graph_path.exists() {
        match std::fs::read_to_string(&graph_path) {
            Ok(content) => {
                if let Ok(graph) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(nodes) = graph.get("nodes").and_then(|n| n.as_array()) {
                        result.push_str(&format!(
                            "### 图谱节点（共 {} 个，显示前 50 个）\n",
                            nodes.len()
                        ));
                        for node in nodes.iter().take(50) {
                            let label = node.get("label").and_then(|v| v.as_str()).unwrap_or("");
                            let desc = node
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if desc.is_empty() {
                                result.push_str(&format!("- {label}\n"));
                            } else {
                                result.push_str(&format!("- {label}: {desc}\n"));
                            }
                        }
                        result.push('\n');
                    }
                }
            }
            Err(e) => {
                tracing::warn!("failed to read graph.json: {e}");
            }
        }
    }

    let archives_dir = rings_dir.join(ring_id).join("archives");
    if archives_dir.exists() {
        let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(&archives_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();
        entries
            .sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

        result.push_str("### 最近归档\n\n");
        for entry in entries.iter().take(3) {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".md") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let truncated = if content.len() > 500 {
                            let s: String = content.chars().take(500).collect();
                            format!("{s}...（截断）")
                        } else {
                            content
                        };
                        result.push_str(&format!("#### {name}\n{truncated}\n\n"));
                    }
                }
            }
        }
    }

    result
}

async fn save_super_message(
    db: &sqlx::SqlitePool,
    msg_id: &str,
    user_id: &str,
    role: &str,
    content: &str,
    token_usage: Option<&str>,
) {
    let _ = message::insert_message(
        db,
        &message::NewMessage {
            id: msg_id,
            ring_id: Some(SUPER_RING_ID),
            user_id,
            role,
            sender_name: if role == "user" { "" } else { "SUPER RING" },
            content,
            node_refs: &[],
            tag_refs: &[],
            token_usage,
        },
    )
    .await;

    if role != "user" {
        let _ = crate::services::search::upsert_search_index(
            db,
            "message",
            msg_id,
            "super",
            "",
            "SUPER RING",
            content,
            &serde_json::json!({"role": role}).to_string(),
        )
        .await;
    }
}

pub fn stream_super_chat(
    state: AppState,
    user: crate::models::user::UserRow,
    content: String,
) -> tokio::sync::mpsc::Receiver<SseEvent> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        if let Err(e) = stream_super_chat_inner(state, user, content, &tx).await {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
        }
    });

    rx
}

async fn stream_super_chat_inner(
    state: AppState,
    user: crate::models::user::UserRow,
    content: String,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> Result<()> {
    let user_msg_id = ulid::Ulid::new().to_string();
    save_super_message(
        &state.db,
        &user_msg_id,
        &user.token_id,
        "user",
        &content,
        None,
    )
    .await;

    let base_prompt = get_system_prompt(&state.hub_dir).await;
    let ring_summary = crate::services::cross_ring_cache::get_summary(
        &state.cross_ring_cache,
        &state.db,
        &user.token_id,
    )
    .await;
    let prefs = get_user_preferences(&state.hub_dir).await;
    let search_ctx = if content.len() >= 5 && !content.starts_with('/') {
        let ring_ids = crate::services::search::get_user_ring_ids(&state.db, &user.token_id)
            .await
            .unwrap_or_default();
        if !ring_ids.is_empty() {
            let results =
                crate::services::search::search_cross_ring(&state.db, &ring_ids, &content, 20)
                    .await
                    .unwrap_or_default();
            let ctx = crate::services::search::format_search_context(&results);
            if !ctx.is_empty() {
                format!(
                    "\n\n{}\n\n{}",
                    crate::prompts::search::cross_ring_context_instruction(),
                    ctx
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let system_prompt =
        format!("{base_prompt}\n\n{ring_summary}\n\n## 用户偏好\n{prefs}{search_ctx}");

    let history =
        chat::load_history_context(&state.db, Some(SUPER_RING_ID), &user.token_id, 20).await?;

    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_content = apply_filters(&content, &filters);

    let llm = LlmClient::from_user(&user)?;
    let db = state.db.clone();
    let rings_dir = state.rings_dir.clone();
    let hub_dir = state.hub_dir.clone();
    let uid = user.token_id.clone();
    let state_clone = state.clone();

    let mut rx = llm.chat_stream_with_tools(
        system_prompt,
        history,
        filtered_content,
        "super_ring".to_string(),
        get_super_tools(),
        move |name, args| {
            let db = db.clone();
            let rings_dir = rings_dir.clone();
            let hub_dir = hub_dir.clone();
            let uid = uid.clone();
            let st = state_clone.clone();
            let args_str = serde_json::to_string(&args).unwrap_or_default();
            Box::pin(async move {
                execute_tool(&db, &rings_dir, &hub_dir, &uid, &name, &args_str, Some(&st)).await
            })
        },
        |_: String, _: Option<String>| Box::pin(async {}),
    );

    let (msg_id, full_content) = forward_sse_stream(&mut rx, tx).await;

    if let Some(mid) = msg_id {
        save_super_message(
            &state.db,
            &mid,
            &user.token_id,
            "super_ring",
            &full_content,
            None,
        )
        .await;
    }

    Ok(())
}

async fn forward_sse_stream(
    rx: &mut tokio::sync::mpsc::Receiver<SseEvent>,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> (Option<String>, String) {
    let mut full_content = String::new();
    let mut msg_id: Option<String> = None;
    while let Some(event) = rx.recv().await {
        match &event {
            SseEvent::Start { message_id, .. } => {
                msg_id = Some(message_id.clone());
                let _ = tx.send(event).await;
            }
            SseEvent::Delta { content: delta } => {
                full_content.push_str(delta);
                let _ = tx.send(event).await;
            }
            SseEvent::End { .. } => {
                let _ = tx.send(event).await;
            }
            SseEvent::Error(_) => {
                let _ = tx.send(event).await;
            }
        }
    }
    (msg_id, full_content)
}

pub async fn get_super_history(
    state: &AppState,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<message::MessageRow>> {
    chat::get_history(state, Some(SUPER_RING_ID), user_id, before_id, limit).await
}

pub fn stream_cross_ring_query(
    state: AppState,
    user: crate::models::user::UserRow,
    query: String,
) -> tokio::sync::mpsc::Receiver<SseEvent> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        if let Err(e) = stream_cross_ring_query_inner(state, user, query, &tx).await {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
        }
    });

    rx
}

async fn stream_cross_ring_query_inner(
    state: AppState,
    user: crate::models::user::UserRow,
    query: String,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> Result<()> {
    let ring_summary = build_ring_summary(&state.db, &user.token_id).await;

    let mut all_ring_details = String::new();
    let rings = sqlx::query_as::<_, (String, String)>(
        "SELECT r.id, r.name FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         ORDER BY r.created_at",
    )
    .bind(&user.token_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (ring_id, ring_name) in &rings {
        let detail = crate::services::cross_ring_cache::get_detail(
            &state.cross_ring_cache,
            &state.db,
            &state.rings_dir,
            &user.token_id,
            ring_id,
            ring_name,
        )
        .await;
        all_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
    }

    let system_prompt =
        crate::prompts::super_ring::cross_ring_query(&ring_summary, &all_ring_details);

    let llm = LlmClient::from_user(&user)?;

    let mut rx = llm.chat_stream(
        system_prompt,
        vec![],
        query,
        "super_ring".to_string(),
        |_: String, _: Option<String>| Box::pin(async {}),
    );

    let (msg_id, full_content) = forward_sse_stream(&mut rx, tx).await;

    if let Some(mid) = msg_id {
        save_super_message(
            &state.db,
            &mid,
            &user.token_id,
            "super_ring",
            &full_content,
            None,
        )
        .await;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct CrossRingAnalysisRequest {
    pub ring_names: Vec<String>,
    pub analysis_type: String,
    pub question: Option<String>,
}

pub fn stream_cross_ring_analysis(
    state: AppState,
    user: crate::models::user::UserRow,
    request: CrossRingAnalysisRequest,
) -> tokio::sync::mpsc::Receiver<SseEvent> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        if let Err(e) = stream_cross_ring_analysis_inner(state, user, request, &tx).await {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
        }
    });

    rx
}

async fn stream_cross_ring_analysis_inner(
    state: AppState,
    user: crate::models::user::UserRow,
    request: CrossRingAnalysisRequest,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> Result<()> {
    let mut selected_ring_details = String::new();

    for ring_name in &request.ring_names {
        let ring_id: Option<String> = sqlx::query_scalar(
            "SELECT r.id FROM rings r
             JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
             WHERE r.name LIKE ?2",
        )
        .bind(&user.token_id)
        .bind(format!("%{ring_name}%"))
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten();

        if let Some(ring_id) = ring_id {
            let detail = crate::services::cross_ring_cache::get_detail(
                &state.cross_ring_cache,
                &state.db,
                &state.rings_dir,
                &user.token_id,
                &ring_id,
                ring_name,
            )
            .await;
            selected_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
        }
    }

    let analysis_prompt = crate::prompts::super_ring::cross_ring_analysis(
        &request.analysis_type,
        &selected_ring_details,
    );

    let system_prompt =
        "你是 Super Ring，用户的全局 AI 助手。你的任务是分析多个 Ring 的数据并提供洞察。"
            .to_string();

    let llm = LlmClient::from_user(&user)?;

    let mut rx = llm.chat_stream(
        system_prompt,
        vec![],
        analysis_prompt,
        "super_ring".to_string(),
        |_: String, _: Option<String>| Box::pin(async {}),
    );

    let (msg_id, full_content) = forward_sse_stream(&mut rx, tx).await;

    if let Some(mid) = msg_id {
        save_super_message(
            &state.db,
            &mid,
            &user.token_id,
            "super_ring",
            &full_content,
            None,
        )
        .await;
    }

    Ok(())
}
