# Skill Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Skill management — list/install/remove Skills with local + remote support, accessible via Super Ring tool and CLI `%skill` commands.

**Architecture:** File-based Skill storage in `~/.ring/skills/{name}/SKILL.md`. YAML frontmatter + Markdown body. Backend service handles listing, filesystem-based resolution, remote download (reqwest HTTP + git clone), and format validation. New routes module for Skill API endpoints. New `manage_skills` tool for Super Ring. CLI `%skill` commands on frontend.

**Tech Stack:** Rust + Axum (backend), reqwest (HTTP download), git CLI (clone), TypeScript + Zustand (frontend)

---

### Task 1: Add skills_dir to AppState + create directory on startup

**Files:**
- Modify: `server/src/state.rs`
- Modify: `server/src/main.rs`

- [ ] **Step 1: Add skills_dir field to AppState**

In `server/src/state.rs`, add `skills_dir: PathBuf` field to `AppState`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
    pub rings_dir: PathBuf,
    pub hub_dir: PathBuf,
    pub skills_dir: PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool, rings_dir: PathBuf, hub_dir: PathBuf, skills_dir: PathBuf) -> Self {
        Self {
            db,
            ws_hub: WsHub::new(),
            rings_dir,
            hub_dir,
            skills_dir,
        }
    }
}
```

- [ ] **Step 2: Create skills_dir on startup**

In `server/src/main.rs`, add after the `hub_dir` creation (after line 30):

```rust
    let skills_dir = std::path::PathBuf::from(format!("{data_dir}/skills"));
    std::fs::create_dir_all(&skills_dir).expect("failed to create skills dir");
```

Update the `AppState::new` call to include `skills_dir`:

```rust
    let state = AppState::new(pool, rings_dir, hub_dir, skills_dir);
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles — note: all existing tests will need to be updated to pass `skills_dir` to `AppState::new`. We'll fix tests in a later task.

- [ ] **Step 4: Commit**

```bash
git add server/src/state.rs server/src/main.rs
git commit -m "Add skills_dir to AppState and create on startup"
```

---

### Task 2: Fix existing tests for new AppState signature

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Update setup_app helper**

In the `setup_app()` function, add `skills_dir`:

```rust
async fn setup_app() -> AppState {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let rings_dir = std::path::PathBuf::from("/tmp/ring-test-rings");
    let hub_dir = std::path::PathBuf::from("/tmp/ring-test-hub");
    let skills_dir = std::path::PathBuf::from("/tmp/ring-test-skills");
    let _ = std::fs::create_dir_all(&rings_dir);
    let _ = std::fs::create_dir_all(&hub_dir);
    let _ = std::fs::create_dir_all(&skills_dir);

    AppState::new(pool, rings_dir, hub_dir, skills_dir)
}
```

Also check if there's a `setup_unique_app` helper that also creates `AppState` — if so, update it too.

- [ ] **Step 2: Run all tests**

Run: `cd server && cargo test`
Expected: all existing tests pass (21/21)

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Fix tests for new AppState signature with skills_dir"
```

---

### Task 3: Extend skill.rs — ResolvedSkill, list_skills, export_builtin

**Files:**
- Modify: `server/src/services/skill.rs`

This task extends the existing skill service with file-system resolution, listing, and builtin export. No download/install yet (that's Task 4).

- [ ] **Step 1: Add new types and functions**

Rewrite `server/src/services/skill.rs` to the following. This preserves the existing `get_skill`, `build_material_system_prompt`, `build_summary_system_prompt` functions and adds new functionality:

```rust
use std::path::Path;

use crate::error::{Result, RingError};

pub struct SkillDef {
    pub name: &'static str,
    pub description: &'static str,
    pub material_prompt: &'static str,
    pub summary_prompt: &'static str,
}

