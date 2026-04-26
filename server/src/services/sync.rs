use crate::error::{Result, RingError};
use crate::models::archive;
use crate::models::graph;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncBundle {
    pub version: String,
    pub ring_id: String,
    pub exported_at: String,
    pub graphs: Vec<GraphBundle>,
    pub archive_records: Vec<ArchiveRecordBundle>,
    pub group_docs: Vec<GroupDocBundle>,
    pub archive_files: Vec<ArchiveFileBundle>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphBundle {
    pub graph: graph::GraphRow,
    pub nodes: Vec<graph::GraphNodeRow>,
    pub edges: Vec<graph::GraphEdgeRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveRecordBundle {
    pub id: String,
    pub ring_id: String,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub file_name: String,
    pub commit_sha: Option<String>,
    pub status: String,
    pub archived_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupDocBundle {
    pub doc_name: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveFileBundle {
    pub file_name: String,
    pub content: String,
}

pub async fn export_bundle(state: &AppState, ring_id: &str) -> Result<SyncBundle> {
    let graphs = graph::list_graphs(&state.db, ring_id).await?;
    let mut graph_bundles = Vec::new();
    for g in &graphs {
        let nodes = graph::list_nodes(&state.db, &g.id).await?;
        let edges = graph::list_edges(&state.db, &g.id).await?;
        graph_bundles.push(GraphBundle {
            graph: g.clone(),
            nodes,
            edges,
        });
    }

    let records = archive::list_by_ring(&state.db, ring_id).await?;
    let record_bundles: Vec<ArchiveRecordBundle> = records
        .into_iter()
        .map(|r| ArchiveRecordBundle {
            id: r.id,
            ring_id: r.ring_id,
            session_id: r.session_id,
            node_id: r.node_id,
            file_name: r.file_name,
            commit_sha: r.commit_sha,
            status: r.status,
            archived_by: r.archived_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect();

    let group_rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT doc_name, content, COALESCE(updated_at, '') FROM group_docs WHERE ring_id = ?1",
    )
    .bind(ring_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;
    let doc_bundles: Vec<GroupDocBundle> = group_rows
        .into_iter()
        .map(|(doc_name, content, updated_at)| GroupDocBundle {
            doc_name,
            content,
            updated_at,
        })
        .collect();

    let repo_path = state.rings_dir.join(ring_id).join("archives");
    let mut archive_files = Vec::new();
    if repo_path.exists() {
        let entries =
            std::fs::read_dir(&repo_path).map_err(|e| RingError::Internal(e.to_string()))?;
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") {
                    let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    archive_files.push(ArchiveFileBundle {
                        file_name: name,
                        content,
                    });
                }
            }
        }
    }

    Ok(SyncBundle {
        version: "1.0".into(),
        ring_id: ring_id.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        graphs: graph_bundles,
        archive_records: record_bundles,
        group_docs: doc_bundles,
        archive_files,
    })
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub graphs: usize,
    pub nodes: usize,
    pub edges: usize,
    pub archive_records: usize,
    pub group_docs: usize,
    pub archive_files: usize,
}

pub async fn import_bundle(state: &AppState, bundle: &SyncBundle) -> Result<ImportResult> {
    let mut result = ImportResult {
        graphs: 0,
        nodes: 0,
        edges: 0,
        archive_records: 0,
        group_docs: 0,
        archive_files: 0,
    };

    for gb in &bundle.graphs {
        sqlx::query(
            "INSERT INTO graphs (id, ring_id, name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET name = ?3, updated_at = ?5",
        )
        .bind(&gb.graph.id)
        .bind(&gb.graph.ring_id)
        .bind(&gb.graph.name)
        .bind(&gb.graph.created_at)
        .bind(&gb.graph.updated_at)
        .execute(&state.db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
        result.graphs += 1;

        for node in &gb.nodes {
            sqlx::query(
                "INSERT INTO graph_nodes (id, graph_id, ring_id, label, parent_id, node_type, content, tags, markdown_path, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                    label = ?4, parent_id = ?5, node_type = ?6, content = ?7,
                    tags = ?8, markdown_path = ?9, metadata = ?10, updated_at = ?12",
            )
            .bind(&node.id)
            .bind(&node.graph_id)
            .bind(&node.ring_id)
            .bind(&node.label)
            .bind(&node.parent_id)
            .bind(&node.node_type)
            .bind(&node.content)
            .bind(&node.tags)
            .bind(&node.markdown_path)
            .bind(&node.metadata)
            .bind(&node.created_at)
            .bind(&node.updated_at)
            .execute(&state.db)
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;
            result.nodes += 1;
        }

        for edge in &gb.edges {
            sqlx::query(
                "INSERT INTO graph_edges (id, graph_id, ring_id, source_id, target_id, relation, label, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    source_id = ?4, target_id = ?5, relation = ?6, label = ?7",
            )
            .bind(&edge.id)
            .bind(&edge.graph_id)
            .bind(&edge.ring_id)
            .bind(&edge.source_id)
            .bind(&edge.target_id)
            .bind(&edge.relation)
            .bind(&edge.label)
            .bind(&edge.created_at)
            .execute(&state.db)
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;
            result.edges += 1;
        }
    }

    for rec in &bundle.archive_records {
        sqlx::query(
            "INSERT INTO archive_records (id, ring_id, session_id, node_id, file_name, commit_sha, status, archived_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                status = ?7, commit_sha = COALESCE(?6, commit_sha), updated_at = ?10",
        )
        .bind(&rec.id)
        .bind(&rec.ring_id)
        .bind(&rec.session_id)
        .bind(&rec.node_id)
        .bind(&rec.file_name)
        .bind(&rec.commit_sha)
        .bind(&rec.status)
        .bind(&rec.archived_by)
        .bind(&rec.created_at)
        .bind(&rec.updated_at)
        .execute(&state.db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
        result.archive_records += 1;
    }

    for doc in &bundle.group_docs {
        sqlx::query(
            "INSERT INTO group_docs (ring_id, doc_name, content, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(ring_id, doc_name) DO UPDATE SET content = ?3, updated_at = ?4",
        )
        .bind(&bundle.ring_id)
        .bind(&doc.doc_name)
        .bind(&doc.content)
        .bind(&doc.updated_at)
        .execute(&state.db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
        result.group_docs += 1;
    }

    let archives_dir = state.rings_dir.join(&bundle.ring_id).join("archives");
    std::fs::create_dir_all(&archives_dir).map_err(|e| RingError::Internal(e.to_string()))?;
    for file in &bundle.archive_files {
        let path = archives_dir.join(&file.file_name);
        std::fs::write(&path, &file.content).map_err(|e| RingError::Internal(e.to_string()))?;
        result.archive_files += 1;
    }

    sqlx::query(
        "INSERT INTO sync_meta (ring_id, key, value, updated_at)
         VALUES (?1, 'last_sync_at', ?2, datetime('now'))
         ON CONFLICT(ring_id, key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
    )
    .bind(&bundle.ring_id)
    .bind(&bundle.exported_at)
    .execute(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(result)
}
