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
    ] {
        let path = self_dir.join(METRICS_DIR).join(format!("{name}.json"));
        let val = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());
        result.insert((*name).to_string(), val.unwrap_or(serde_json::Value::Null));
    }
    serde_json::Value::Object(result)
}
