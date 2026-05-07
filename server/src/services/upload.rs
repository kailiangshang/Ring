use crate::error::{Result, RingError};
use crate::models::message::{self, MessageRow};
use crate::models::session::{self, SessionMaterialRow};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const CHUNK_SIZE: usize = 20000;
const TOKEN_WARNING_THRESHOLD: usize = 10000;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "json", "py", "js", "ts", "tsx", "rs", "go", "java", "yaml", "yml", "xml",
    "html", "css", "toml", "sh", "sql", "log", "env", "conf", "cfg", "ini", "pdf",
];

pub fn validate_file(filename: &str, size: usize) -> Result<()> {
    if size > MAX_FILE_SIZE {
        return Err(RingError::BadRequest(format!(
            "file too large: {} bytes (max {})",
            size, MAX_FILE_SIZE
        )));
    }

    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(RingError::BadRequest(format!(
            "unsupported file type: .{ext}"
        )));
    }

    Ok(())
}

pub fn extract_text(filename: &str, data: &[u8]) -> Result<String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let text = if ext == "pdf" {
        extract_pdf_text(data)?
    } else {
        String::from_utf8_lossy(data).into_owned()
    };
    Ok(text)
}

fn extract_pdf_text(data: &[u8]) -> Result<String> {
    // Primary: pdf-extract supports Identity-H and other CMap encodings
    match pdf_extract::extract_text_from_mem(data) {
        Ok(text) => {
            if text.trim().is_empty() {
                return Err(RingError::BadRequest(
                    "PDF contains no extractable text (possibly scanned image)".into(),
                ));
            }
            Ok(text)
        }
        Err(e) => {
            let err_str = e.to_string();
            tracing::warn!("pdf-extract failed: {err_str}, trying pdftotext fallback");
            // Fallback: external pdftotext (poppler-utils)
            if let Ok(text) = extract_pdf_with_pdftotext(data) {
                return Ok(text);
            }
            Err(RingError::BadRequest(format!(
                "Failed to extract PDF text. Error: {err_str}. \
                 If this is a CJK PDF, try converting it to plain text first."
            )))
        }
    }
}

fn extract_pdf_with_pdftotext(data: &[u8]) -> Result<String> {
    use std::process::Command;

    let mut temp_pdf = std::env::temp_dir();
    temp_pdf.push(format!("ring_pdf_{}.pdf", std::process::id()));
    let mut temp_txt = temp_pdf.clone();
    temp_txt.set_extension("txt");

    std::fs::write(&temp_pdf, data)?;

    let output = Command::new("pdftotext")
        .arg(&temp_pdf)
        .arg(&temp_txt)
        .output()?;

    let _ = std::fs::remove_file(&temp_pdf);

    if output.status.success() {
        let text = std::fs::read_to_string(&temp_txt)?;
        let _ = std::fs::remove_file(&temp_txt);
        Ok(text)
    } else {
        let _ = std::fs::remove_file(&temp_txt);
        Err(RingError::BadRequest("pdftotext failed".into()))
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    let cjk = text
        .chars()
        .filter(|c| {
            (*c >= '\u{4E00}' && *c <= '\u{9FFF}')
                || (*c >= '\u{3040}' && *c <= '\u{309F}')
                || (*c >= '\u{30A0}' && *c <= '\u{30FF}')
                || (*c >= '\u{AC00}' && *c <= '\u{D7AF}')
        })
        .count();
    let other = text.chars().count().saturating_sub(cjk);
    (other / 4) + (cjk * 3 / 2)
}

pub fn split_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }

    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        let combined = if current.is_empty() {
            para.to_string()
        } else {
            format!("{current}\n\n{para}")
        };

        if combined.chars().count() > max_chars && !current.is_empty() {
            chunks.push(current);
            current = para.to_string();
        } else {
            current = combined;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

pub fn chunk_info(text: &str) -> (usize, usize) {
    let tokens = estimate_tokens(text);
    let count = split_into_chunks(text, CHUNK_SIZE).len();
    (tokens, count)
}

pub fn exceeds_token_warning(text: &str) -> bool {
    estimate_tokens(text) > TOKEN_WARNING_THRESHOLD
}

pub async fn upload_to_chat(
    db: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    sender_name: &str,
    filename: &str,
    data: &[u8],
) -> Result<Vec<MessageRow>> {
    validate_file(filename, data.len())?;
    let content = extract_text(filename, data)?;

    let chunks = split_into_chunks(&content, CHUNK_SIZE);
    let total_chunks = chunks.len();
    let mut messages = Vec::with_capacity(total_chunks);

    for (i, chunk) in chunks.iter().enumerate() {
        let msg_id = ulid::Ulid::new().to_string();
        let label = if total_chunks > 1 {
            format!(
                "\u{1f4ce} {filename} [{}/{}]\n---\n{chunk}",
                i + 1,
                total_chunks
            )
        } else {
            format!("\u{1f4ce} {filename}\n---\n{chunk}")
        };

        let msg = message::insert_message(
            db,
            &message::NewMessage {
                id: &msg_id,
                ring_id,
                user_id,
                role: "system",
                sender_name,
                content: &label,
                node_refs: &[],
                tag_refs: &[],
                token_usage: None,
            },
        )
        .await?;

        if i == 0 {
            if let Some(rid) = ring_id {
                let ring_name = crate::services::search::get_ring_name(db, rid)
                    .await
                    .unwrap_or_default();
                if let Err(e) = crate::services::search::upsert_search_index(
                    db,
                    "message",
                    &msg_id,
                    rid,
                    &ring_name,
                    &format!("\u{1f4ce} {filename}"),
                    &content,
                    &serde_json::json!({"role": "system", "filename": filename}).to_string(),
                )
                .await
                {
                    tracing::warn!("failed to update search index: {e}");
                }
            }
        }

        messages.push(msg);
    }

    Ok(messages)
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
    let material =
        session::create_material(db, &material_id, session_id, "document", filename, &content)
            .await?;

    if let Err(e) = crate::services::search::upsert_search_index(
        db,
        "session_message",
        &material_id,
        ring_id,
        "",
        filename,
        &content,
        &serde_json::json!({"session_id": session_id, "item_type": "document"}).to_string(),
    )
    .await
    {
        tracing::warn!("failed to update search index: {e}");
    }

    Ok(material)
}
