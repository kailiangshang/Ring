use sqlx::{QueryBuilder, SqlitePool};

use crate::error::{Result, RingError};

#[derive(Debug, sqlx::FromRow)]
pub struct SearchRow {
    pub source_type: String,
    pub source_id: String,
    pub ring_id: String,
    pub ring_name: String,
    pub title: String,
    pub content: String,
    pub metadata: String,
    pub rank: f64,
}

pub async fn search_cross_ring(
    db: &SqlitePool,
    ring_ids: &[String],
    query: &str,
    limit: i64,
) -> Result<Vec<SearchRow>> {
    if ring_ids.is_empty() || query.trim().is_empty() {
        return Ok(vec![]);
    }

    let fts_query = sanitize_fts_query(query);
    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT source_type, source_id, ring_id, ring_name, title, content, metadata, rank \
         FROM search_index \
         WHERE search_index MATCH ",
    );
    builder.push_bind(&fts_query);
    builder.push(" AND ring_id IN (");
    let mut separated = builder.separated(",");
    for id in ring_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");
    builder.push(" ORDER BY bm25(search_index) LIMIT ");
    builder.push_bind(limit);

    builder
        .build_query_as::<SearchRow>()
        .fetch_all(db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_search_index(
    db: &SqlitePool,
    source_type: &str,
    source_id: &str,
    ring_id: &str,
    ring_name: &str,
    title: &str,
    content: &str,
    metadata: &str,
) -> Result<()> {
    delete_search_index(db, source_type, source_id).await.ok();

    sqlx::query(
        "INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(source_type)
    .bind(source_id)
    .bind(ring_id)
    .bind(ring_name)
    .bind(title)
    .bind(content)
    .bind(metadata)
    .execute(db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn delete_search_index(
    db: &SqlitePool,
    source_type: &str,
    source_id: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM search_index WHERE source_type = ?1 AND source_id = ?2")
        .bind(source_type)
        .bind(source_id)
        .execute(db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn delete_search_index_by_ring(db: &SqlitePool, ring_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM search_index WHERE ring_id = ?1")
        .bind(ring_id)
        .execute(db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

fn sanitize_fts_query(input: &str) -> String {
    let chars_to_strip = ['*', '"', '(', ')', ':', '^', '{', '}', '[', ']'];
    let cleaned: String = input
        .chars()
        .map(|c| if chars_to_strip.contains(&c) { ' ' } else { c })
        .collect();
    let terms: Vec<&str> = cleaned
        .split_whitespace()
        .filter(|t| t.len() >= 2)
        .take(10)
        .collect();
    if terms.is_empty() {
        return String::new();
    }
    terms
        .iter()
        .map(|t| format!("{t}*"))
        .collect::<Vec<_>>()
        .join(" OR ")
}

pub async fn get_user_ring_ids(db: &SqlitePool, user_id: &str) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT r.id FROM rings r JOIN members m ON r.id = m.ring_id WHERE m.user_id = ?1",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn get_ring_name(db: &SqlitePool, ring_id: &str) -> Result<String> {
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(ring_id)
        .fetch_optional(db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    Ok(name.unwrap_or_default())
}

pub fn format_search_context(results: &[SearchRow]) -> String {
    if results.is_empty() {
        return String::new();
    }
    let mut ctx =
        String::from("<cross_ring_context>\n以下是从用户的所有 Ring 中检索到的相关内容：\n\n");
    for r in results {
        let truncated: String = r.content.chars().take(500).collect();
        let ellipsis = if r.content.len() > 500 { "..." } else { "" };
        ctx.push_str(&format!(
            "[Ring: {} > {}]\ntype: {}, id: {}\n{}{}\n\n",
            r.ring_name, r.title, r.source_type, r.source_id, truncated, ellipsis
        ));
    }
    ctx.push_str("</cross_ring_context>");
    ctx
}
