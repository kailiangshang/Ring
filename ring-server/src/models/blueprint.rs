use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub graphs: String,
    pub is_system: bool,
    pub created_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBlueprintTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub graphs: String,
    pub is_system: bool,
}
