//! HTTP Client wrapper for ve-server API endpoints

use anyhow::{Context, Result};
use reqwest::{Client, Method, Response};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use ve_shared::models::PermissionDecision;

/// API client for ve-server
pub struct MockClient {
    http: Client,
    server_url: String,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiEnvelope<T> {
    pub success: bool,
    pub data: T,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileTreePayload {
    pub tree: ve_shared::models::FileTreeNode,
}

impl MockClient {
    pub fn new(server_url: String, token: String) -> Self {
        Self {
            http: Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            server_url,
            token,
        }
    }

    /// Create a client without authentication (for unauthenticated endpoints
    /// like POST /api/auth/register-device).
    pub fn unauthenticated(server_url: String) -> Self {
        Self {
            http: Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            server_url,
            token: String::new(),
        }
    }

    // ---- Auth API ----

    pub async fn register_device(
        &self,
        device_name: &str,
        device_type: ve_shared::types::DeviceType,
        server_url: &str,
        idempotency_key: &str,
    ) -> Result<RegisterDeviceResponse> {
        let body = serde_json::json!({
            "device_name": device_name,
            "device_type": device_type,
            "server_url": server_url,
        });
        let resp = self
            .post_json_value_raw("/api/auth/register-device", body, Some(idempotency_key))
            .await?;
        serde_json::from_value(resp).with_context(|| "parsing register-device response")
    }

    pub async fn pairing_status(
        &self,
        host_id: Uuid,
        pairing_secret: &str,
    ) -> Result<ve_shared::models::PairingStatusResponse> {
        let url = format!("/api/auth/pairing-status?host_id={host_id}");
        let resp = self
            .http
            .request(Method::GET, format!("{}{}", self.server_url, url))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("x-pairing-secret", pairing_secret)
            .send()
            .await
            .with_context(|| format!("request to {url}"))?;
        let status = resp.status();
        let text = resp.text().await.context("reading response body")?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "GET {url} failed with status {status}: {text}"
            ));
        }
        serde_json::from_str(&text).with_context(|| format!("parsing JSON: {text}"))
    }

    pub async fn pair(&self, pair_code: &str) -> Result<ve_shared::models::PairResponse> {
        let body = serde_json::json!({ "pair_code": pair_code });
        let resp = self
            .post_json_value_raw("/api/auth/pair", body, None)
            .await?;
        serde_json::from_value(resp).context("parsing pair response")
    }

    // ---- Hosts API ----

    pub async fn list_hosts(&self) -> Result<ve_shared::models::HostListResponse> {
        let resp = self.get_json("/api/hosts").await?;
        serde_json::from_value(resp).context("parsing list_hosts response")
    }

    // ---- Workspaces API ----

    pub async fn create_workspace(
        &self,
        host_id: Uuid,
        path: &str,
        display_name: Option<&str>,
    ) -> Result<ve_shared::models::Workspace> {
        let mut body = serde_json::json!({
            "host_id": host_id,
            "path": path,
        });
        if let Some(dn) = display_name {
            body["display_name"] = serde_json::Value::String(dn.to_string());
        }
        let resp = self.post_json_value("/api/workspaces", body).await?;
        serde_json::from_value(resp).context("parsing create_workspace response")
    }

    pub async fn list_workspaces(
        &self,
        host_id: Uuid,
    ) -> Result<ve_shared::types::Paginated<ve_shared::models::Workspace>> {
        let resp = self
            .get_json(&format!("/api/workspaces?host_id={host_id}"))
            .await?;
        serde_json::from_value(resp).context("parsing list_workspaces response")
    }

    pub async fn get_workspace(&self, id: Uuid) -> Result<ve_shared::models::Workspace> {
        let resp = self.get_json(&format!("/api/workspaces/{id}")).await?;
        serde_json::from_value(resp).context("parsing get_workspace response")
    }

    pub async fn update_workspace(
        &self,
        id: Uuid,
        name: &str,
        _path: &str,
    ) -> Result<ve_shared::models::Workspace> {
        let resp = self
            .post_json_value(
                &format!("/api/workspaces/{id}"),
                serde_json::json!({ "display_name": name }),
            )
            .await?;
        serde_json::from_value(resp).context("parsing update_workspace response")
    }

    pub async fn delete_workspace(&self, id: Uuid) -> Result<()> {
        let resp = self.delete(&format!("/api/workspaces/{id}")).await?;
        let status = resp.status();
        let text = resp.text().await.context("reading delete response body")?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "DELETE workspace failed with status {status}: {text}"
            ));
        }
        Ok(())
    }

    // ---- Sessions API ----

    pub async fn create_session(
        &self,
        host_id: Uuid,
        workspace_id: Uuid,
        title: &str,
        initial_message: &str,
        idempotency_key: &str,
    ) -> Result<ve_shared::models::Session> {
        let body = serde_json::json!({
            "host_id": host_id,
            "workspace_id": workspace_id,
            "title": title,
            "initial_message": initial_message,
            "idempotency_key": idempotency_key,
        });
        let resp = self.post_json_value("/api/sessions", body).await?;
        serde_json::from_value(resp).context("parsing create_session response")
    }

    pub async fn list_sessions(&self) -> Result<Vec<ve_shared::models::Session>> {
        let resp = self.get_json("/api/sessions").await?;
        serde_json::from_value(resp).context("parsing list_sessions response")
    }

    pub async fn get_session(&self, id: Uuid) -> Result<ve_shared::models::Session> {
        let resp = self.get_json(&format!("/api/sessions/{id}")).await?;
        serde_json::from_value(resp).context("parsing get_session response")
    }

    pub async fn send_message(
        &self,
        session_id: Uuid,
        content: &str,
    ) -> Result<ve_shared::models::SendMessageResponse> {
        let body = serde_json::json!({ "content": content });
        let resp = self
            .post_json_value(&format!("/api/sessions/{session_id}/messages"), body)
            .await?;
        serde_json::from_value(resp).context("parsing send_message response")
    }

    pub async fn list_messages(
        &self,
        session_id: Uuid,
    ) -> Result<ve_shared::types::Paginated<ve_shared::models::SessionMessage>> {
        let resp = self
            .get_json(&format!("/api/sessions/{session_id}/messages"))
            .await?;
        serde_json::from_value(resp).context("parsing list_messages response")
    }

    pub async fn control_session(
        &self,
        session_id: Uuid,
        action: &str,
    ) -> Result<ve_shared::models::ControlSessionResponse> {
        let resp = self
            .post_json_value(
                &format!("/api/sessions/{session_id}/control"),
                serde_json::json!({ "action": action }),
            )
            .await?;
        serde_json::from_value(resp).context("parsing control_session response")
    }

    pub async fn close_session(
        &self,
        session_id: Uuid,
    ) -> Result<ve_shared::models::CloseSessionResponse> {
        let resp = self
            .request(
                Method::POST,
                &format!("/api/sessions/{session_id}/close"),
                None::<Value>,
            )
            .await?;
        let value =
            Self::parse_json_response(resp, &format!("/api/sessions/{session_id}/close")).await?;
        serde_json::from_value(value).context("parsing close_session response")
    }

    // ---- Permissions API ----

    pub async fn list_permissions(
        &self,
        session_id: Option<Uuid>,
    ) -> Result<Vec<ve_shared::models::PermissionRequest>> {
        let url = match session_id {
            Some(sid) => format!("/api/permissions?session_id={sid}"),
            None => "/api/permissions".to_string(),
        };
        let resp = self.get_json(&url).await?;
        serde_json::from_value(resp).context("parsing list_permissions response")
    }

    pub async fn respond_permission(
        &self,
        permission_id: Uuid,
        decision: PermissionDecision,
        note: Option<&str>,
    ) -> Result<ve_shared::models::PermissionRequest> {
        let mut body = serde_json::json!({
            "decision": decision,
        });
        if let Some(n) = note {
            body["note"] = serde_json::Value::String(n.to_string());
        }
        let resp = self
            .post_json_value(&format!("/api/permissions/{permission_id}/respond"), body)
            .await?;
        serde_json::from_value(resp).context("parsing respond_permission response")
    }

    // ---- Archives API ----

    pub async fn list_archives(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<ve_shared::types::Paginated<ve_shared::models::SessionArchive>> {
        let mut url = "/api/archives".to_string();
        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(format!("page={p}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let resp = self.get_json(&url).await?;
        serde_json::from_value(resp).context("parsing list_archives response")
    }

    pub async fn get_archive(&self, id: Uuid) -> Result<ve_shared::models::SessionArchive> {
        let resp = self.get_json(&format!("/api/archives/{id}")).await?;
        serde_json::from_value(resp).context("parsing get_archive response")
    }

    pub async fn batch_delete_archives(
        &self,
        ids: Vec<Uuid>,
    ) -> Result<ve_shared::models::BatchDeleteResponse> {
        let body = serde_json::json!({ "archive_ids": ids });
        let resp = self
            .post_json_value("/api/archives/batch-delete", body)
            .await?;
        serde_json::from_value(resp).context("parsing batch_delete_archives response")
    }

    // ---- Files API ----

    pub async fn get_file_tree(
        &self,
        host_id: Uuid,
        workspace_id: Uuid,
        path: Option<&str>,
    ) -> Result<ApiEnvelope<FileTreePayload>> {
        let mut url = format!("/api/hosts/{host_id}/files/tree?workspace_id={workspace_id}",);
        if let Some(p) = path {
            url.push_str(&format!("&path={}", urlencoding::encode(p)));
        }
        let resp = self.get_json(&url).await?;
        serde_json::from_value(resp).context("parsing get_file_tree response")
    }

    pub async fn get_file_content(
        &self,
        host_id: Uuid,
        workspace_id: Uuid,
        file_path: &str,
    ) -> Result<ApiEnvelope<ve_shared::models::FileContent>> {
        let url = format!(
            "/api/hosts/{host_id}/files/content?workspace_id={workspace_id}&path={}",
            urlencoding::encode(file_path)
        );
        let resp = self.get_json(&url).await?;
        serde_json::from_value(resp).context("parsing get_file_content response")
    }

    // ---- Settings API ----

    pub async fn get_notification_preferences(
        &self,
    ) -> Result<ve_shared::models::NotificationPreference> {
        let resp = self.get_json("/api/settings/notifications").await?;
        serde_json::from_value(resp).context("parsing get_notification_preferences response")
    }

    pub async fn update_notification_preferences(
        &self,
        enabled: bool,
        permission_request_enabled: bool,
        task_completed_enabled: bool,
        task_failed_enabled: bool,
        session_error_enabled: bool,
    ) -> Result<ve_shared::models::NotificationPreference> {
        let body = serde_json::json!({
            "enabled": enabled,
            "permission_request_enabled": permission_request_enabled,
            "task_completed_enabled": task_completed_enabled,
            "task_failed_enabled": task_failed_enabled,
            "session_error_enabled": session_error_enabled,
        });
        let resp = self
            .put_json_value("/api/settings/notifications", body)
            .await?;
        serde_json::from_value(resp).context("parsing update_notification_preferences response")
    }

    // ---- Low-level HTTP methods ----

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let resp = self.request(Method::GET, path, None::<Value>).await?;
        let status = resp.status();
        let text = resp.text().await.context("reading response body")?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "GET {path} failed with status {status}: {text}"
            ));
        }
        serde_json::from_str(&text).with_context(|| format!("parsing JSON: {text}"))
    }

    async fn post_json_value(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.post_json_value_raw(path, body, None).await
    }

    async fn post_json_value_raw(
        &self,
        path: &str,
        body: serde_json::Value,
        _idempotency_key: Option<&str>,
    ) -> Result<serde_json::Value> {
        let resp = self.request(Method::POST, path, Some(body)).await?;
        Self::parse_json_response(resp, path).await
    }

    async fn put_json_value(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self.request(Method::PUT, path, Some(body)).await?;
        Self::parse_json_response(resp, path).await
    }

    async fn delete(&self, path: &str) -> Result<Response> {
        self.request(Method::DELETE, path, None::<Value>).await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.server_url, path);
        let mut req = self
            .http
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            req = req.json(&b);
        }

        req.send()
            .await
            .with_context(|| format!("request to {url}"))
    }

    async fn parse_json_response(resp: Response, path: &str) -> Result<serde_json::Value> {
        let status = resp.status();
        let text = resp.text().await.context("reading response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Request {path} failed with status {status}: {text}"
            ));
        }

        serde_json::from_str(&text).with_context(|| format!("parsing JSON: {text}"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn paginated_workspace_response_parses_correctly() {
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let response = json!({
            "items": [
                {
                    "workspace_id": workspace_id,
                    "host_id": host_id,
                    "path": "/tmp/workspace",
                    "display_name": "workspace",
                    "is_favorited": false,
                    "last_used_at": null,
                    "exists_on_host": true,
                    "created_at": "2026-04-23T00:00:00Z",
                    "updated_at": "2026-04-23T00:00:00Z"
                }
            ],
            "total": 1,
            "page": 1,
            "limit": 20,
            "has_more": false
        });

        let page: ve_shared::types::Paginated<ve_shared::models::Workspace> =
            serde_json::from_value(response).expect("paginated workspace response should parse");

        assert_eq!(page.total, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].workspace_id, workspace_id);
    }

    #[test]
    fn shared_pair_response_parses_full_contract() {
        let host_id = Uuid::new_v4();
        let response = json!({
            "host_id": host_id,
            "host_name": "host-a",
            "token": "client-token",
        });

        let parsed: ve_shared::models::PairResponse =
            serde_json::from_value(response).expect("pair response should parse");

        assert_eq!(parsed.host_id, host_id);
        assert_eq!(parsed.host_name, "host-a");
        assert_eq!(parsed.token, "client-token");
    }

    #[test]
    fn shared_pairing_status_response_parses_daemon_token() {
        let response = json!({
            "status": "paired",
            "daemon_token": "daemon-token",
        });

        let parsed: ve_shared::models::PairingStatusResponse =
            serde_json::from_value(response).expect("pairing status response should parse");

        assert_eq!(parsed.status, "paired");
        assert_eq!(parsed.daemon_token.as_deref(), Some("daemon-token"));
    }

    #[test]
    fn typed_file_content_response_parses_truncation_metadata() {
        let response = json!({
            "success": true,
            "data": {
                "path": "README.md",
                "content": "hello",
                "file_type": "text",
                "truncated": true,
                "total_size": 42
            },
            "error": null
        });

        let parsed: super::ApiEnvelope<ve_shared::models::FileContent> =
            serde_json::from_value(response).expect("typed file content response should parse");

        assert!(parsed.success);
        assert_eq!(parsed.data.path, "README.md");
        assert!(parsed.data.truncated);
        assert_eq!(parsed.data.total_size, 42);
    }
}

// ---- Response types ----

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceResponse {
    pub device_id: Uuid,
    pub token: String,
}