const SKILLS: &[SkillDef] = &[
    SkillDef {
        name: "decision",
        description: "团队决策：收集材料 → 讨论 → 决策结论 + 行动项",
        material_prompt: "You are assisting a decision-making session. Based on the session title and description, identify and collect relevant documents, data points, and graph nodes. For each material, create a concise summary. List pros, cons, risks, and options related to the decision topic.",
        summary_prompt: "Summarize this decision-making session. Include: 1) The key decision made, 2) Main arguments for and against, 3) Action items with owners, 4) Follow-up dates. Format as structured markdown.",
    },
    SkillDef {
        name: "research",
        description: "研究讨论：收集资源 → 讨论 → 研究报告",
        material_prompt: "You are assisting a research session. Based on the session title and description, collect relevant resources, references, and existing knowledge from the graph. Identify gaps in knowledge and suggest areas to investigate.",
        summary_prompt: "Write a research report summarizing this session. Include: 1) Research question, 2) Key findings, 3) Data sources, 4) Conclusions, 5) Recommendations for further research. Format as structured markdown.",
    },
    SkillDef {
        name: "review",
        description: "评审：收集评审目标 → 讨论 → 评审意见 + 改进建议",
        material_prompt: "You are assisting a review session. Based on the session title and description, collect the review targets (documents, code, designs). Identify review criteria and checklists relevant to the review type.",
        summary_prompt: "Summarize this review session. Include: 1) Items reviewed, 2) Key findings (issues and positive aspects), 3) Improvement suggestions with priority levels, 4) Agreed actions. Format as structured markdown.",
    },
    SkillDef {
        name: "retrospective",
        description: "回顾：收集项目数据 → 讨论 → 经验教训 + 改进计划",
        material_prompt: "You are assisting a retrospective session. Based on the session title and description, collect project timeline data, metrics, and previous retrospective outcomes from the graph. Identify key events and milestones.",
        summary_prompt: "Summarize this retrospective. Include: 1) What went well, 2) What could be improved, 3) Lessons learned, 4) Action items for next cycle. Format as structured markdown.",
    },
    SkillDef {
        name: "knowledge_sharing",
        description: "知识分享：收集材料 → 讨论 → 整理笔记",
        material_prompt: "You are assisting a knowledge sharing session. Based on the session title and description, collect relevant materials, prior discussions, and graph nodes related to the topic. Organize materials into a logical flow for presentation.",
        summary_prompt: "Create organized notes from this knowledge sharing session. Include: 1) Key topics covered, 2) Important takeaways, 3) References and resources mentioned, 4) Open questions. Format as structured markdown.",
    },
];

pub fn get_skill(name: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.name == name)
}

pub fn build_material_system_prompt(
    skill_name: &str,
    session_title: &str,
    session_description: &str,
) -> Option<String> {
    let skill = get_skill(skill_name)?;
    Some(format!(
        "{}\n\nSession: {}\nDescription: {}\n\nAnalyze the topic and provide a structured list of materials that should be prepared for this session. For each material, specify: title, type (document/graph_node/ai_generated), and a brief description of what it should contain.",
        skill.material_prompt,
        session_title,
        if session_description.is_empty() {
            "N/A"
        } else {
            session_description
        },
    ))
}

