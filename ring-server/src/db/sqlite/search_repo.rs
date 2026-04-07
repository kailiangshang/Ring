use crate::error::{Result, RingError};
use crate::models::graph_model::SearchResult;

#[derive(sqlx::FromRow)]
pub(crate) struct SearchResultRow {
    pub node_id: String,
    pub graph_id: String,
    pub label: String,
    pub snippet: String,
    pub rank: f64,
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn index_node_search_inner(
        &self,
        node_id: &str,
        graph_id: &str,
        label: &str,
        content: &str,
    ) -> Result<()> {
        let jieba = self.get_jieba();
        let tok_label = jieba.cut(label, true).join(" ");
        let tok_content = jieba.cut(content, true).join(" ");
        sqlx::query(
            "INSERT INTO nodes_search(node_id, graph_id, label, content) VALUES(?, ?, ?, ?)",
        )
        .bind(node_id)
        .bind(graph_id)
        .bind(&tok_label)
        .bind(&tok_content)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn delete_node_search_inner(&self, node_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM nodes_search WHERE node_id = ?")
            .bind(node_id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn search_nodes_fts_inner(
        &self,
        query: &str,
        graph_ids: Option<Vec<String>>,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        let jieba = self.get_jieba();
        let tok_query = jieba.cut(query, true).join(" ");
        let match_expr = tok_query
            .split_whitespace()
            .map(|w| format!("\"{}\"", w))
            .collect::<Vec<_>>()
            .join(" OR ");

        if match_expr.is_empty() {
            return Ok(vec![]);
        }

        let results = if let Some(ref gids) = graph_ids {
            let placeholders = gids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT node_id, graph_id, label, snippet(nodes_search, 1, '<mark>', '</mark>', '...', 32) as snippet, rank FROM nodes_search WHERE (label MATCH ? OR content MATCH ?) AND graph_id IN ({}) ORDER BY rank LIMIT ?",
                placeholders
            );
            let mut q = sqlx::query_as::<_, SearchResultRow>(&sql)
                .bind(&match_expr)
                .bind(&match_expr);
            for gid in gids {
                q = q.bind(gid);
            }
            q.bind(limit)
                .fetch_all(self.pool())
                .await
                .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SearchResultRow>(
                "SELECT node_id, graph_id, label, snippet(nodes_search, 1, '<mark>', '</mark>', '...', 32) as snippet, rank FROM nodes_search WHERE label MATCH ? OR content MATCH ? ORDER BY rank LIMIT ?",
            )
            .bind(&match_expr)
            .bind(&match_expr)
            .bind(limit)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        };

        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                node_id: r.node_id,
                graph_id: r.graph_id,
                label: r.label,
                snippet: r.snippet,
                rank: r.rank,
            })
            .collect())
    }
}
