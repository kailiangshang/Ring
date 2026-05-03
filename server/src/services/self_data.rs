use std::path::Path;

use crate::error::Result;

const SELF_DIR: &str = ".self";
const METRICS_DIR: &str = "metrics";

fn ensure_dirs(self_dir: &Path) {
    let _ = std::fs::create_dir_all(self_dir.join(SELF_DIR));
    let _ = std::fs::create_dir_all(self_dir.join(METRICS_DIR));
}

pub fn read_self_file(self_dir: &Path, name: &str) -> Result<(String, bool)> {
    let path = self_dir.join(SELF_DIR).join(format!("{name}.md"));
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok((content, true)),
        Err(_) => Ok((String::new(), false)),
    }
}

pub fn write_self_file(self_dir: &Path, name: &str, content: &str) -> Result<()> {
    ensure_dirs(self_dir);
    let path = self_dir.join(SELF_DIR).join(format!("{name}.md"));
    Ok(std::fs::write(&path, content)?)
}

pub fn read_metrics(self_dir: &Path) -> serde_json::Value {
    ensure_dirs(self_dir);
    let mut result = serde_json::Map::new();
    for name in &[
        "session_stats",
        "tool_usage",
        "dwell_time",
        "archive_patterns",
        "chat_patterns",
        "ring_activity",
    ] {
        let path = self_dir.join(METRICS_DIR).join(format!("{name}.json"));
        let val = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
        result.insert((*name).to_string(), val.unwrap_or(serde_json::Value::Null));
    }
    serde_json::Value::Object(result)
}

