use lopdf::Document;

use crate::error::{Result, RingError};
use crate::models::message::{self, MessageRow};
use crate::models::session::{self, SessionMaterialRow};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const MAX_CONTENT_CHARS: usize = 50000;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "json", "py", "js", "ts", "tsx", "rs", "go", "java",
    "yaml", "yml", "xml", "html", "css", "toml", "sh", "sql", "log", "env",
    "conf", "cfg", "ini", "pdf",
];

pub fn validate_file(filename: &str, size: usize) -> Result<()> {
    if size > MAX_FILE_SIZE {
        return Err(RingError::BadRequest(format!(
            "file too large: {} bytes (max {})",
            size, MAX_FILE_SIZE
        )));
    }

    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(RingError::BadRequest(format!(
            "unsupported file type: .{ext}"
        )));
    }

    Ok(())
}

pub fn extract_text(filename: &str, data: &[u8]) -> Result<String> {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let text = if ext == "pdf" {
        extract_pdf_text(data)?
    } else {
        String::from_utf8_lossy(data).into_owned()
    };

    let truncated: String = text.chars().take(MAX_CONTENT_CHARS).collect();
    Ok(truncated)
}

fn extract_pdf_text(data: &[u8]) -> Result<String> {
    let doc = Document::load_mem(data)
        .map_err(|e| RingError::BadRequest(format!("failed to parse PDF: {e}")))?;

    let pages = doc.get_pages();
    let page_numbers: Vec<u32> = pages.keys().copied().collect();

    let text = doc
        .extract_text(&page_numbers)
        .map_err(|e| RingError::BadRequest(format!("failed to extract PDF text: {e}")))?;

    if text.trim().is_empty() {
        return Err(RingError::BadRequest(
            "PDF contains no extractable text (possibly scanned image)".into(),
        ));
    }

    Ok(text)
}

pub async fn upload_to_chat(
    db: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    sender_name: &str,
    filename: &str,
    data: &[u8],
) -> Result<MessageRow> {
    validate_file(filename, data.len())?;
    let content = extract_text(filename, data)?;

    let msg_id = ulid::Ulid::new().to_string();
    let file_content = format!("\u{1f4ce} {filename}\n---\n{content}");

    let msg = message::insert_message(
        db,
        &message::NewMessage {
            id: &msg_id,
            ring_id,
            user_id,
            role: "system",
            sender_name,
            content: &file_content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    if let Some(rid) = ring_id {
        let ring_name = crate::services::search::get_ring_name(db, rid)
            .await
            .unwrap_or_default();
        let _ = crate::services::search::upsert_search_index(
            db,
            "message",
            &msg_id,
            rid,
            &ring_name,
            &format!("\u{1f4ce} {filename}"),
            &content,
            &serde_json::json!({"role": "system", "filename": filename}).to_string(),
        )
        .await;
    }

    Ok(msg)
}

pub async fn upload_to_session(
    db: &sqlx::SqlitePool,
    ring_id: &str,
    session_id: &str,
    filename: &str,
    data: &[u8],
) -> Result<SessionMaterialRow> {
    validate_file(filename, data.len())?;
    let content = extract_text(filename, data)?;

    let material_id = ulid::Ulid::new().to_string();
    let material = session::create_material(
        db,
        &material_id,
        session_id,
        "document",
        filename,
        &content,
    )
    .await?;

    let _ = crate::services::search::upsert_search_index(
        db,
        "session_message",
        &material_id,
        ring_id,
        "",
        filename,
        &content,
        &serde_json::json!({"session_id": session_id, "item_type": "document"}).to_string(),
    )
    .await;

    Ok(material)
}