pub fn build_summary_system_prompt(skill_name: &str) -> Option<String> {
    let skill = get_skill(skill_name)?;
    Some(skill.summary_prompt.to_string())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub source: String,
    pub installed_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSkill {
    pub name: String,
    pub description: String,
    pub source: String,
    pub content: String,
    pub installed_at: Option<String>,
}

pub fn list_skills(skills_dir: &Path) -> Vec<SkillInfo> {
    let mut skills: Vec<SkillInfo> = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    if skills_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(skills_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let skill_md = entry.path().join("SKILL.md");
                        if let Ok(content) = std::fs::read_to_string(&skill_md) {
                            if let Some(frontmatter) = parse_frontmatter(&content) {
                                let name = frontmatter.get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if !name.is_empty() {
                                    seen_names.insert(name.clone());
                                    let is_builtin = SKILLS.iter().any(|s| s.name == name);
                                    let modified = entry.metadata().ok()
                                        .and_then(|m| m.modified().ok())
                                        .map(|t| {
                                            let dt: chrono::DateTime<chrono::Utc> = t.into();
                                            dt.to_rfc3339()
                                        });
                                    skills.push(SkillInfo {
                                        description: frontmatter.get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        source: if is_builtin { "builtin".to_string() } else { "user".to_string() },
                                        installed_at: if is_builtin { None } else { modified },
                                        name,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for builtin in SKILLS {
        if !seen_names.contains(builtin.name) {
            skills.push(SkillInfo {
                name: builtin.name.to_string(),
                description: builtin.description.to_string(),
                source: "builtin".to_string(),
                installed_at: None,
            });
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

pub fn get_skill_resolved(name: &str, skills_dir: &Path) -> Option<ResolvedSkill> {
    let skill_path = skills_dir.join(name).join("SKILL.md");
    if let Ok(content) = std::fs::read_to_string(&skill_path) {
        let frontmatter = parse_frontmatter(&content)?;
        let is_builtin = SKILLS.iter().any(|s| s.name == name);
        let modified = std::fs::metadata(&skill_path).ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                dt.to_rfc3339()
            });
        return Some(ResolvedSkill {
            name: frontmatter.get("name").and_then(|v| v.as_str()).unwrap_or(name).to_string(),
            description: frontmatter.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            source: if is_builtin { "builtin".to_string() } else { "user".to_string() },
            content,
            installed_at: if is_builtin { None } else { modified },
        });
    }

    let builtin = get_skill(name)?;
    export_builtin_skill(name, skills_dir).ok()?;
    let content = std::fs::read_to_string(skills_dir.join(name).join("SKILL.md")).ok()?;
    Some(ResolvedSkill {
        name: builtin.name.to_string(),
        description: builtin.description.to_string(),
        source: "builtin".to_string(),
        content,
        installed_at: None,
    })
}

pub fn remove_skill(skills_dir: &Path, name: &str) -> Result<()> {
    let builtin_names: Vec<&str> = SKILLS.iter().map(|s| s.name).collect();
    if builtin_names.contains(&name) {
        let skill_path = skills_dir.join(name);
        if skill_path.exists() {
            std::fs::remove_dir_all(&skill_path)?;
        }
        return Err(RingError::BadRequest("Cannot remove built-in skill".to_string()));
    }

    let skill_path = skills_dir.join(name);
    if !skill_path.exists() {
        return Err(RingError::NotFound(format!("Skill '{name}' not found")));
    }
    std::fs::remove_dir_all(&skill_path)?;
    Ok(())
}

fn export_builtin_skill(name: &str, skills_dir: &Path) -> std::io::Result<()> {
    let builtin = match get_skill(name) {
        Some(s) => s,
        None => return Ok(()),
    };
    let skill_dir = skills_dir.join(name);
    std::fs::create_dir_all(&skill_dir)?;
    let content = format!(
        "---\nname: {}\ndescription: \"{}\"\nversion: \"1.0.0\"\n---\n\n# {} Skill\n\n{}",
        builtin.name,
        builtin.description.replace('"', "\\\""),
        capitalize_first(builtin.name),
        builtin.material_prompt,
    );
    std::fs::write(skill_dir.join("SKILL.md"), content)?;
    Ok(())
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn parse_frontmatter(content: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let end = trimmed[3..].find("---")?;
    let yaml_str = &trimmed[3..3 + end];
    let yaml_value: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    yaml_value.as_object().cloned()
}

pub fn validate_skill_content(content: &str) -> std::result::Result<(String, String), String> {
    let frontmatter = parse_frontmatter(content)
        .ok_or("Invalid SKILL.md: missing YAML frontmatter")?;
    let name = frontmatter.get("name")
        .and_then(|v| v.as_str())
        .ok_or("Invalid SKILL.md: missing required field 'name'")?
        .to_string();
    let description = frontmatter.get("description")
        .and_then(|v| v.as_str())
        .ok_or("Invalid SKILL.md: missing required field 'description'")?
        .to_string();
    if name.is_empty() {
        return Err("Invalid SKILL.md: 'name' cannot be empty".to_string());
    }
    Ok((name, description))
}

pub fn write_skill_to_dir(skills_dir: &Path, name: &str, content: &str) -> Result<()> {
    let skill_dir = skills_dir.join(name);
    if skill_dir.exists() {
        std::fs::remove_dir_all(&skill_dir)?;
    }
    std::fs::create_dir_all(&skill_dir)?;
    std::fs::write(skill_dir.join("SKILL.md"), content)?;
    Ok(())
}
```

- [ ] **Step 2: Add serde_yaml dependency to Cargo.toml**

Add to `server/Cargo.toml` dependencies:
```toml
serde_yaml = "0.9"
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/services/skill.rs server/Cargo.toml
git commit -m "Extend skill service with list, resolve, export, and validation"
```

---

### Task 4: Add install_skill with remote download

**Files:**
- Modify: `server/src/services/skill.rs`

This task adds the `install_skill` function that downloads Skills from remote URLs.

- [ ] **Step 1: Add install_skill function**

Add to `server/src/services/skill.rs` (after the `write_skill_to_dir` function):

```rust
pub async fn install_skill(
    skills_dir: &Path,
    _name: &str,
    source_url: &str,
) -> Result<SkillInfo> {
    let content = download_skill_content(source_url).await?;

    let (name, description) = validate_skill_content(&content)
        .map_err(|e| RingError::BadRequest(e))?;

    write_skill_to_dir(skills_dir, &name, &content)?;

    let is_builtin = SKILLS.iter().any(|s| s.name == name);
    Ok(SkillInfo {
        name,
        description,
        source: if is_builtin { "builtin".to_string() } else { "user".to_string() },
        installed_at: Some(chrono::Utc::now().to_rfc3339()),
    })
}

async fn download_skill_content(url: &str) -> Result<String> {
    if is_single_file_url(url) {
        download_single_file(url).await
    } else {
        download_from_git_repo(url).await
    }
}

fn is_single_file_url(url: &str) -> bool {
    url.to_lowercase().ends_with(".md")
        || url.contains("raw.githubusercontent.com")
        || url.contains("/raw/")
}

async fn download_single_file(url: &str) -> Result<String> {
    let response = reqwest::get(url).await
        .map_err(|e| RingError::BadRequest(format!("下载失败: {e}")))?;

    if !response.status().is_success() {
        return Err(RingError::BadRequest(format!(
            "下载失败: HTTP {}", response.status()
        )));
    }

    let content = response.text().await
        .map_err(|e| RingError::BadRequest(format!("下载失败: {e}")))?;

    Ok(content)
}

async fn download_from_git_repo(url: &str) -> Result<String> {
    let tmp_dir = std::env::temp_dir().join(format!("ring-skill-{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&tmp_dir)?;

    let result = async {
        crate::services::git_service::GitService::clone(url, &tmp_dir)?;

        let skill_md = find_skill_md(&tmp_dir)
            .ok_or_else(|| RingError::BadRequest(
                "Git 仓库中未找到 SKILL.md 文件".to_string()
            ))?;

        std::fs::read_to_string(&skill_md)
            .map_err(|e| RingError::Internal(format!("读取 SKILL.md 失败: {e}")))
    }.await;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    result
}

fn find_skill_md(dir: &Path) -> Option<std::path::PathBuf> {
    let direct = dir.join("SKILL.md");
    if direct.exists() {
        return Some(direct);
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let candidate = entry.path().join("SKILL.md");
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
    }

    None
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/skill.rs
git commit -m "Add install_skill with remote download support"
```

---

### Task 5: Create Skill API routes

**Files:**
- Create: `server/src/routes/skills.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Create routes/skills.rs**

Create new file `server/src/routes/skills.rs`:

```rust
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::services::skill;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct ListSkillsResponse {
    pub skills: Vec<skill::SkillInfo>,
}

pub async fn list_skills(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<ListSkillsResponse>> {
    let skills = skill::list_skills(&state.skills_dir);
    Ok(Json(ListSkillsResponse { skills }))
}

#[derive(Debug, Deserialize)]
pub struct InstallSkillRequest {
    pub name: String,
    pub source_url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct InstallSkillResponse {
    pub ok: bool,
    pub name: String,
    pub description: String,
}

pub async fn install_skill_handler(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<InstallSkillRequest>,
) -> Result<Json<InstallSkillResponse>> {
    let info = skill::install_skill(&state.skills_dir, &body.name, &body.source_url).await?;
    Ok(Json(InstallSkillResponse {
        ok: true,
        name: info.name,
        description: info.description,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct SkillDetailResponse {
    pub name: String,
    pub description: String,
    pub source: String,
    pub content: String,
}

pub async fn get_skill_detail(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<SkillDetailResponse>> {
    let resolved = skill::get_skill_resolved(&name, &state.skills_dir)
        .ok_or_else(|| crate::error::RingError::NotFound(format!("Skill '{name}' not found")))?;
    Ok(Json(SkillDetailResponse {
        name: resolved.name,
        description: resolved.description,
        source: resolved.source,
        content: resolved.content,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct RemoveSkillResponse {
    pub ok: bool,
    pub name: String,
}

pub async fn remove_skill(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<RemoveSkillResponse>> {
    skill::remove_skill(&state.skills_dir, &name)?;
    Ok(Json(RemoveSkillResponse { ok: true, name }))
}
```

- [ ] **Step 2: Register routes in mod.rs**

In `server/src/routes/mod.rs`:
1. Add `mod skills;` to the module declarations
2. Add these routes before `.with_state(state)`:

```rust
        .route("/skills", get(skills::list_skills))
        .route("/skills/install", post(skills::install_skill_handler))
        .route("/skills/{name}", get(skills::get_skill_detail).delete(skills::remove_skill))
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/skills.rs server/src/routes/mod.rs
git commit -m "Add Skill API routes: list, install, detail, remove"
```

---

### Task 6: Add manage_skills tool to Super Ring

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Add ManageSkillsArgs struct**

Add after the existing `UpdatePreferencesArgs` struct:

```rust
#[derive(Debug, Deserialize)]
struct ManageSkillsArgs {
    action: String,
    name: Option<String>,
    source_url: Option<String>,
}
```

- [ ] **Step 2: Add manage_skills tool to get_super_tools()**

Add to the `vec![]` in `get_super_tools()`:

```rust
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
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
```

- [ ] **Step 3: Add routing in execute_tool()**

Add before the `_` wildcard in `execute_tool()`:

```rust
        "manage_skills" => {
            let args: ManageSkillsArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_manage_skills(pool, rings_dir, hub_dir, user_id, args).await
        }
```

- [ ] **Step 4: Add execute_manage_skills function**

Add before `execute_query_rings`:

```rust
async fn execute_manage_skills(
    _pool: &sqlx::SqlitePool,
    _rings_dir: &Path,
    hub_dir: &Path,
    _user_id: &str,
    args: ManageSkillsArgs,
) -> Result<String> {
    let skills_dir = hub_dir.parent()
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
                let source_label = if s.source == "builtin" { "内置" } else { "用户" };
                result.push_str(&format!("### {} [{}]\n{}\n\n", s.name, source_label, s.description));
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
                Ok(info) => Ok(format!("Skill '{}' 安装成功：{}", info.name, info.description)),
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
        _ => Ok(format!("未知操作 '{}'。支持: list, install, remove", args.action)),
    }
}
```

**Note**: The `skills_dir` is derived from `hub_dir.parent().join("skills")` since `hub_dir` is `~/.ring/hub/` and `skills_dir` is `~/.ring/skills/`. This avoids passing `skills_dir` through `execute_tool()` signature.

- [ ] **Step 5: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 6: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "Add manage_skills tool to Super Ring tool framework"
```

---

### Task 7: Backend integration tests for Skills

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add skill management tests**

Add at the end of the test file. Use `setup_unique_app()` pattern (per-test temp dirs) since Skills use file system.

```rust
async fn setup_unique_skills_app() -> (AppState, String) {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = format!("/tmp/ring-skill-test-{id}");
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory db");
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrations");

    let rings_dir = std::path::PathBuf::from(format!("{tmp}/rings"));
    let hub_dir = std::path::PathBuf::from(format!("{tmp}/hub"));
    let skills_dir = std::path::PathBuf::from(format!("{tmp}/skills"));
    std::fs::create_dir_all(&rings_dir).unwrap();
    std::fs::create_dir_all(&hub_dir).unwrap();
    std::fs::create_dir_all(&skills_dir).unwrap();

    let state = AppState::new(pool, rings_dir, hub_dir, skills_dir);
    (state, tmp)
}

#[tokio::test]
async fn test_skills_list_includes_builtins() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/skills", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    let skills = json["skills"].as_array().unwrap();
    assert!(skills.len() >= 5);
    let names: Vec<&str> = skills.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"decision"));
    assert!(names.contains(&"research"));

    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_skill_detail_builtin() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/skills/decision", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["name"], "decision");
    assert_eq!(json["source"], "builtin");
    assert!(json["content"].as_str().unwrap().contains("---"));

    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_skill_remove_builtin_rejected() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request("DELETE", "/api/skills/decision", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_skill_remove_nonexistent() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request("DELETE", "/api/skills/nonexistent", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}
```

- [ ] **Step 2: Run all tests**

Run: `cd server && cargo test`
Expected: all tests pass

- [ ] **Step 3: Run clippy + fmt**

Run: `cd server && cargo fmt && cargo clippy -- -D warnings`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration tests for Skill management endpoints"
```

---

### Task 8: Frontend — Skill types + API functions

**Files:**
- Create: `ui/src/types/skill.ts`
- Modify: `ui/src/services/api.ts`

- [ ] **Step 1: Create skill types**

Create `ui/src/types/skill.ts`:

```typescript
export interface SkillInfo {
  name: string
  description: string
  source: 'builtin' | 'user'
  installed_at: string | null
}

export interface SkillDetail {
  name: string
  description: string
  source: string
  content: string
}

export interface InstallResult {
  ok: boolean
  name: string
  description: string
}
```

- [ ] **Step 2: Add API functions to api.ts**

Add at the end of `ui/src/services/api.ts`:

```typescript
export async function listSkills(): Promise<{ skills: import('../types/skill').SkillInfo[] }> {
  return api.get('/skills')
}

export async function installSkill(name: string, sourceUrl: string): Promise<import('../types/skill').InstallResult> {
  return api.post('/skills/install', { name, source_url: sourceUrl })
}

export async function getSkillDetail(name: string): Promise<import('../types/skill').SkillDetail> {
  return api.get(`/skills/${encodeURIComponent(name)}`)
}

export async function removeSkill(name: string): Promise<{ ok: boolean; name: string }> {
  return api.delete(`/skills/${encodeURIComponent(name)}`)
}
```

- [ ] **Step 3: Verify TypeScript**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add ui/src/types/skill.ts ui/src/services/api.ts
git commit -m "Add Skill types and API functions for frontend"
```

---

### Task 9: Frontend — %skill CLI command

**Files:**
- Modify: `ui/src/services/command-parser.ts`
- Modify: `ui/src/stores/chat-store.ts`

- [ ] **Step 1: Add skill variant to ParsedCommand**

In `ui/src/services/command-parser.ts`, add to the `ParsedCommand` union:

```typescript
  | { type: 'skill'; subcommand: 'list' | 'install' | 'remove'; name?: string; url?: string }
```

- [ ] **Step 2: Handle %skill in parseCommand**

In `parseCommand`, inside the `%` token handling block (after the `prefs` check, before the general `meta` fallback), add:

```typescript
      if (body === 'skill') {
        const sub = tokens[i + 1]?.toLowerCase()
        if (sub === 'install' && tokens[i + 2] && tokens[i + 3]) {
          commands.push({ type: 'skill', subcommand: 'install', name: tokens[i + 2], url: tokens.slice(i + 3).join(' ') })
        } else if (sub === 'remove' && tokens[i + 2]) {
          commands.push({ type: 'skill', subcommand: 'remove', name: tokens[i + 2] })
        } else {
          commands.push({ type: 'skill', subcommand: 'list' })
        }
        break
      }
```

This goes between the `if (body === 'prefs')` block and the general `const nextToken = tokens[i + 1]` line.

- [ ] **Step 3: Add import and handler in chat-store.ts**

Add import at top of `ui/src/stores/chat-store.ts`:

```typescript
import { listSkills, installSkill, removeSkill } from '../services/api'
```

Add handler functions after the `handlePrefsSet` function:

```typescript
async function handleSkillList(addMessage: (msg: import('../types/chat').ChatMessage) => void) {
  try {
    const { skills } = await listSkills()
    if (skills.length === 0) {
      addMessage({ id: `sys-skill-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: 'No skills installed.', created_at: new Date().toISOString() })
      return
    }
    const lines = skills.map(s => {
      const tag = s.source === 'builtin' ? '[built-in]' : '[user]'
      return `- **${s.name}** ${tag}: ${s.description}`
    })
    addMessage({ id: `sys-skill-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: `## Skills\n\n${lines.join('\n')}`, created_at: new Date().toISOString() })
  } catch {
    addMessage({ id: `sys-skill-err-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: 'Failed to load skills.', created_at: new Date().toISOString() })
  }
}

async function handleSkillInstall(name: string, url: string, addMessage: (msg: import('../types/chat').ChatMessage) => void) {
  try {
    const result = await installSkill(name, url)
    addMessage({ id: `sys-skill-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: result.ok ? `Skill "${result.name}" installed: ${result.description}` : `Install failed`, created_at: new Date().toISOString() })
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-skill-err-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: `Skill install failed: ${msg}`, created_at: new Date().toISOString() })
  }
}

async function handleSkillRemove(name: string, addMessage: (msg: import('../types/chat').ChatMessage) => void) {
  try {
    await removeSkill(name)
    addMessage({ id: `sys-skill-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: `Skill "${name}" removed.`, created_at: new Date().toISOString() })
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-skill-err-${Date.now()}`, role: 'system', sender_name: 'SYSTEM', content: `Failed to remove skill: ${msg}`, created_at: new Date().toISOString() })
  }
}
```

Add `skill` case in the switch inside `send()`, after the `prefs` case:

```typescript
          case 'skill': {
            if (cmd.subcommand === 'install' && cmd.name && cmd.url) {
              handleSkillInstall(cmd.name, cmd.url, addMessage)
            } else if (cmd.subcommand === 'remove' && cmd.name) {
              handleSkillRemove(cmd.name, addMessage)
            } else {
              handleSkillList(addMessage)
            }
            break
          }
```

- [ ] **Step 4: Verify TypeScript**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add ui/src/services/command-parser.ts ui/src/stores/chat-store.ts
git commit -m "Add %skill list/install/remove CLI commands"
```

---

### Task 10: Final verification

- [ ] **Step 1: Run all backend tests**

Run: `cd server && cargo test`
Expected: all tests pass

- [ ] **Step 2: Run clippy + fmt**

Run: `cd server && cargo fmt --check && cargo clippy -- -D warnings`
Expected: no errors

- [ ] **Step 3: Run frontend build**

Run: `cd ui && npm run build`
Expected: build succeeds

- [ ] **Step 4: Fix any issues and commit**

```bash
git add -A
git commit -m "Final fixes for Skill Management feature"
```
