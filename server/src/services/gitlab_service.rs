use serde::{Deserialize, Serialize};

use crate::error::{Result, RingError};

#[derive(Clone)]
pub struct GitLabClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeRequest {
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub state: String,
    pub web_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffRef {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabUser {
    pub id: i64,
    pub username: String,
}

impl GitLabClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn project_id_from_url(repo_url: &str) -> String {
        let url = url::Url::parse(repo_url)
            .unwrap_or_else(|_| url::Url::parse(&format!("https://{}", repo_url)).unwrap());
        let path = url.path().trim_start_matches('/').trim_end_matches(".git");
        urlencoding::encode(path).to_string()
    }

    pub async fn get_current_user(&self) -> Result<GitLabUser> {
        let url = format!("{}/api/v4/user", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError { status, message: body });
        }

        resp.json::<GitLabUser>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn create_mr(
        &self,
        project_url: &str,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: &str,
    ) -> Result<MergeRequest> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests",
            self.base_url, project_id
        );

        let resp = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(&serde_json::json!({
                "source_branch": source_branch,
                "target_branch": target_branch,
                "title": title,
                "description": description,
            }))
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        let status = resp.status().as_u16();
        if status == 409 {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: format!("conflict: {body}"),
            });
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError { status, message: body });
        }

        resp.json::<MergeRequest>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn merge_mr(&self, project_url: &str, mr_iid: i64) -> Result<MergeRequest> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/merge",
            self.base_url, project_id, mr_iid
        );

        let resp = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        let status = resp.status().as_u16();
        if status == 405 {
            let _body = resp.text().await.unwrap_or_default();
            return Err(RingError::ArchiveConflict {
                record_id: format!("mr-{}", mr_iid),
            });
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError { status, message: body });
        }

        resp.json::<MergeRequest>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn close_mr(&self, project_url: &str, mr_iid: i64) -> Result<MergeRequest> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}",
            self.base_url, project_id, mr_iid
        );

        let resp = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(&serde_json::json!({
                "state_event": "close"
            }))
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError { status, message: body });
        }

        resp.json::<MergeRequest>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn get_mr_diffs(
        &self,
        project_url: &str,
        mr_iid: i64,
    ) -> Result<Vec<DiffRef>> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/diffs",
            self.base_url, project_id, mr_iid
        );

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError { status, message: body });
        }

        #[derive(Deserialize)]
        struct MrDiffsResponse {
            diffs: Vec<DiffRef>,
        }

        let result: MrDiffsResponse = resp.json().await.map_err(|e| RingError::GitlabApiError {
            status: 0,
            message: format!("parse error: {e}"),
        })?;

        Ok(result.diffs)
    }
}