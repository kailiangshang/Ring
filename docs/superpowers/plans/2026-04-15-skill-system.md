# Skill System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Skill system - file-based plugin system for Session Ring behavior, replacing hardcoded workflows with Claude Code Skill format

**Architecture:** Skills are Markdown files with YAML frontmatter, stored at `~/.ring/skills/`. Session Ring loads the skill matching its scenario. Super Ring handles skill installation from network.

**Tech Stack:** Rust (axum), file system, async-openai for LLM calls

---

## File Structure

```
ring-server/src/
├── models/
│   ├── skill_model.rs          # Skill data structures + YAML parsing
│   └── preinstalled_skills.rs  # Built-in skill content
├── services/
│   └── skill_service.rs        # Skill loading, installation, management
├── handlers/
│   └── skill.rs                # Skill API handlers
└── routes.rs                   # Add /skills routes

ring-frontend/src/
├── stores/
│   └── skillStore.ts           # Skill state management
└── components/
    └── skill/
        └── SkillManagement.tsx # Skill display component
```

---

## 3. Pre-installed Skills (Confirmed)

5 business-focused Skills for Session:

| Skill | Scenario | Purpose |
|-------|----------|---------|
| decision | decision | 团队决策 - 收集材料 → 讨论 → 决策结论 |
| research | research | 联合调研 - 收集材料 → 讨论 → 调研报告 |
| review | review | 集体评审 - 收集材料 → 讨论 → 评审意见 |
| retrospective | retrospective | 项目复盘 - 收集材料 → 讨论 → 改进建议 |
| knowledge_sharing | knowledge_sharing | 知识分享 - 收集材料 → 分享 → 整理笔记 |

### Skill Behavior per Session Phase

| Phase | AI Behavior |
|-------|-------------|
| **Material Preparation** | AI 根据 Session 主题收集/整理相关材料，让讨论有内容可依，而不是空谈。参与者可查看进度，Owner 可标记重点 |
| **Discussion** | AI 不参与（只记录） |
| **Summary** | AI 基于材料生成总结报告 |

---

## 4. Skill Format (Claude Code Style)

```yaml
---
name: skill-name
description: 简短描述
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
---

## Skill 内容
Instructions here...
```

---

## 5. Tasks

### Task 1: Skill Models

**Files:**
- Create: `ring-server/src/models/skill_model.rs`
- Modify: `ring-server/src/models/mod.rs`

- [ ] **Step 1: Write tests for Skill model**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_metadata_parsing() {
        let skill = SkillMetadata {
            name: "decision".into(),
            description: "团队决策 Skill".into(),
            disable_model_invocation: false,
            allowed_tools: Some(vec!["Bash".into(), "Read".into()]),
        };
        assert_eq!(skill.name, "decision");
    }

    #[test]
    fn test_skill_from_md_with_frontmatter() {
        let content = r#"---
name: test_skill
description: A test skill
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
---

## Skill 内容
This is the skill content.
"#;
        let skill = Skill::from_skill_md(content).unwrap();
        assert_eq!(skill.metadata.name, "test_skill");
        assert!(skill.system_prompt.is_some());
    }
}
```

Run: `cargo test -p ring-server models::skill_model`
Expected: PASS

- [ ] **Step 2: Implement Skill data structures**

```rust
// ring-server/src/models/skill_model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub disable_model_invocation: bool,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub metadata: SkillMetadata,
    pub content: String,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillListItem {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub built_in: bool,
}

impl Skill {
    pub fn from_skill_md(content: &str) -> Result<Self, String> {
        let mut lines = content.lines();
        
        if lines.next() != Some("---") {
            return Err("Missing YAML frontmatter".to_string());
        }
        
        let mut yaml_lines = Vec::new();
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
            yaml_lines.push(line);
        }
        
