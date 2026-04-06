use serde::Deserialize;
use serde_json::json;

use crate::error::{Result, RingError};

#[derive(Debug, Clone)]
pub struct GitlabService {
    pub base_url: String,
    pub token: String,
    pub client: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRepoResponse {
    pub id: i64,
    #[serde(rename = "http_url_to_repo")]
    pub url: String,
    #[serde(rename = "ssh_url_to_repo")]
    pub ssh_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MergeRequestInfo {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub author: MergeRequestAuthor,
    pub state: String,
    #[serde(rename = "web_url")]
    pub web_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MergeRequestAuthor {
    pub username: String,
}

impl MergeRequestInfo {
    pub fn author(&self) -> &str {
        &self.author.username
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MrDiff {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

impl GitlabService {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_repo(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<CreateRepoResponse> {
        let url = format!("{}/api/v4/projects", self.base_url);
        let mut body = json!({
            "name": name,
        });
        if let Some(ns) = namespace {
            body["namespace"] = json!(ns);
        }
        let resp = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab request failed: {}", e)))?;
        self.handle_response(resp).await
    }

    pub async fn create_mr(
        &self,
        project_id: i64,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<MergeRequestInfo> {
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests",
            self.base_url, project_id
        );
        let body = json!({
            "source_branch": source_branch,
            "target_branch": target_branch,
            "title": title,
        });
        let resp = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab request failed: {}", e)))?;
        self.handle_response(resp).await
    }

    pub async fn merge_mr(&self, project_id: i64, mr_iid: i64) -> Result<MergeRequestInfo> {
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
            .map_err(|e| RingError::Internal(format!("gitlab request failed: {}", e)))?;
        self.handle_response(resp).await
    }

    pub async fn close_mr(&self, project_id: i64, mr_iid: i64) -> Result<MergeRequestInfo> {
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}",
            self.base_url, project_id, mr_iid
        );
        let body = json!({
            "state_event": "close",
        });
        let resp = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab request failed: {}", e)))?;
        self.handle_response(resp).await
    }

    pub async fn list_mrs(&self, project_id: i64, state: &str) -> Result<Vec<MergeRequestInfo>> {
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests?state={}",
            self.base_url, project_id, state
        );
        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab request failed: {}", e)))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "gitlab api error: {} - {}",
                status, body
            )));
        }
        resp.json::<Vec<MergeRequestInfo>>()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab response parse error: {}", e)))
    }

    pub async fn get_mr_diff(&self, project_id: i64, mr_iid: i64) -> Result<Vec<MrDiff>> {
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
            .map_err(|e| RingError::Internal(format!("gitlab request failed: {}", e)))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "gitlab api error: {} - {}",
                status, body
            )));
        }
        resp.json::<Vec<MrDiff>>()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab response parse error: {}", e)))
    }

    pub fn get_repo_url(&self, project_path: &str) -> Result<String> {
        Ok(format!("{}/{}.git", self.base_url, project_path))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "gitlab api error: {} - {}",
                status, body
            )));
        }
        resp.json::<T>()
            .await
            .map_err(|e| RingError::Internal(format!("gitlab response parse error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_strips_trailing_slash() {
        let svc = GitlabService::new("https://gitlab.com/", "tok");
        assert_eq!(svc.base_url, "https://gitlab.com");
    }

    #[test]
    fn get_repo_url_constructs_clone_url() {
        let svc = GitlabService::new("https://gitlab.example.com", "tok");
        let url = svc.get_repo_url("group/project").unwrap();
        assert_eq!(url, "https://gitlab.example.com/group/project.git");
    }

    #[test]
    fn parse_merge_request_response() {
        let json = r#"{
            "id": 1,
            "iid": 7,
            "title": "Add feature",
            "author": {"username": "alice"},
            "state": "merged",
            "web_url": "https://gitlab.com/group/project/-/merge_requests/7"
        }"#;
        let mr: MergeRequestInfo = serde_json::from_str(json).unwrap();
        assert_eq!(mr.id, 1);
        assert_eq!(mr.iid, 7);
        assert_eq!(mr.title, "Add feature");
        assert_eq!(mr.author(), "alice");
        assert_eq!(mr.state, "merged");
        assert_eq!(
            mr.web_url,
            "https://gitlab.com/group/project/-/merge_requests/7"
        );
    }

    #[test]
    fn parse_create_repo_response() {
        let json = r#"{
            "id": 42,
            "http_url_to_repo": "https://gitlab.com/group/project.git",
            "ssh_url_to_repo": "git@gitlab.com:group/project.git"
        }"#;
        let repo: CreateRepoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(repo.id, 42);
        assert_eq!(repo.url, "https://gitlab.com/group/project.git");
        assert_eq!(repo.ssh_url, "git@gitlab.com:group/project.git");
    }

    #[test]
    fn parse_mr_diff_response() {
        let json = r#"[
            {
                "old_path": "src/main.rs",
                "new_path": "src/main.rs",
                "diff": "@@ -1,3 +1,4 @@\n+use new_mod;\n fn main() {}"
            }
        ]"#;
        let diffs: Vec<MrDiff> = serde_json::from_str(json).unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].old_path, "src/main.rs");
        assert_eq!(diffs[0].new_path, "src/main.rs");
        assert!(diffs[0].diff.contains("use new_mod"));
    }

    #[test]
    fn parse_merge_request_list() {
        let json = r#"[
            {
                "id": 10,
                "iid": 1,
                "title": "Fix bug",
                "author": {"username": "bob"},
                "state": "opened",
                "web_url": "https://gitlab.com/g/p/-/merge_requests/1"
            },
            {
                "id": 11,
                "iid": 2,
                "title": "Add docs",
                "author": {"username": "carol"},
                "state": "opened",
                "web_url": "https://gitlab.com/g/p/-/merge_requests/2"
            }
        ]"#;
        let mrs: Vec<MergeRequestInfo> = serde_json::from_str(json).unwrap();
        assert_eq!(mrs.len(), 2);
        assert_eq!(mrs[0].title, "Fix bug");
        assert_eq!(mrs[1].author(), "carol");
    }
}
