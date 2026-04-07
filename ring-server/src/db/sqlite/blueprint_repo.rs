use crate::error::{Result, RingError};
use crate::models::blueprint::BlueprintTemplate;

#[derive(sqlx::FromRow)]
pub(crate) struct BlueprintTemplateRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub graphs: String,
    pub is_system: bool,
    pub created_by: Option<String>,
    pub created_at: String,
}

impl BlueprintTemplateRow {
    pub fn into_model(self) -> BlueprintTemplate {
        BlueprintTemplate {
            id: self.id,
            name: self.name,
            description: self.description,
            graphs: self.graphs,
            is_system: self.is_system,
            created_by: self.created_by,
            created_at: self.created_at,
        }
    }
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn list_blueprint_templates_inner(&self) -> Result<Vec<BlueprintTemplate>> {
        let rows = sqlx::query_as::<_, BlueprintTemplateRow>(
            "SELECT id, name, description, graphs, is_system, created_by, created_at FROM blueprint_templates ORDER BY created_at ASC",
        )
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(rows.into_iter().map(|r| r.into_model()).collect())
    }

    pub async fn create_blueprint_template_inner(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        graphs_json: &str,
        is_system: bool,
    ) -> Result<BlueprintTemplate> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO blueprint_templates (id, name, description, graphs, is_system, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(graphs_json)
        .bind(is_system)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(BlueprintTemplate {
            id: id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            graphs: graphs_json.to_string(),
            is_system,
            created_by: None,
            created_at: now,
        })
    }
}