fn read_metric_file(self_dir: &Path, name: &str) -> serde_json::Value {
    let path = self_dir.join(METRICS_DIR).join(format!("{name}.json"));
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

fn write_metric_file(self_dir: &Path, name: &str, data: &serde_json::Value) -> Result<()> {
    ensure_dirs(self_dir);
    let path = self_dir.join(METRICS_DIR).join(format!("{name}.json"));
    let json = serde_json::to_string_pretty(data)?;
    Ok(std::fs::write(&path, json)?)
}

pub fn record_chat_message(
    self_dir: &Path,
    ring_id: Option<&str>,
    content_len: usize,
) -> Result<()> {
    let mut stats = read_metric_file(self_dir, "chat_patterns");
    let total_messages = stats
        .get("total_messages")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    let total_chars = stats
        .get("total_chars")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + content_len as i64;
    let avg_length = if total_messages > 0 {
        total_chars / total_messages
    } else {
        0
    };

    stats["total_messages"] = serde_json::json!(total_messages);
    stats["total_chars"] = serde_json::json!(total_chars);
    stats["avg_message_length"] = serde_json::json!(avg_length);

    if let Some(ring_id) = ring_id {
        let ring_key = format!("ring_{}", ring_id);
        let ring_count = stats.get(&ring_key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
        stats[ring_key] = serde_json::json!(ring_count);
    } else {
        let self_count = stats
            .get("self_messages")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            + 1;
        stats["self_messages"] = serde_json::json!(self_count);
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let daily_key = format!("daily_{}", today);
    let daily_count = stats.get(&daily_key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
    stats[daily_key] = serde_json::json!(daily_count);

    write_metric_file(self_dir, "chat_patterns", &stats)
}

pub fn record_session_created(self_dir: &Path) -> Result<()> {
    let mut stats = read_metric_file(self_dir, "session_stats");
    let total = stats
        .get("total_sessions")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    stats["total_sessions"] = serde_json::json!(total);
    write_metric_file(self_dir, "session_stats", &stats)
}

pub fn record_archive_operation(self_dir: &Path, ring_id: &str, file_name: &str) -> Result<()> {
    let mut patterns = read_metric_file(self_dir, "archive_patterns");
    let total = patterns
        .get("total_archives")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    patterns["total_archives"] = serde_json::json!(total);

    let ring_key = format!("ring_{}", ring_id);
    let ring_count = patterns
        .get(&ring_key)
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    patterns[ring_key] = serde_json::json!(ring_count);

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    patterns["last_archive_date"] = serde_json::json!(today);
    patterns["last_archive_file"] = serde_json::json!(file_name);

    write_metric_file(self_dir, "archive_patterns", &patterns)
}

pub fn record_ring_joined(self_dir: &Path, ring_id: &str, ring_name: &str) -> Result<()> {
    let mut activity = read_metric_file(self_dir, "ring_activity");
    let total = activity
        .get("total_rings")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    activity["total_rings"] = serde_json::json!(total);

    if activity.get("rings").is_none() {
        activity["rings"] = serde_json::json!([]);
    }

    let exists = activity["rings"]
        .as_array()
        .map(|rings| {
            rings
                .iter()
                .any(|r| r.get("id").and_then(|v| v.as_str()) == Some(ring_id))
        })
        .unwrap_or(false);

    if !exists {
        if let Some(rings) = activity["rings"].as_array_mut() {
            rings.push(serde_json::json!({
                "id": ring_id,
                "name": ring_name,
                "joined_at": chrono::Local::now().to_rfc3339(),
            }));
        }
    }

    write_metric_file(self_dir, "ring_activity", &activity)
}

pub fn record_tool_usage(self_dir: &Path, tool_name: &str) -> Result<()> {
    let mut usage = read_metric_file(self_dir, "tool_usage");
    if usage.is_null() {
        usage = serde_json::json!({});
    }
    let Some(tools) = usage.as_object_mut() else {
        return Ok(());
    };
    let count = tools
        .get("tools")
        .and_then(|t| t.get(tool_name))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    if tools.get("tools").is_none() {
        tools.insert("tools".into(), serde_json::json!({}));
    }
    tools["tools"][tool_name] = serde_json::json!(count);
    if tools.get("last_used").is_none() {
        tools.insert("last_used".into(), serde_json::json!({}));
    }
    let now = chrono::Local::now().to_rfc3339();
    tools["last_used"][tool_name] = serde_json::json!(now);
    write_metric_file(
        self_dir,
        "tool_usage",
        &serde_json::Value::Object(tools.clone()),
    )
}

pub fn record_dwell_heartbeat(self_dir: &Path, view: &str, duration_s: u64) -> Result<()> {
    let mut dwell = read_metric_file(self_dir, "dwell_time");
    if dwell.is_null() {
        dwell = serde_json::json!({});
    }
    let Some(obj) = dwell.as_object_mut() else {
        return Ok(());
    };

    let views_total = obj
        .get("views")
        .and_then(|v| v.get(view))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + duration_s as i64;
    if obj.get("views").is_none() {
        obj.insert("views".into(), serde_json::json!({}));
    }
    obj["views"][view] = serde_json::json!(views_total);

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if obj.get("daily").is_none() {
        obj.insert("daily".into(), serde_json::json!({}));
    }
    if obj["daily"].get(&today).is_none() {
        obj["daily"][&today] = serde_json::json!({});
    }
    let daily_total = obj["daily"]
        .get(&today)
        .and_then(|d| d.get(view))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + duration_s as i64;
    obj["daily"][&today][view] = serde_json::json!(daily_total);

    obj.insert(
        "last_heartbeat".into(),
        serde_json::json!(chrono::Local::now().to_rfc3339()),
    );

    write_metric_file(
        self_dir,
        "dwell_time",
        &serde_json::Value::Object(obj.clone()),
    )
}

pub fn flush_dwell_buffer(
    self_dir: &Path,
    buffer: &std::collections::HashMap<String, u64>,
) -> Result<()> {
    for (view, seconds) in buffer {
        record_dwell_heartbeat(self_dir, view, *seconds)?;
    }
    Ok(())
}

pub fn get_self_dir(user_id: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base = std::path::PathBuf::from(format!("{home}/.ring/self"));
    // Migrate from legacy shared directory
    migrate_legacy_self_dir(&base, user_id);
    base.join(user_id)
}

fn migrate_legacy_self_dir(base: &std::path::Path, user_id: &str) {
    let user_dir = base.join(user_id);
    if user_dir.exists() {
        return;
    }
    if !base.exists() {
        return;
    }
    // Legacy directory exists but user-specific dir doesn't — migrate
    let _ = std::fs::create_dir_all(&user_dir);
    for entry in std::fs::read_dir(base)
        .unwrap_or_else(|_| std::fs::read_dir("/dev/null").unwrap())
        .flatten()
    {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default();
        if name == user_id {
            continue;
        }
        let dest = user_dir.join(name);
        let _ = std::fs::rename(&path, &dest);
    }
}

pub fn reset_all_data(self_dir: &Path) -> Result<()> {
    let _ = std::fs::remove_dir_all(self_dir.join(SELF_DIR));
    let _ = std::fs::remove_dir_all(self_dir.join(METRICS_DIR));
    Ok(())
}

pub fn export_all_data(self_dir: &Path) -> Result<serde_json::Value> {
    let mut result = serde_json::Map::new();

    for name in &["identity", "style", "personality", "privacy", "growth"] {
        let (content, exists) = read_self_file(self_dir, name)?;
        result.insert(
            (*name).to_string(),
            serde_json::json!({
                "content": content,
                "exists": exists,
            }),
        );
    }

    result.insert("metrics".to_string(), read_metrics(self_dir));

    Ok(serde_json::Value::Object(result))
}

pub struct GreetingContext {
    pub date: String,
    pub user_profile: String,
    pub active_goals: String,
    pub most_active_ring: String,
}

pub fn build_greeting_context(self_dir: &Path, metrics: &serde_json::Value) -> GreetingContext {
    let date = chrono::Local::now().format("%Y年%m月%d日").to_string();
    let (user_profile, _) =
        crate::services::self_memory::read_memory_file_sync(self_dir, "user_profile")
            .unwrap_or_default();
    let (active_goals, _) =
        crate::services::self_memory::read_memory_file_sync(self_dir, "active_goals")
            .unwrap_or_default();
    let chat_patterns = metrics.get("chat_patterns");
    let mut ring_entries: Vec<(String, i64)> = Vec::new();
    if let Some(Some(obj)) = chat_patterns.map(|v| v.as_object()) {
        for (key, val) in obj {
            if let Some(ring_id) = key.strip_prefix("ring_") {
                if let Some(count) = val.as_i64() {
                    ring_entries.push((ring_id.to_string(), count));
                }
            }
        }
    }
    let most_active_ring = if !ring_entries.is_empty() {
        ring_entries.sort_by(|a, b| b.1.cmp(&a.1));
        ring_entries[0].0.clone()
    } else {
        String::new()
    };
    GreetingContext {
        date,
        user_profile,
        active_goals,
        most_active_ring,
    }
}

pub fn check_first_today(self_dir: &Path) -> bool {
    let metrics = read_metrics(self_dir);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let daily_key = format!("daily_{}", today);
    if let Some(cp) = metrics.get("chat_patterns") {
        if cp.get(&daily_key).and_then(|v| v.as_i64()).unwrap_or(0) > 0 {
            return false;
        }
    }
    true
}
