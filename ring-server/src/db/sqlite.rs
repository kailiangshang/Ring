use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::blueprint::BlueprintTemplate;
use crate::models::conversation::{Conversation, Message};
use crate::models::graph_model::SearchResult;
use crate::models::invite::InviteToken;
use crate::models::ring::{NewRing, Ring};
use crate::models::user::{NewUser, User};
use sqlx::SqlitePool;

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        SqliteRepository { pool }
    }
}

#[async_trait::async_trait]
impl Repository for SqliteRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, display_name, setup_completed, created_at) VALUES (?, ?, FALSE, ?)",
        )
        .bind(&id)
        .bind(&new_user.display_name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(User {
            id,
            display_name: new_user.display_name,
            avatar_url: None,
            ip_address: None,
            setup_completed: false,
            created_at: now,
        })
    }

    async fn get_user(&self, id: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>("SELECT id, display_name, avatar_url, ip_address, setup_completed, created_at FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;

        Ok(row.map(|r| User {
            id: r.id,
            display_name: r.display_name,
            avatar_url: r.avatar_url,
            ip_address: r.ip_address,
            setup_completed: r.setup_completed,
            created_at: r.created_at,
        }))
    }

    async fn list_all_users(&self) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, display_name, avatar_url, ip_address, setup_completed, created_at FROM users",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.id,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                ip_address: r.ip_address,
                setup_completed: r.setup_completed,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn is_setup_completed(&self) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as("SELECT setup_completed FROM users LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;

        match row {
            Some((setup_completed,)) => Ok(setup_completed),
            None => Ok(false),
        }
    }

    async fn complete_setup(&self, user_id: &str) -> Result<()> {
        sqlx::query("UPDATE users SET setup_completed = TRUE WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_ring(&self, new_ring: NewRing) -> Result<Ring> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let local_path = format!(".ring/repos/ring-{}", new_ring.name);

        sqlx::query(
            "INSERT INTO rings (id, name, description, creator_id, gitlab_repo, local_path, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(&new_ring.name)
        .bind(&new_ring.description)
        .bind(&new_ring.creator_id)
        .bind(&new_ring.gitlab_repo)
        .bind(&local_path)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Ring {
            id,
            name: new_ring.name,
            description: new_ring.description,
            creator_id: new_ring.creator_id,
            gitlab_repo: new_ring.gitlab_repo,
            local_path,
            next_token_id: 2,
            status: "active".into(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn get_ring(&self, id: &str) -> Result<Option<Ring>> {
        let row = sqlx::query_as::<_, RingRow>(
            "SELECT id, name, description, creator_id, gitlab_repo, local_path, next_token_id, status, created_at, updated_at FROM rings WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Ring {
            id: r.id,
            name: r.name,
            description: r.description,
            creator_id: r.creator_id,
            gitlab_repo: r.gitlab_repo,
            local_path: r.local_path,
            next_token_id: r.next_token_id,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list_rings_by_user(&self, user_id: &str) -> Result<Vec<Ring>> {
        let rows = sqlx::query_as::<_, RingRow>(
            "SELECT id, name, description, creator_id, gitlab_repo, local_path, next_token_id, status, created_at, updated_at FROM rings WHERE creator_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| Ring {
                id: r.id,
                name: r.name,
                description: r.description,
                creator_id: r.creator_id,
                gitlab_repo: r.gitlab_repo,
                local_path: r.local_path,
                next_token_id: r.next_token_id,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn update_ring(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Ring> {
        let existing = self
            .get_ring(id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", id)))?;

        let new_name = name.unwrap_or(existing.name);
        let new_description = description.or(existing.description);
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE rings SET name = ?, description = ?, updated_at = ? WHERE id = ?")
            .bind(&new_name)
            .bind(&new_description)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;

        Ok(Ring {
            name: new_name,
            description: new_description,
            updated_at: now,
            ..existing
        })
    }

    async fn delete_ring(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM rings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RingError::NotFound(format!("ring {}", id)));
        }
        Ok(())
    }

    async fn create_invite_token(
        &self,
        ring_id: &str,
        token: &str,
        token_type: &str,
        inviter_id: &str,
    ) -> Result<InviteToken> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        sqlx::query(
            "INSERT INTO invite_tokens (id, ring_id, token, token_type, inviter_id, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(token)
        .bind(token_type)
        .bind(inviter_id)
        .bind(expires_at.to_rfc3339())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(InviteToken {
            id,
            ring_id: ring_id.to_string(),
            token: token.to_string(),
            token_type: token_type.to_string(),
            role: "member".into(),
            inviter_id: inviter_id.to_string(),
            max_uses: 1,
            use_count: 0,
            max_members: None,
            expires_at: expires_at.to_rfc3339(),
            used_at: None,
            revoked_at: None,
            created_at: now,
        })
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(row.map(|(v,)| v))
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    async fn count_members_by_ring(&self, ring_id: &str) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM members WHERE ring_id = ?")
            .bind(ring_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(row.map(|(c,)| c).unwrap_or(0))
    }

    async fn get_invite_token(&self, token: &str) -> Result<Option<InviteToken>> {
        let row = sqlx::query_as::<_, InviteTokenRow>(
            "SELECT id, ring_id, token, token_type, role, inviter_id, max_uses, use_count, max_members, expires_at, used_at, revoked_at, created_at FROM invite_tokens WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| InviteToken {
            id: r.id,
            ring_id: r.ring_id,
            token: r.token,
            token_type: r.token_type,
            role: r.role,
            inviter_id: r.inviter_id,
            max_uses: r.max_uses,
            use_count: r.use_count,
            max_members: r.max_members,
            expires_at: r.expires_at,
            used_at: r.used_at,
            revoked_at: r.revoked_at,
            created_at: r.created_at,
        }))
    }

    async fn create_conversation(
        &self,
        ring_id: &str,
        title: Option<String>,
        context_mode: &str,
        created_by: &str,
    ) -> Result<Conversation> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO conversations (id, ring_id, title, mode, context_mode, created_by, created_at, updated_at) VALUES (?, ?, ?, 'chat', ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(&title)
        .bind(context_mode)
        .bind(created_by)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Conversation {
            id,
            ring_id: ring_id.to_string(),
            title,
            mode: "chat".into(),
            context_mode: context_mode.to_string(),
            token_count: 0,
            token_limit: 100000,
            auto_compact: false,
            summary: None,
            compacted_at: None,
            created_by: created_by.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn list_conversations(&self, ring_id: &str) -> Result<Vec<Conversation>> {
        let rows = sqlx::query_as::<_, ConversationRow>(
            "SELECT id, ring_id, title, mode, context_mode, token_count, token_limit, auto_compact, summary, compacted_at, created_by, created_at, updated_at FROM conversations WHERE ring_id = ? ORDER BY created_at DESC",
        )
        .bind(ring_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows.into_iter().map(|r| r.into_model()).collect())
    }

    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        let row = sqlx::query_as::<_, ConversationRow>(
            "SELECT id, ring_id, title, mode, context_mode, token_count, token_limit, auto_compact, summary, compacted_at, created_by, created_at, updated_at FROM conversations WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| r.into_model()))
    }

    async fn create_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        sender_id: Option<&str>,
    ) -> Result<Message> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, sender_id, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(sender_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Message {
            id,
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            sender_id: sender_id.map(|s| s.to_string()),
            tool_calls: None,
            archived: false,
            created_at: now,
        })
    }

    async fn get_messages(
        &self,
        conversation_id: &str,
        limit: i64,
        before_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let rows = match before_id {
            Some(bid) => {
                let before_created = sqlx::query_as::<_, (String,)>(
                    "SELECT created_at FROM messages WHERE id = ?",
                )
                .bind(bid)
                .fetch_optional(&self.pool)
                .await
                .map_err(RingError::Database)?;

                match before_created {
                    Some((ts,)) => {
                        sqlx::query_as::<_, MessageRow>(
                            "SELECT id, conversation_id, role, content, sender_id, tool_calls, archived, created_at FROM messages WHERE conversation_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?",
                        )
                        .bind(conversation_id)
                        .bind(&ts)
                        .bind(limit)
                        .fetch_all(&self.pool)
                        .await
                        .map_err(RingError::Database)?
                    }
                    None => vec![],
                }
            }
            None => {
                sqlx::query_as::<_, MessageRow>(
                    "SELECT id, conversation_id, role, content, sender_id, tool_calls, archived, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ?",
                )
                .bind(conversation_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(RingError::Database)?
            }
        };

        Ok(rows.into_iter().map(|r| r.into_model()).collect())
    }

    async fn update_ring_status(&self, id: &str, status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query("UPDATE rings SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RingError::NotFound(format!("ring {}", id)));
        }
        Ok(())
    }

    async fn list_blueprint_templates(&self) -> Result<Vec<BlueprintTemplate>> {
        let rows = sqlx::query_as::<_, BlueprintTemplateRow>(
            "SELECT id, name, description, graphs, is_system, created_by, created_at FROM blueprint_templates ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows.into_iter().map(|r| r.into_model()).collect())
    }

    async fn create_blueprint_template(
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
        .execute(&self.pool)
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

    async fn index_node_search(
        &self,
        node_id: &str,
        graph_id: &str,
        label: &str,
        content: &str,
    ) -> Result<()> {
        let jieba = jieba_rs::Jieba::new();
        let tok_label = jieba.cut(label, true).join(" ");
        let tok_content = jieba.cut(content, true).join(" ");
        sqlx::query(
            "INSERT INTO nodes_search(node_id, graph_id, label, content) VALUES(?, ?, ?, ?)",
        )
        .bind(node_id)
        .bind(graph_id)
        .bind(&tok_label)
        .bind(&tok_content)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    async fn delete_node_search(&self, node_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM nodes_search WHERE node_id = ?")
            .bind(node_id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn search_nodes_fts(
        &self,
        query: &str,
        graph_ids: Option<Vec<String>>,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        let jieba = jieba_rs::Jieba::new();
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
                .fetch_all(&self.pool)
                .await
                .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SearchResultRow>(
                "SELECT node_id, graph_id, label, snippet(nodes_search, 1, '<mark>', '</mark>', '...', 32) as snippet, rank FROM nodes_search WHERE label MATCH ? OR content MATCH ? ORDER BY rank LIMIT ?",
            )
            .bind(&match_expr)
            .bind(&match_expr)
            .bind(limit)
            .fetch_all(&self.pool)
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

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    display_name: String,
    avatar_url: Option<String>,
    ip_address: Option<String>,
    setup_completed: bool,
    created_at: String,
}

#[derive(sqlx::FromRow)]
struct RingRow {
    id: String,
    name: String,
    description: Option<String>,
    creator_id: String,
    gitlab_repo: String,
    local_path: String,
    next_token_id: i64,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct InviteTokenRow {
    id: String,
    ring_id: String,
    token: String,
    token_type: String,
    role: String,
    inviter_id: String,
    max_uses: i64,
    use_count: i64,
    max_members: Option<i64>,
    expires_at: String,
    used_at: Option<String>,
    revoked_at: Option<String>,
    created_at: String,
}

#[derive(sqlx::FromRow)]
struct ConversationRow {
    id: String,
    ring_id: String,
    title: Option<String>,
    mode: String,
    context_mode: String,
    token_count: i64,
    token_limit: i64,
    auto_compact: bool,
    summary: Option<String>,
    compacted_at: Option<String>,
    created_by: String,
    created_at: String,
    updated_at: String,
}

impl ConversationRow {
    fn into_model(self) -> Conversation {
        Conversation {
            id: self.id,
            ring_id: self.ring_id,
            title: self.title,
            mode: self.mode,
            context_mode: self.context_mode,
            token_count: self.token_count,
            token_limit: self.token_limit,
            auto_compact: self.auto_compact,
            summary: self.summary,
            compacted_at: self.compacted_at,
            created_by: self.created_by,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    sender_id: Option<String>,
    tool_calls: Option<String>,
    archived: bool,
    created_at: String,
}

impl MessageRow {
    fn into_model(self) -> Message {
        Message {
            id: self.id,
            conversation_id: self.conversation_id,
            role: self.role,
            content: self.content,
            sender_id: self.sender_id,
            tool_calls: self.tool_calls,
            archived: self.archived,
            created_at: self.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BlueprintTemplateRow {
    id: String,
    name: String,
    description: Option<String>,
    graphs: String,
    is_system: bool,
    created_by: Option<String>,
    created_at: String,
}

impl BlueprintTemplateRow {
    fn into_model(self) -> BlueprintTemplate {
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

#[derive(sqlx::FromRow)]
struct SearchResultRow {
    node_id: String,
    graph_id: String,
    label: String,
    snippet: String,
    rank: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ring::NewRing;
    use crate::models::user::NewUser;

    async fn setup_test_db() -> SqliteRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        SqliteRepository::new(pool)
    }

    #[tokio::test]
    async fn create_and_get_user() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        assert_eq!(user.display_name, "张三");
        assert!(!user.id.is_empty());

        let fetched = repo.get_user(&user.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, user.id);
    }

    #[tokio::test]
    async fn create_and_list_rings() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();

        let ring = repo
            .create_ring(NewRing {
                name: "竞品分析".into(),
                description: Some("desc".into()),
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "产品专家".into(),
            })
            .await
            .unwrap();
        assert_eq!(ring.name, "竞品分析");

        let rings = repo.list_rings_by_user(&user.id).await.unwrap();
        assert_eq!(rings.len(), 1);
    }

    #[tokio::test]
    async fn setup_status_defaults_to_false() {
        let repo = setup_test_db().await;
        let status = repo.is_setup_completed().await.unwrap();
        assert!(!status);
    }

    #[tokio::test]
    async fn complete_setup_sets_flag() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        repo.complete_setup(&user.id).await.unwrap();
        let status = repo.is_setup_completed().await.unwrap();
        assert!(status);
    }

    #[tokio::test]
    async fn create_and_get_invite_token() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "竞品分析".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();

        let invite = repo
            .create_invite_token(&ring.id, "test-token-123", "open", &user.id)
            .await
            .unwrap();
        assert_eq!(invite.token, "test-token-123");
        assert_eq!(invite.token_type, "open");

        let fetched = repo
            .get_invite_token("test-token-123")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, invite.id);
    }

    #[tokio::test]
    async fn delete_ring_removes_from_list() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "竞品分析".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();

        repo.delete_ring(&ring.id).await.unwrap();

        let fetched = repo.get_ring(&ring.id).await.unwrap();
        assert!(fetched.is_none());

        let rings = repo.list_rings_by_user(&user.id).await.unwrap();
        assert!(rings.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_conversations() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "test-ring".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();

        let conv = repo
            .create_conversation(&ring.id, Some("my chat".into()), "storage", &user.id)
            .await
            .unwrap();
        assert_eq!(conv.title, Some("my chat".into()));
        assert_eq!(conv.context_mode, "storage");
        assert_eq!(conv.mode, "chat");

        let fetched = repo.get_conversation(&conv.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, conv.id);

        let list = repo.list_conversations(&ring.id).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn create_and_get_messages() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "test-ring".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();
        let conv = repo
            .create_conversation(&ring.id, None, "storage", &user.id)
            .await
            .unwrap();

        let msg = repo
            .create_message(&conv.id, "user", "hello", Some(&user.id))
            .await
            .unwrap();
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.sender_id, Some(user.id.clone()));

        let msgs = repo.get_messages(&conv.id, 50, None).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "hello");
    }

    #[tokio::test]
    async fn get_messages_with_limit() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "test-ring".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();
        let conv = repo
            .create_conversation(&ring.id, None, "storage", &user.id)
            .await
            .unwrap();

        for i in 0..5 {
            repo.create_message(&conv.id, "user", &format!("msg {}", i), Some(&user.id))
                .await
                .unwrap();
        }

        let msgs = repo.get_messages(&conv.id, 3, None).await.unwrap();
        assert_eq!(msgs.len(), 3);

        let msgs_before = repo
            .get_messages(&conv.id, 10, Some(&msgs[2].id))
            .await
            .unwrap();
        assert_eq!(msgs_before.len(), 2);
    }

    #[tokio::test]
    async fn list_blueprint_templates_empty() {
        let repo = setup_test_db().await;
        let templates = repo.list_blueprint_templates().await.unwrap();
        assert!(templates.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_blueprint_templates() {
        let repo = setup_test_db().await;

        let bt = repo
            .create_blueprint_template(
                "bp-1",
                "knowledge-graph",
                Some("standard knowledge graph"),
                r#"[{"name":"concepts","graph_type":"knowledge"}]"#,
                true,
            )
            .await
            .unwrap();
        assert_eq!(bt.name, "knowledge-graph");
        assert!(bt.is_system);

        let templates = repo.list_blueprint_templates().await.unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id, "bp-1");
    }
}