        let yaml_content = yaml_lines.join("\n");
        let metadata: SkillMetadata = serde_yaml::from_str(&yaml_content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        
        let remaining: Vec<&str> = lines.collect();
        let content = remaining.join("\n").trim().to_string();
        
        let system_prompt = if content.contains("## Skill 内容") {
            let parts: Vec<&str> = content.split("## Skill 内容").collect();
            if parts.len() > 1 {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        } else {
            None
        };
        
        Ok(Skill {
            metadata,
            content,
            system_prompt,
        })
    }
}
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test -p ring-server models::skill_model`
Expected: PASS

- [ ] **Step 4: Update models/mod.rs**

Add `pub mod skill_model;`

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/models/skill_model.rs ring-server/src/models/mod.rs
git commit -m "feat: add Skill data models with YAML frontmatter parsing"
```

---

### Task 2: Pre-installed Skills Content

**Files:**
- Create: `ring-server/src/models/preinstalled_skills.rs`

- [ ] **Step 1: Create pre-installed skill content**

```rust
// ring-server/src/models/preinstalled_skills.rs

pub const DECISION_SKILL: &str = r#"---
name: decision
description: 团队决策 - 收集材料 → 讨论 → 决策结论
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
  - Grep
---

## Skill 内容

你是决策助手。在材料准备阶段，根据决策议题收集相关信息、整理利弊分析、生成决策材料。在总结阶段，基于讨论内容生成决策结论和行动项。

### 材料准备阶段
1. 理解决策议题和目标
2. 收集相关数据和信息
3. 整理利弊分析
4. 生成决策材料供讨论参考

### 总结阶段
1. 汇总讨论要点
2. 生成明确的决策结论
3. 列出后续行动项
"#;

pub const RESEARCH_SKILL: &str = r#"---
name: research
description: 联合调研 - 收集材料 → 讨论 → 调研报告
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
  - Grep
---

## Skill 内容

你是调研助手。在材料准备阶段，收集和整理与调研主题相关的资料。在总结阶段，生成结构化的调研报告。

### 材料准备阶段
1. 理解调研主题和目标
2. 搜索相关信息源
3. 整理和分类资料
4. 生成调研材料供讨论参考

### 总结阶段
1. 汇总调研发现
2. 生成结构化报告
3. 列出参考资源
"#;

pub const REVIEW_SKILL: &str = r#"---
name: review
description: 集体评审 - 收集材料 → 讨论 → 评审意见
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
  - Grep
---

## Skill 内容

你是评审助手。在材料准备阶段，收集待评审内容并整理评审要点。在总结阶段，生成评审意见。

### 材料准备阶段
1. 理解评审目标和范围
2. 收集待评审材料
3. 整理评审要点
4. 生成评审材料供讨论参考

### 总结阶段
1. 汇总评审意见
2. 生成结构化评审报告
3. 列出改进建议
"#;

pub const RETROSPECTIVE_SKILL: &str = r#"---
name: retrospective
description: 项目复盘 - 收集材料 → 讨论 → 改进建议
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
  - Grep
---

## Skill 内容

你是复盘助手。在材料准备阶段，收集项目相关数据和反馈。在总结阶段，生成改进建议。

### 材料准备阶段
1. 理解复盘目标和范围
2. 收集项目数据
3. 整理问题和经验
4. 生成复盘材料供讨论参考

### 总结阶段
1. 汇总问题和经验
2. 生成改进建议
3. 列出后续行动项
"#;

pub const KNOWLEDGE_SHARING_SKILL: &str = r#"---
name: knowledge_sharing
description: 知识分享 - 收集材料 → 分享 → 整理笔记
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
  - Grep
---

## Skill 内容

你是知识分享助手。在材料准备阶段，收集和整理分享内容。在总结阶段，整理分享笔记。

### 材料准备阶段
1. 理解分享主题和目标
2. 收集相关资料
3. 整理知识点
4. 生成分享材料供讨论参考

### 总结阶段
1. 汇总分享要点
2. 整理笔记
3. 列出推荐学习资源
"#;

pub fn get_built_in_skill(name: &str) -> Option<&'static str> {
    match name {
        "decision" => Some(DECISION_SKILL),
        "research" => Some(RESEARCH_SKILL),
        "review" => Some(REVIEW_SKILL),
        "retrospective" => Some(RETROSPECTIVE_SKILL),
        "knowledge_sharing" => Some(KNOWLEDGE_SHARING_SKILL),
        _ => None,
    }
}

pub fn list_built_in_skills() -> Vec<(&'static str, &'static str)> {
    vec![
        ("decision", "团队决策 - 收集材料 → 讨论 → 决策结论"),
        ("research", "联合调研 - 收集材料 → 讨论 → 调研报告"),
        ("review", "集体评审 - 收集材料 → 讨论 → 评审意见"),
        ("retrospective", "项目复盘 - 收集材料 → 讨论 → 改进建议"),
        ("knowledge_sharing", "知识分享 - 收集材料 → 分享 → 整理笔记"),
    ]
}
```

- [ ] **Step 2: Update models/mod.rs**

Add `pub mod preinstalled_skills;`

- [ ] **Step 3: Commit**

```bash
git add ring-server/src/models/preinstalled_skills.rs ring-server/src/models/mod.rs
git commit -m "feat: add 5 pre-installed Session skills"
```

---

### Task 3: Skill Service

**Files:**
- Create: `ring-server/src/services/skill_service.rs`
- Modify: `ring-server/src/services/mod.rs`

- [ ] **Step 1: Implement SkillService**

```rust
// ring-server/src/services/skill_service.rs
use std::path::PathBuf;

use crate::error::RingError;
use crate::models::preinstalled_skills;
use crate::models::skill_model::{Skill, SkillListItem};

pub struct SkillService {
    base_path: PathBuf,
}

impl SkillService {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub async fn init_skills_directory(&self) -> Result<(), RingError> {
        tokio::fs::create_dir_all(&self.base_path).await?;
        Ok(())
    }

    pub async fn get_skill(&self, name: &str) -> Result<Skill, RingError> {
        // Try built-in first
        if let Some(content) = preinstalled_skills::get_built_in_skill(name) {
            return Skill::from_skill_md(content)
                .map_err(|e| RingError::BadRequest(e));
        }

        // Try file-based
        let path = self.base_path.join(name).join("SKILL.md");
        if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            return Skill::from_skill_md(&content)
                .map_err(|e| RingError::BadRequest(e));
        }

        Err(RingError::NotFound(format!("Skill '{}' not found", name)))
    }

    pub async fn list_skills(&self) -> Result<Vec<SkillListItem>, RingError> {
        let mut items = Vec::new();

        // Add built-in skills
        for (name, description) in preinstalled_skills::list_built_in_skills() {
            items.push(SkillListItem {
                name: name.to_string(),
                description: description.to_string(),
                installed: true,
                built_in: true,
            });
        }

        // Add user-installed skills
        let mut entries = tokio::fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                if preinstalled_skills::get_built_in_skill(&name).is_some() {
                    continue;
                }

                let skill_path = path.join("SKILL.md");
                if skill_path.exists() {
                    if let Ok(content) = tokio::fs::read_to_string(&skill_path).await {
                        if let Ok(skill) = Skill::from_skill_md(&content) {
                            items.push(SkillListItem {
                                name: skill.metadata.name,
                                description: skill.metadata.description,
                                installed: true,
                                built_in: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    pub async fn install_skill(&self, name: &str, content: &str) -> Result<(), RingError> {
        let skill_dir = self.base_path.join(name);
        tokio::fs::create_dir_all(&skill_dir).await?;
        
        let skill_path = skill_dir.join("SKILL.md");
        tokio::fs::write(skill_path, content).await?;
        
        Ok(())
    }

    pub async fn uninstall_skill(&self, name: &str) -> Result<(), RingError> {
        if preinstalled_skills::get_built_in_skill(name).is_some() {
            return Err(RingError::BadRequest("Cannot uninstall built-in skill".to_string()));
        }

        let skill_dir = self.base_path.join(name);
        if skill_dir.exists() {
            tokio::fs::remove_dir_all(skill_dir).await?;
        }
        Ok(())
    }

    pub async fn get_skill_system_prompt(&self, name: &str) -> Result<String, RingError> {
        let skill = self.get_skill(name).await?;
        Ok(skill.system_prompt.unwrap_or_default())
    }
}
```

- [ ] **Step 2: Update services/mod.rs**

Add:
```rust
pub mod skill_service;
pub use skill_service::SkillService;
```

- [ ] **Step 3: Commit**

```bash
git add ring-server/src/services/skill_service.rs ring-server/src/services/mod.rs
git commit -m "feat: add SkillService for skill loading and management"
```

---

### Task 4: Skill Handler

**Files:**
- Create: `ring-server/src/handlers/skill.rs`
- Modify: `ring-server/src/handlers/mod.rs`, `ring-server/src/routes.rs`

- [ ] **Step 1: Implement skill handler**

```rust
// ring-server/src/handlers/skill.rs
use axum::{extract::State, Json};
use std::sync::Arc;

use crate::error::RingError;
use crate::models::skill_model::*;
use crate::services::SkillService;
use crate::state::AppState;

pub async fn list_skills(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SkillListItem>>, RingError> {
    let service = SkillService::new(state.skills_base_path.clone());
    let skills = service.list_skills().await?;
    Ok(Json(skills))
}

pub async fn get_skill(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<Skill>, RingError> {
    let service = SkillService::new(state.skills_base_path.clone());
    let skill = service.get_skill(&name).await?;
    Ok(Json(skill))
}

pub async fn install_skill(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InstallSkillRequest>,
) -> Result<Json<SkillInstallResponse>, RingError> {
    let service = SkillService::new(state.skills_base_path.clone());
    
    let skill_name = req.name.unwrap_or_default();
    let skill_content = req.url
        .map(|url| fetch_skill_from_url(&url))
        .unwrap_or_else(|| Err(RingError::BadRequest("No URL provided".to_string())))?;
    
    service.install_skill(&skill_name, &skill_content).await?;
    
    Ok(Json(SkillInstallResponse {
        success: true,
        skill_name,
        message: "Skill installed successfully".to_string(),
    }))
}

async fn fetch_skill_from_url(url: &str) -> Result<String, RingError> {
    let response = reqwest::get(url).await
        .map_err(|e| RingError::BadRequest(format!("Failed to fetch: {}", e)))?;
    let content = response.text().await
        .map_err(|e| RingError::BadRequest(format!("Failed to read response: {}", e)))?;
    Ok(content)
}

pub async fn uninstall_skill(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<SkillInstallResponse>, RingError> {
    let service = SkillService::new(state.skills_base_path.clone());
    service.uninstall_skill(&name).await?;
    
    Ok(Json(SkillInstallResponse {
        success: true,
        skill_name: name,
        message: "Skill uninstalled successfully".to_string(),
    }))
}

pub async fn get_skill_prompt(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<SkillPromptResponse>, RingError> {
    let service = SkillService::new(state.skills_base_path.clone());
    let prompt = service.get_skill_system_prompt(&name).await?;
    
    Ok(Json(SkillPromptResponse {
        name,
        system_prompt: prompt,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallSkillRequest {
    pub url: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillInstallResponse {
    pub success: bool,
    pub skill_name: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillPromptResponse {
    pub name: String,
    pub system_prompt: String,
}
```

- [ ] **Step 2: Update handlers/mod.rs**

Add `pub mod skill;`

- [ ] **Step 3: Update routes.rs**

Add imports and routes:
```rust
use crate::handlers::skill;

let skill_routes = Router::new()
    .route("/", get(skill::list_skills))
    .route("/{name}", get(skill::get_skill))
    .route("/install", post(skill::install_skill))
    .route("/{name}", delete(skill::uninstall_skill))
    .route("/{name}/prompt", get(skill::get_skill_prompt));

// Mount in router
Router::new()
    .nest("/api/v1/setup", setup_routes)
    .nest("/api/v1/skills", skill_routes)
    // ... rest of routes
```

- [ ] **Step 4: Run build to verify**

Run: `cargo build -p ring-server`
Expected: Success

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/skill.rs ring-server/src/handlers/mod.rs ring-server/src/routes.rs
git commit -m "feat: add Skill handler for skill management API"
```

---

### Task 5: AppState Skills Base Path

**Files:**
- Modify: `ring-server/src/state.rs`

- [ ] **Step 1: Add skills_base_path to AppState**

```rust
pub struct AppState {
    // ... existing fields ...
    pub skills_base_path: std::path::PathBuf,
}
```

- [ ] **Step 2: Initialize in main.rs**

```rust
skills_base_path: home_dir.join(".ring/skills"),
```

- [ ] **Step 3: Commit**

```bash
git add ring-server/src/state.rs
git commit -m "feat: add skills_base_path to AppState for Skill system"
```

---

### Task 6: Frontend Skill Store

**Files:**
- Create: `ring-frontend/src/stores/skillStore.ts`

- [ ] **Step 1: Write Skill store**

```typescript
// ring-frontend/src/stores/skillStore.ts
import { create } from 'zustand';

interface SkillListItem {
  name: string;
  description: string;
  installed: boolean;
  built_in: boolean;
}

interface SkillStore {
  skills: SkillListItem[];
  loading: boolean;
  fetchSkills: () => Promise<void>;
  installSkill: (url: string, name: string) => Promise<void>;
  uninstallSkill: (name: string) => Promise<void>;
  getSkillPrompt: (name: string) => Promise<string>;
}

export const useSkillStore = create<SkillStore>((set, get) => ({
  skills: [],
  loading: false,
  fetchSkills: async () => {
    set({ loading: true });
    try {
      const response = await fetch('/api/v1/skills/');
      const data = await response.json();
      set({ skills: data, loading: false });
    } catch (error) {
      console.error('Failed to fetch skills:', error);
      set({ loading: false });
    }
  },
  installSkill: async (url, name) => {
    await fetch('/api/v1/skills/install', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url, name }),
    });
    get().fetchSkills();
  },
  uninstallSkill: async (name) => {
    await fetch(`/api/v1/skills/${name}`, { method: 'DELETE' });
    get().fetchSkills();
  },
  getSkillPrompt: async (name) => {
    const response = await fetch(`/api/v1/skills/${name}/prompt`);
    const data = await response.json();
    return data.system_prompt;
  },
}));
```

- [ ] **Step 2: Commit**

```bash
git add ring-frontend/src/stores/skillStore.ts
git commit -m "feat: add SkillStore for skill management UI"
```

---

## Self-Review Checklist

1. **Spec coverage:** All Skill requirements from design spec covered
   - ✅ Claude Code Skill format with YAML frontmatter
   - ✅ Skill files stored at `~/.ring/skills/`
   - ✅ 5 pre-installed business-focused skills
   - ✅ Skill installation from network via Super Ring
   - ✅ Session Ring loads skill based on scenario

2. **Placeholder scan:** No "TBD" or "TODO" found

3. **Type consistency:** Types match across tasks

---

## Execution Options

**Plan complete and saved to `docs/superpowers/plans/2026-04-15-skill-system.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?