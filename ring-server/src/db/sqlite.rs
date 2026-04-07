use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::blueprint::BlueprintTemplate;
use crate::models::conversation::{Conversation, Message};
use crate::models::git_model::ArchiveRecord;
use crate::models::graph_model::SearchResult;
use crate::models::invite::InviteToken;
use crate::models::member::{Member, NewMember};
use crate::models::notification_model::{NewNotification, Notification};
use crate::models::ring::{NewRing, Ring};
use crate::models::session_model::{Session, SessionMember, SessionMessage};
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
            "SELECT DISTINCT r.id, r.name, r.description, r.creator_id, r.gitlab_repo, r.local_path, r.next_token_id, r.status, r.created_at, r.updated_at \
             FROM rings r \
             LEFT JOIN members m ON m.ring_id = r.id \
             WHERE r.creator_id = ? OR m.user_id = ?",
        )
        .bind(user_id)
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

    async fn create_archive_record(
        &self,
        id: &str,
        ring_id: &str,
        node_id: Option<&str>,
        conversation_id: Option<&str>,
        message_ids: &str,
        markdown_path: &str,
        archived_by: &str,
        git_commit_sha: Option<&str>,
        pr_status: Option<&str>,
        pr_url: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO archive_records (id, ring_id, node_id, conversation_id, message_ids, markdown_path, archived_by, git_commit_sha, pr_status, pr_url) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(ring_id)
        .bind(node_id)
        .bind(conversation_id)
        .bind(message_ids)
        .bind(markdown_path)
        .bind(archived_by)
        .bind(git_commit_sha)
        .bind(pr_status)
        .bind(pr_url)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    async fn list_archive_records_by_ring(&self, ring_id: &str) -> Result<Vec<ArchiveRecord>> {
        let rows = sqlx::query_as::<_, ArchiveRecord>(
            "SELECT id, ring_id, node_id, conversation_id, message_ids, markdown_path, archived_by, git_commit_sha, pr_status, pr_url, created_at FROM archive_records WHERE ring_id = ? ORDER BY created_at DESC",
        )
        .bind(ring_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(rows)
    }

    async fn get_archive_record(&self, id: &str) -> Result<Option<ArchiveRecord>> {
        let row = sqlx::query_as::<_, ArchiveRecord>(
            "SELECT id, ring_id, node_id, conversation_id, message_ids, markdown_path, archived_by, git_commit_sha, pr_status, pr_url, created_at FROM archive_records WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(row)
    }

    async fn update_archive_pr_status(&self, id: &str, pr_status: &str) -> Result<()> {
        sqlx::query("UPDATE archive_records SET pr_status = ? WHERE id = ?")
            .bind(pr_status)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_member(&self, new_member: NewMember) -> Result<Member> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let token_id: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(token_id), 0) + 1 FROM members WHERE ring_id = ?")
                .bind(&new_member.ring_id)
                .fetch_one(&self.pool)
                .await
                .map_err(RingError::Database)?;

        sqlx::query(
            "INSERT INTO members (id, ring_id, user_id, token_id, display_name, role, joined_at) VALUES (?, ?, ?, ?, ?, COALESCE(?, 'member'), ?)",
        )
        .bind(&id)
        .bind(&new_member.ring_id)
        .bind(&new_member.user_id)
        .bind(token_id.0)
        .bind(&new_member.display_name)
        .bind(&new_member.role)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Member {
            id,
            ring_id: new_member.ring_id,
            user_id: new_member.user_id,
            token_id: token_id.0,
            display_name: new_member.display_name,
            role: new_member.role.unwrap_or_else(|| "member".into()),
            joined_at: now,
        })
    }

    async fn get_member(&self, id: &str) -> Result<Option<Member>> {
        let row = sqlx::query_as::<_, MemberRow>(
            "SELECT id, ring_id, user_id, token_id, display_name, role, joined_at FROM members WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Member {
            id: r.id,
            ring_id: r.ring_id,
            user_id: r.user_id,
            token_id: r.token_id,
            display_name: r.display_name,
            role: r.role,
            joined_at: r.joined_at,
        }))
    }

    async fn list_members_by_ring(&self, ring_id: &str) -> Result<Vec<Member>> {
        let rows = sqlx::query_as::<_, MemberRow>(
            "SELECT id, ring_id, user_id, token_id, display_name, role, joined_at FROM members WHERE ring_id = ? ORDER BY joined_at",
        )
        .bind(ring_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| Member {
                id: r.id,
                ring_id: r.ring_id,
                user_id: r.user_id,
                token_id: r.token_id,
                display_name: r.display_name,
                role: r.role,
                joined_at: r.joined_at,
            })
            .collect())
    }

    async fn get_member_by_user_and_ring(
        &self,
        user_id: &str,
        ring_id: &str,
    ) -> Result<Option<Member>> {
        let row = sqlx::query_as::<_, MemberRow>(
            "SELECT id, ring_id, user_id, token_id, display_name, role, joined_at FROM members WHERE user_id = ? AND ring_id = ?",
        )
        .bind(user_id)
        .bind(ring_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Member {
            id: r.id,
            ring_id: r.ring_id,
            user_id: r.user_id,
            token_id: r.token_id,
            display_name: r.display_name,
            role: r.role,
            joined_at: r.joined_at,
        }))
    }

    async fn update_member_role(&self, id: &str, role: &str) -> Result<()> {
        sqlx::query("UPDATE members SET role = ? WHERE id = ?")
            .bind(role)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn delete_member(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM members WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn get_next_token_id(&self, ring_id: &str) -> Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(token_id), 0) + 1 FROM members WHERE ring_id = ?")
                .bind(ring_id)
                .fetch_one(&self.pool)
                .await
                .map_err(RingError::Database)?;
        Ok(row.0)
    }

    async fn create_notification(&self, n: NewNotification) -> Result<Notification> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO notifications (id, ring_id, user_id, type, title, body, related_id, is_read, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, FALSE, ?)",
        )
        .bind(&id)
        .bind(&n.ring_id)
        .bind(&n.user_id)
        .bind(&n.n_type)
        .bind(&n.title)
        .bind(&n.body)
        .bind(&n.related_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Notification {
            id,
            ring_id: n.ring_id,
            user_id: n.user_id,
            r#type: n.n_type,
            title: n.title,
            body: n.body,
            related_id: n.related_id,
            is_read: false,
            created_at: now,
        })
    }

    async fn list_notifications_by_user(
        &self,
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>> {
        let rows = if unread_only {
            sqlx::query_as::<_, NotificationRow>(
                "SELECT id, ring_id, user_id, type, title, body, related_id, is_read, created_at FROM notifications WHERE user_id = ? AND is_read = FALSE ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, NotificationRow>(
                "SELECT id, ring_id, user_id, type, title, body, related_id, is_read, created_at FROM notifications WHERE user_id = ? ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| Notification {
                id: r.id,
                ring_id: r.ring_id,
                user_id: r.user_id,
                r#type: r.type_field,
                title: r.title,
                body: r.body,
                related_id: r.related_id,
                is_read: r.is_read,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn mark_notification_read(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE notifications SET is_read = TRUE WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_session(
        &self,
        ring_id: &str,
        title: Option<&str>,
        scenario: &str,
        created_by: &str,
        archive_enabled: bool,
    ) -> Result<Session> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(title)
        .bind(scenario)
        .bind(created_by)
        .bind(archive_enabled)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Session {
            id,
            ring_id: ring_id.to_string(),
            title: title.map(|s| s.to_string()),
            scenario: scenario.to_string(),
            created_by: created_by.to_string(),
            archive_enabled,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Session {
            id: r.id,
            ring_id: r.ring_id,
            title: r.title,
            scenario: r.scenario,
            created_by: r.created_by,
            archive_enabled: r.archive_enabled,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list_sessions_by_ring(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Session>> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE ring_id = ? AND status = ? ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .bind(s)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE ring_id = ? ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| Session {
                id: r.id,
                ring_id: r.ring_id,
                title: r.title,
                scenario: r.scenario,
                created_by: r.created_by,
                archive_enabled: r.archive_enabled,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn update_session_status(&self, id: &str, status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sessions SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn update_session_archive(&self, id: &str, enabled: bool) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sessions SET archive_enabled = ?, updated_at = ? WHERE id = ?")
            .bind(enabled)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn delete_session(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM session_messages WHERE session_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        sqlx::query("DELETE FROM session_members WHERE session_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_session_member(
        &self,
        session_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<SessionMember> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO session_members (id, session_id, user_id, role, status, joined_at) VALUES (?, ?, ?, ?, 'active', ?)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(user_id)
        .bind(role)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(SessionMember {
            id,
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            role: role.to_string(),
            status: "active".to_string(),
            joined_at: now,
            left_at: None,
        })
    }

    async fn list_session_members(&self, session_id: &str) -> Result<Vec<SessionMember>> {
        let rows = sqlx::query_as::<_, SessionMemberRow>(
            "SELECT id, session_id, user_id, role, status, joined_at, left_at FROM session_members WHERE session_id = ? AND status = 'active'",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| SessionMember {
                id: r.id,
                session_id: r.session_id,
                user_id: r.user_id,
                role: r.role,
                status: r.status,
                joined_at: r.joined_at,
                left_at: r.left_at,
            })
            .collect())
    }

    async fn leave_session_member(&self, session_id: &str, user_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE session_members SET status = 'left', left_at = ? WHERE session_id = ? AND user_id = ?",
        )
        .bind(&now)
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_session_message(
        &self,
        session_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
        seq_num: i64,
    ) -> Result<SessionMessage> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO session_messages (id, session_id, sender_id, role, content, seq_num, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(sender_id)
        .bind(role)
        .bind(content)
        .bind(seq_num)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(SessionMessage {
            id,
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            seq_num,
            created_at: now,
        })
    }

    async fn get_session_messages(
        &self,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<SessionMessage>> {
        let rows = if let Some(seq) = after_seq {
            sqlx::query_as::<_, SessionMessageRow>(
                "SELECT id, session_id, sender_id, role, content, seq_num, created_at FROM session_messages WHERE session_id = ? AND seq_num > ? ORDER BY seq_num ASC LIMIT ?",
            )
            .bind(session_id)
            .bind(seq)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SessionMessageRow>(
                "SELECT id, session_id, sender_id, role, content, seq_num, created_at FROM session_messages WHERE session_id = ? ORDER BY seq_num ASC LIMIT ?",
            )
            .bind(session_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| SessionMessage {
                id: r.id,
                session_id: r.session_id,
                sender_id: r.sender_id,
                role: r.role,
                content: r.content,
                seq_num: r.seq_num,
                created_at: r.created_at,
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

#[derive(sqlx::FromRow)]
struct MemberRow {
    id: String,
    ring_id: String,
    user_id: String,
    token_id: i64,
    display_name: String,
    role: String,
    joined_at: String,
}

#[derive(sqlx::FromRow)]
struct NotificationRow {
    id: String,
    ring_id: String,
    user_id: String,
    #[sqlx(rename = "type")]
    type_field: String,
    title: String,
    body: Option<String>,
    related_id: Option<String>,
    is_read: bool,
    created_at: String,
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    ring_id: String,
    title: Option<String>,
    scenario: String,
    created_by: String,
    archive_enabled: bool,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct SessionMemberRow {
    id: String,
    session_id: String,
    user_id: String,
    role: String,
    status: String,
    joined_at: String,
    left_at: Option<String>,
}

#[derive(sqlx::FromRow)]
struct SessionMessageRow {
    id: String,
    session_id: String,
    sender_id: String,
    role: String,
    content: String,
    seq_num: i64,
    created_at: String,
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
