use std::path::Path;

use crate::error::{Result, RingError};

pub struct SkillDef {
    pub name: &'static str,
    pub description: &'static str,
}

const SKILLS: &[SkillDef] = &[
    SkillDef {
        name: "decision",
        description: "团队决策：收集材料 → 讨论 → 决策结论 + 行动项",
    },
    SkillDef {
        name: "research",
        description: "研究讨论：收集资源 → 讨论 → 研究报告",
    },
    SkillDef {
        name: "review",
        description: "评审：收集评审目标 → 讨论 → 评审意见 + 改进建议",
    },
    SkillDef {
        name: "retrospective",
        description: "回顾：收集项目数据 → 讨论 → 经验教训 + 改进计划",
    },
    SkillDef {
        name: "knowledge_sharing",
        description: "知识分享：收集材料 → 讨论 → 整理笔记",
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
    let material = crate::prompts::session::skill::material_prompt(skill_name)?;
    Some(format!(
        "{material}\n\nSession: {session_title}\nDescription: {}\n\n\
        Analyze the topic and provide a structured list of materials that should be prepared. \
        For each material, specify: title, type (document/graph_node/ai_generated), and a brief description.",
        if session_description.is_empty() { "N/A" } else { session_description }
    ))
}

pub fn build_summary_system_prompt(skill_name: &str) -> Option<String> {
    crate::prompts::session::skill::summary_prompt(skill_name).map(|s| s.to_string())
}

pub fn load_skill_prompt(skill_name: &str, skills_dir: &Path) -> Option<String> {
    let resolved = get_skill_resolved(skill_name, skills_dir)?;
    let frontmatter = parse_frontmatter(&resolved.content)?;
    frontmatter
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
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
                                let name = frontmatter
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if !name.is_empty() {
                                    seen_names.insert(name.clone());
                                    let is_builtin = SKILLS.iter().any(|s| s.name == name);
                                    let modified =
                                        entry.metadata().ok().and_then(|m| m.modified().ok()).map(
                                            |t| {
                                                let dt: chrono::DateTime<chrono::Utc> = t.into();
                                                dt.to_rfc3339()
                                            },
                                        );
                                    skills.push(SkillInfo {
                                        description: frontmatter
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        source: if is_builtin {
                                            "builtin".to_string()
                                        } else {
                                            "user".to_string()
                                        },
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
        let modified = std::fs::metadata(&skill_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                dt.to_rfc3339()
            });
        return Some(ResolvedSkill {
            name: frontmatter
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(name)
                .to_string(),
            description: frontmatter
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source: if is_builtin {
                "builtin".to_string()
            } else {
                "user".to_string()
            },
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
        return Err(RingError::BadRequest(
            "Cannot remove built-in skill".to_string(),
        ));
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
    let material = crate::prompts::session::skill::material_prompt(name).unwrap_or("");
    let summary = crate::prompts::session::skill::summary_prompt(name).unwrap_or("");
    let system_prompt = format!(
        "{}\n\nAnalyze the topic and provide a structured list of materials that should be prepared for this session.",
        material
    );
    let content = format!(
        "---\nname: {}\ndescription: \"{}\"\nversion: \"1.0.0\"\nsystem_prompt: \"{}\"\n---\n\n# {} Skill\n\n{}\n\n## Summary\n\n{}",
        builtin.name,
        builtin.description.replace('"', "\\\""),
        system_prompt.replace('"', "\\\""),
        capitalize_first(builtin.name),
        material,
        summary,
    );
    let skill_dir = skills_dir.join(name);
    std::fs::create_dir_all(&skill_dir)?;
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
    let frontmatter =
        parse_frontmatter(content).ok_or("Invalid SKILL.md: missing YAML frontmatter")?;
    let name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Invalid SKILL.md: missing required field 'name'")?
        .to_string();
    let description = frontmatter
        .get("description")
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

pub async fn install_skill(skills_dir: &Path, _name: &str, source_url: &str) -> Result<SkillInfo> {
    let content = download_skill_content(source_url).await?;

    let (name, description) = validate_skill_content(&content).map_err(RingError::BadRequest)?;

    write_skill_to_dir(skills_dir, &name, &content)?;

    let is_builtin = SKILLS.iter().any(|s| s.name == name);
    Ok(SkillInfo {
        name,
        description,
        source: if is_builtin {
            "builtin".to_string()
        } else {
            "user".to_string()
        },
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
    let response = reqwest::get(url)
        .await
        .map_err(|e| RingError::BadRequest(format!("下载失败: {e}")))?;

    if !response.status().is_success() {
        return Err(RingError::BadRequest(format!(
            "下载失败: HTTP {}",
            response.status()
        )));
    }

    let content = response
        .text()
        .await
        .map_err(|e| RingError::BadRequest(format!("下载失败: {e}")))?;

    Ok(content)
}

async fn download_from_git_repo(url: &str) -> Result<String> {
    let tmp_dir = std::env::temp_dir().join(format!("ring-skill-{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&tmp_dir)?;

    let result = async {
        crate::services::git_service::GitService::clone(url, &tmp_dir)?;

        let skill_md = find_skill_md(&tmp_dir)
            .ok_or_else(|| RingError::BadRequest("Git 仓库中未找到 SKILL.md 文件".to_string()))?;

        std::fs::read_to_string(&skill_md)
            .map_err(|e| RingError::Internal(format!("读取 SKILL.md 失败: {e}")))
    }
    .await;

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
